#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd -P)
python3 - "$root" <<'PY'
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(sys.argv[1])
prechecks = ROOT / "tests/ci/run_repository_prechecks.sh"
entry = ROOT / "tests/ci/run_work_item_verification.sh"

def run(args, cwd, env):
    return subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True)

def repository(path):
    path.mkdir()
    for args in (["git", "init", "-q"],
                 ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                  "commit", "-qm", "fixture baseline", "--allow-empty"]):
        result = run(args, path, os.environ.copy())
        assert result.returncode == 0, result.stderr
    return path

with tempfile.TemporaryDirectory(prefix="repository-precheck-fixture-") as temporary:
    temp = Path(temporary)
    repo = repository(temp / "source")
    log = temp / "calls.log"
    env = dict(os.environ, PRECHECK_PROBE_LOG=str(log))
    leaves = [
        ("tests/docs/governance_cost_baseline_test.py", "budget"),
        ("tests/ci/workflow_convergence_test.sh", "consumers"),
        ("tests/release/action_runtime_policy_test.sh", "pins"),
    ]
    for relative, name in leaves:
        path = repo / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.suffix == ".py":
            path.write_text("import os\nfrom pathlib import Path\n"
                            f"with Path(os.environ['PRECHECK_PROBE_LOG']).open('a') as f:f.write('{name}\\n')\n"
                            f"raise SystemExit(23 if os.environ.get('PRECHECK_PROBE_FAIL')=='{name}' else 0)\n")
        else:
            path.write_text(f"#!/usr/bin/env bash\nset -eu\nprintf '{name}\\n' >> \"$PRECHECK_PROBE_LOG\"\n"
                            f"if [[ ${{PRECHECK_PROBE_FAIL:-}} == {name} ]]; then exit 23; fi\n")
        path.chmod(0o755)
    result = run(["bash", str(prechecks), "--repo", str(repo)], ROOT, env)
    assert result.returncode == 0, f"valid source prechecks must run: {result.stderr}"
    assert log.read_text().splitlines() == ["budget", "consumers", "pins"]
    for index, (_, name) in enumerate(leaves):
        log.unlink()
        result = run(["bash", str(prechecks), "--repo", str(repo)], ROOT,
                     dict(env, PRECHECK_PROBE_FAIL=name))
        assert result.returncode == 23, (name, result.stderr)
        assert log.read_text().splitlines() == [item[1] for item in leaves[:index+1]]

    runtime = temp / "runtime"
    runtime.write_text("""#!/usr/bin/env python3
import json,os,sys
from pathlib import Path
args=sys.argv[1:]
log=Path(os.environ['PRECHECK_PROBE_LOG'])
def note(s):
    with log.open('a') as f:f.write(s+'\\n')
if args[0]=='verify':
    note('plan' if '--plan-only' in args else 'verify')
    print(json.dumps({'state':'planned','processesSpawned':0}))
elif args[:2]==['work-item','validate']:
    note('validate')
    blocked=os.environ.get('PRECHECK_PROBE_INVALID')=='1'
    print(json.dumps({'state':'blocked' if blocked else 'unknown',
      'scenarioCoverage':'unknown','acceptanceEvidence':'unknown' if blocked else 'verified',
      'evidenceClasses':'verified','intentAlignment':'unknown','finalDimensions':'not_applicable',
      'unknowns':['source_usage_unavailable'],
      'findings':[{'code':'acceptance_id_invalid','severity':'error','message':'invalid fixture ID'}] if blocked else []}))
elif args[:2]==['work-item','status']:
    note('status')
    actions=['run_verification']
    if os.environ.get('PRECHECK_PROBE_PREFLIGHT')=='1' and not log.with_suffix('.refreshed').exists():actions=['run_preflight']
    print(json.dumps({'schemaVersion':1,'workItemId':'WI-PRECHECK',
      'lifecyclePhase':'checkpointed','humanDecisionRequired':False,'blocking':False,
      'safeActions':[] if os.environ.get('PRECHECK_PROBE_INVALID')=='1' else actions}))
elif args[0]=='preflight':
    note('preflight')
    assert '--contract' in args and '--work-item' not in args
    contract=Path(args[args.index('--contract')+1])
    assert contract.name=='WI-PRECHECK.contract.json' and contract.is_file()
    log.with_suffix('.refreshed').touch()
    print(json.dumps({'state':'ready'}))
else:raise SystemExit('unexpected Runtime call: '+repr(args))
""")
    runtime.chmod(0o755)
    args = ["bash", str(entry), "--runtime", str(runtime), "--repo", str(repo), "--work-item", "WI-PRECHECK"]
    log.unlink(missing_ok=True)
    result = run(args, ROOT, env)
    assert result.returncode == 0, result.stderr
    calls = log.read_text().splitlines()
    assert calls[-4:] == ["budget", "consumers", "pins", "verify"], calls
    assert calls.count("verify") == 1
    contract = repo / '.ai/work-items/active/WI-PRECHECK.contract.json'
    contract.parent.mkdir(parents=True)
    contract.write_text('{}')
    log.unlink()
    result = run(args, ROOT, dict(env, PRECHECK_PROBE_PREFLIGHT="1"))
    assert result.returncode == 0, result.stderr
    calls = log.read_text().splitlines()
    assert calls == ["validate", "status", "preflight", "status", "budget", "consumers", "pins", "verify"], calls
    for name in [item[1] for item in leaves]:
        log.unlink()
        result = run(args, ROOT, dict(env, PRECHECK_PROBE_FAIL=name))
        assert result.returncode != 0
        assert "verify" not in log.read_text().splitlines()
    log.unlink()
    result = run(args, ROOT, dict(env, PRECHECK_PROBE_INVALID="1"))
    assert result.returncode != 0
    assert "verify" not in log.read_text().splitlines()
    assert "budget" not in log.read_text().splitlines()
    log.unlink()
    result = run(args+["--plan-only"], ROOT, env)
    assert result.returncode == 0, result.stderr
    assert log.read_text().splitlines() == ["plan"]
    for extra in [["--repo", str(temp)], ["--command", "anything"], ["--unknown-argument"]]:
        log.unlink(missing_ok=True)
        result = run(args+extra, ROOT, env)
        assert result.returncode != 0
        assert not log.exists(), extra

    # Exercise the existing canonical ID/control rules in real isolated fixtures.
    # A fake Runtime must never stand in for this format/atomicity regression.
    actual = os.environ.get("AI_COCKPIT_TEST_RUNTIME") or shutil.which("ai-cockpit")
    if not actual:
        actual = str(ROOT / "target/release/ai-cockpit")
    assert Path(actual).is_file(), "provide an existing compiled fixture Runtime; do not build in a cheap check"
    for number, acceptance, expected in [(0, "S01-long-scenario", False), (1, "S01", True)]:
        native = repository(temp / f"native-{number}")
        def native_run(args):
            return run([actual, *args], native, os.environ.copy())
        attached = native_run(["attach", "--repo", str(native)])
        assert attached.returncode == 0, attached.stderr
        wi = "WI-PRECHECK-ID"
        started = native_run(["start", "--repo", str(native), "--id", wi,
            "--intent", "fixture planning", "--goal", "validate planned mapping before execution",
            "--scope", ".ai/**", "--authority", "authorized",
            "--acceptance", f"{acceptance}: unchanged acceptance text"])
        assert started.returncode == 0, started.stderr
        payload = {"acceptanceEvidence":[{"acceptanceId":acceptance,"evidence":[
            {"type":"plan","path":f".ai/work-items/active/{wi}.contract.json",
             "locator":"/acceptanceCriteria/0","verification":"unverified"}]}],
             "intentAlignment":{"state":"unknown","reason":"fixture implementation not executed"}}
        input_path = native / "planned-controls.json"
        input_path.write_text(json.dumps(payload))
        recorded = native_run(["work-item","controls","--repo",str(native),"--id",wi,"--input",str(input_path)])
        assert (recorded.returncode == 0) == expected, recorded.stderr
        summary = json.loads((native/f".ai/work-items/active/{wi}.summary.json").read_text())
        if expected:
            assert summary["acceptanceEvidence"][0]["evidence"][0]["verification"] == "unverified"
        else:
            assert "acceptance_id_invalid" in recorded.stderr
            assert "acceptanceEvidence" not in summary
print("repository prechecks and canonical acceptance-ID boundary regressions passed")
PY
