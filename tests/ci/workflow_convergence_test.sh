#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
workflow="$root/.github/workflows/ci.yml"
route="$root/tests/ci/quality_route.py"
runner="$root/tests/ci/run_repository_gates.py"

python3 - "$workflow" "$route" "$runner" <<'PY'
from pathlib import Path
import importlib.util
import json
import subprocess
import tempfile
import sys

workflow = Path(sys.argv[1]).read_text(encoding="utf-8")
route = Path(sys.argv[2]).read_text(encoding="utf-8")
runner = Path(sys.argv[3]).read_text(encoding="utf-8")

# Pull-request runs must converge by cancelling only superseded runs for the
# same PR.  Main pushes and release workflow truth must not be cancellable by
# this policy.
assert "concurrency:" in workflow
assert "group: ai-cockpit-quality-${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}" in workflow
assert "cancel-in-progress: ${{ github.event_name == 'pull_request' }}" in workflow

# Heavy cross-platform/oracle jobs consume the same dynamic route as quality;
# a light documentation route must not start them.
assert "needs: route" in workflow
assert "needs.route.outputs.profile != 'light'" in workflow
assert "name: Plan the dynamic quality route" in workflow
assert "outputs:" in workflow and "profile:" in workflow
assert "Verify the route plan is stable across jobs" in workflow

# The route boundary must reject known illegal lifecycle transitions before
# repository gates run, with a stable code and remediation rather than a raw
# traceback or a second copy of the same failure.
assert "lifecycle_transition_invalid" in route
assert "lifecycle_transition_stale" in route
assert "remediation" in route
assert "failure_code" in runner
assert "failureRoots" in runner

# A failing manifest command is represented once in the machine report. Raw
# stderr is captured as diagnostic data and never becomes a second apparent
# gate failure in the hosted log.
route_spec = importlib.util.spec_from_file_location("quality_route", Path(sys.argv[2]))
assert route_spec is not None and route_spec.loader is not None
route_module = importlib.util.module_from_spec(route_spec)
route_spec.loader.exec_module(route_module)
with tempfile.TemporaryDirectory(prefix="ai-cockpit-convergence-runner-") as temporary:
    fixture = Path(temporary)
    repository = fixture / "repo"
    repository.mkdir()
    subprocess.run(["git", "init", "-q", str(repository)], check=True)
    subprocess.run(["git", "-C", str(repository), "config", "user.name", "Convergence Test"], check=True)
    subprocess.run(["git", "-C", str(repository), "config", "user.email", "convergence@example.invalid"], check=True)
    (repository / "README.md").write_text("base\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(repository), "add", "README.md"], check=True)
    subprocess.run(["git", "-C", str(repository), "commit", "-qm", "base"], check=True)
    base = subprocess.check_output(["git", "-C", str(repository), "rev-parse", "HEAD"], text=True).strip()
    (repository / "README.md").write_text("head\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(repository), "add", "README.md"], check=True)
    subprocess.run(["git", "-C", str(repository), "commit", "-qm", "head"], check=True)
    head = subprocess.check_output(["git", "-C", str(repository), "rev-parse", "HEAD"], text=True).strip()
    manifest_path = fixture / "manifest.json"
    manifest_path.write_text(json.dumps({
        "schemaVersion": 2,
        "profileOrder": ["light", "standard", "strict"],
        "unknownProfile": "strict",
        "pathProfiles": {"light": ["docs/**"], "standard": ["src/**"], "strict": [".github/**"]},
        "releaseOwnedPatterns": ["release/**"],
        "stageFloors": {"task": "light", "pre_ci": "light", "pull_request": "light", "merge": "strict", "release": "strict"},
        "gates": [{
            "category": "fixture",
            "command": ["python3", "-c", "import sys; print('expected negative diagnostic', file=sys.stderr); sys.exit(1)"],
            "id": "fixture_failure",
            "minimumProfile": "light",
        }],
    }, sort_keys=True), encoding="utf-8")
    receipt = route_module.plan_repository_route(
        repository=repository,
        manifest_path=manifest_path,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile=None,
    )
    receipt_path = fixture / "route.json"
    receipt_path.write_text(json.dumps(receipt, sort_keys=True), encoding="utf-8")
    report_path = fixture / "report.json"
    run = subprocess.run([
        sys.executable,
        str(Path(sys.argv[3])),
        "--repo", str(repository),
        "--manifest", str(manifest_path),
        "--route-receipt", str(receipt_path),
        "--report", str(report_path),
    ], check=False, capture_output=True, text=True)
    assert run.returncode == 1
    report = json.loads(report_path.read_text(encoding="utf-8"))
    assert report["state"] == "failed"
    assert len(report["failureRoots"]) == 1
    assert report["failureRoots"][0]["code"] == "quality_gate_failed:fixture_failure"
    assert "expected negative diagnostic" not in run.stdout
    assert "expected negative diagnostic" not in run.stderr

    active = repository / ".ai/work-items/active"
    active.mkdir(parents=True)
    (active / "WI-LIFECYCLE.contract.json").write_text('{"risk":"normal"}\n', encoding="utf-8")
    (active / "WI-LIFECYCLE.summary.json").write_text(
        json.dumps({"state": "checkpointed", "checkpointCount": 0, "preflightState": "yellow"}),
        encoding="utf-8",
    )
    invalid_route = subprocess.run([
        sys.executable,
        str(Path(sys.argv[2])),
        "--repo", str(repository),
        "--manifest", str(manifest_path),
        "--base", base,
        "--head", head,
        "--stage", "pull_request",
        "--contract", str(active / "WI-LIFECYCLE.contract.json"),
        "--receipt", str(fixture / "invalid-route.json"),
    ], check=False, capture_output=True, text=True)
    assert invalid_route.returncode != 0
    assert '"failureCode": "lifecycle_transition_invalid"' in invalid_route.stderr
    assert "Traceback" not in invalid_route.stderr

print("workflow convergence policy regression passed")
PY
