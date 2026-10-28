#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
workflow="$root/.github/workflows/ci.yml"
route="$root/tests/ci/quality_route.py"
runner="$root/tests/ci/run_repository_gates.py"
resolver="$root/tests/ci/resolve_work_item.sh"

python3 - "$workflow" "$route" "$runner" "$resolver" <<'PY'
from pathlib import Path
import hashlib
import importlib.util
import json
import re
import subprocess
import tempfile
import sys

workflow = Path(sys.argv[1]).read_text(encoding="utf-8")
route = Path(sys.argv[2]).read_text(encoding="utf-8")
runner = Path(sys.argv[3]).read_text(encoding="utf-8")
resolver = Path(sys.argv[4]).read_text(encoding="utf-8") if Path(sys.argv[4]).exists() else ""

# The fixed public corpus is raw test data, including its final LF. Check its
# checkout contract before compilation, without changing the payload or scanner.
corpus_root = Path(sys.argv[1]).resolve().parents[2]
corpus_path = "tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
corpus_rule = f"/{corpus_path} -text"
corpus_digest = "a71f4b8cce7538d11f17367cea5e106aa152268c57624e5890fffaf2eb38a814"

def require_corpus_bytes(raw):
    assert len(raw) == 43, "fixed corpus must retain its original 43 bytes"
    assert hashlib.sha256(raw).hexdigest() == corpus_digest, "fixed corpus SHA256 mismatch"

def corpus_git(repository, *args):
    return subprocess.run(
        ["git", "-C", str(repository), "-c", "maintenance.autoDetach=false",
         "-c", "gc.autoDetach=false", *args],
        check=True, stdout=subprocess.PIPE,
    ).stdout

def corpus_attributes(repository, paths):
    fields = corpus_git(repository, "check-attr", "-z", "text", "diff", "--", *paths).split(b"\0")[:-1]
    assert len(fields) == len(paths) * 6
    return {(fields[i].decode(), fields[i + 1].decode()): fields[i + 2].decode()
            for i in range(0, len(fields), 3)}

corpus_raw = corpus_git(corpus_root, "show", f"HEAD:{corpus_path}")
require_corpus_bytes(corpus_raw)
corpus_rules = (corpus_root / ".gitattributes").read_text(encoding="utf-8").splitlines()
corpus_old_rules = [line for line in corpus_rules if line.strip() != corpus_rule]
with tempfile.TemporaryDirectory(prefix="fixed-corpus-checkout-") as temporary:
    checkout = Path(temporary)
    corpus_git(checkout, "init", "-q")
    corpus_git(checkout, "config", "--local", "core.autocrlf", "false")
    target = checkout / corpus_path
    target.parent.mkdir(parents=True)
    target.write_bytes(corpus_raw)
    attributes = checkout / ".gitattributes"
    attributes.write_text("\n".join(corpus_old_rules) + "\n", encoding="utf-8")
    corpus_git(checkout, "add", ".")
    corpus_git(checkout, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
               "commit", "-qm", "original corpus byte contract")
    corpus_parent = corpus_path.rsplit("/", 1)[0]
    probes = [corpus_path, corpus_path + ".other", f"{corpus_parent}/other.txt",
              f"{corpus_parent}/nested/material.txt",
              ".ai/evidence/fixture.json", "src/other.rs"]
    before_attributes = corpus_attributes(checkout, probes)
    target.unlink()  # Materialize a fresh checkout instead of reusing index stat data.
    corpus_git(checkout, "-c", "core.autocrlf=true", "checkout-index", "--force", "--", corpus_path)
    old_checkout = target.read_bytes()
    assert len(old_checkout) == 44
    assert hashlib.sha256(old_checkout).hexdigest() == "378a97fa8af550855178dc5e8733f0dd5fb2b88b3270cef6c1d885959cd12fe1"
    assert old_checkout != corpus_raw, "old rules must reproduce the CRLF transport defect"
    attributes.write_text("\n".join([*corpus_old_rules, corpus_rule]) + "\n", encoding="utf-8")
    corpus_git(checkout, "add", ".gitattributes")
    corpus_git(checkout, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
               "commit", "-qm", "preserve only the fixed corpus bytes")
    after_attributes = corpus_attributes(checkout, probes)
    assert {key for key in before_attributes if before_attributes[key] != after_attributes[key]} == {(corpus_path, "text")}
    assert after_attributes[(corpus_path, "text")] == "unset"
    assert after_attributes[(corpus_path, "diff")] != "unset", "the corpus must retain visible text diffs"
    for mode in ("false", "true", "input"):
        target.unlink()
        corpus_git(checkout, "-c", f"core.autocrlf={mode}", "checkout-index", "--force", "--", corpus_path)
        assert target.read_bytes() == corpus_raw, mode
        require_corpus_bytes(target.read_bytes())
    # Corrupt only an isolated fixture blob; the real public corpus is unchanged.
    target.write_bytes(b"X" + corpus_raw[1:])
    corpus_git(checkout, "add", "--", corpus_path)
    corpus_git(checkout, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
               "commit", "-qm", "negative corpus identity fixture")
    try:
        require_corpus_bytes(corpus_git(checkout, "show", f"HEAD:{corpus_path}"))
    except AssertionError as error:
        assert str(error) == "fixed corpus SHA256 mismatch"
    else:
        raise AssertionError("a changed corpus blob must be rejected")
assert corpus_rules.count(corpus_rule) == 1, "the fixed corpus requires one exact -text rule"
source_attributes = corpus_attributes(corpus_root, [corpus_path])
assert source_attributes[(corpus_path, "text")] == "unset"
assert source_attributes[(corpus_path, "diff")] != "unset"
require_corpus_bytes((corpus_root / corpus_path).read_bytes())
print("fixed corpus byte contract: old CRLF defect, false/true/input, exact scope and tamper rejection passed")

def job_block(name):
    marker = f"  {name}:\n"
    start = workflow.index(marker, workflow.index("jobs:\n"))
    remaining = workflow[start + len(marker):]
    next_job = re.search(r"(?m)^  [A-Za-z0-9_-]+:\s*$", remaining)
    return workflow[start:] if next_job is None else workflow[start:start + len(marker) + next_job.start()]

# The managed cheap barrier must precede the first Rust build in every
# source context, including release-input planning before a tag source exists.
precheck = "bash tests/ci/run_repository_prechecks.sh --repo"
route_job = job_block("route")
assert precheck in route_job, "CI must run cheap prechecks before the shared Rust build"
assert route_job.index(precheck) < route_job.index("cargo build")
release_workflow = Path(sys.argv[1]).with_name("release.yml").read_text(encoding="utf-8")
for name in ("release_input_preflight", "release_preflight"):
    remaining = release_workflow.split(f"  {name}:\n", 1)[1]
    match = re.search(r"(?m)^  [A-Za-z0-9_-]+:\s*$", remaining)
    block = remaining if match is None else remaining[:match.start()]
    assert precheck in block, f"{name} omits canonical cheap prechecks"
    assert block.index(precheck) < block.index("cargo build"), name
assert 'run_repository_prechecks.sh --repo "$GITHUB_WORKSPACE/release-source"' in release_workflow

# Pull-request runs must converge by cancelling only superseded runs for the
# same PR.  Main pushes and release workflow truth must not be cancellable by
# this policy.
assert "RUSTFLAGS: -D warnings" in workflow
assert "RUSTDOCFLAGS: -D warnings" in workflow
assert "name: verify Windows zero-warning workspace Clippy" in workflow
assert "components: clippy" in job_block("windows-runtime-suites")
assert "run: python tests/ci/run_no_warnings.py -- cargo clippy --locked --workspace --all-targets --all-features -- -D warnings" in workflow
assert "skip-decompress: true" in workflow
assert "digest-mismatch: error" in workflow
assert "actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c" in workflow
assert "concurrency:" in workflow
assert "group: ai-cockpit-quality-${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}" in workflow
assert "cancel-in-progress: ${{ github.event_name == 'pull_request' }}" in workflow

# Heavy cross-platform/oracle jobs consume the same dynamic route as quality;
# a light documentation route must not start them.
assert "needs: route" in workflow
assert "needs.route.outputs.profile != 'light'" in workflow
assert "name: Plan the dynamic quality route" in workflow
assert "resolve_work_item.sh" in workflow
assert workflow.index("name: Plan the dynamic quality route") < workflow.index("name: Build the shared Rust gate-plan tool once")
assert "EVENT_BEFORE" not in workflow[workflow.index("name: Plan the dynamic quality route"):workflow.index("name: Upload the shared Rust gate-plan tool")]
assert "no active or exact archived Contract is explicitly bound to this event identity" in resolver
assert "jq -r '.baseRevision // empty' target/quality-selection.json" in workflow
assert "work_item_id_required" in resolver
assert "outputs:" in workflow and "profile:" in workflow
assert "Verify the route plan is stable across jobs" in workflow
assert "PR_HEAD_SHA: ${{ github.event.pull_request.head.sha }}" in workflow
assert 'head_revision="$(git rev-parse "${PR_HEAD_SHA}^{commit}")"' in workflow
assert workflow.count("name: Bind source and tested revisions") == 2
assert 'target/ci-revision-binding.json' in workflow

# Splitting Windows work must preserve every original Rust command exactly
# once, keep bounded sibling execution, and retain the stable fail-closed job.
windows_suites = job_block("windows-runtime-suites")
assert "suite: [boundaries, repository-library]" in windows_suites
assert "fail-fast: false" in windows_suites and "max-parallel: 2" in windows_suites
windows_original_commands = ['python tests/ci/run_no_warnings.py -- cargo clippy --locked --workspace --all-targets --all-features -- -D warnings', 'python tests/ci/run_no_warnings.py -- cargo test -p cockpit-agent --all-targets', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-verification --lib --test execution --test composition -- --test-threads=1', 'python tests/ci/run_no_warnings.py -- cargo test -p cockpit-repository --test receipt_store --test verification_context --test verification_service', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-repository --test task_authorization -- --test-threads=1', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-cli --test contract_amendment_processes explicit_task_caller_and_resumption_assessment_match_cli_and_mcp -- --exact --test-threads=1', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-mcp --test rpc mcp_staged_start_preserves_the_complete_plan_without_checkpointing -- --exact --test-threads=1', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-repository --lib', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-mcp --test rpc runtime_preflight_waits_for_lifecycle_lock_and_rechecks_admission -- --exact', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-mcp --test rpc runtime_controls_wait_for_lifecycle_lock_and_recheck_admission_before_receipt_write -- --exact', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-cli --test cognitive_benefit windows_invalid_regular_binary_fails_without_fallback_or_artifact_writes -- --exact', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-cli --test cognitive_benefit generated_json_and_markdown_match_python_in_an_isolated_checkout -- --exact', 'python tests/ci/run_no_warnings.py -- cargo test --locked -p cockpit-cli --test cognitive_benefit rust_check_matches_the_existing_python_gate_without_rewriting_evidence -- --exact']
windows_steps = re.split(r"(?m)^      - ", windows_suites)
windows_actual_commands = []
for step in windows_steps:
    matches = re.findall(r"(?m)^        run: (python tests/ci/run_no_warnings.py -- cargo .+)$", step)
    windows_actual_commands.extend(matches)
    if matches:
        expected_suite = "repository-library" if matches[0].endswith("cockpit-repository --lib") else "boundaries"
        assert f"if: matrix.suite == '{expected_suite}'" in step, matches[0]
assert sorted(windows_actual_commands) == sorted(windows_original_commands), "Windows command coverage changed"
assert "name: ci-revision-binding-windows-${{ matrix.suite }}" in windows_suites
assert "name: Record successful Windows suite" in windows_suites
assert "target/windows-suite-result.json" in windows_suites
windows_aggregate = job_block("windows-runtime")
assert "needs: [route, windows-runtime-suites]" in windows_aggregate
assert "if: always() && (needs.route.result != 'success' || needs.route.outputs.profile != 'light')" in windows_aggregate
assert "SUITES_RESULT: ${{ needs.windows-runtime-suites.result }}" in windows_aggregate
assert "run: test \"$SUITES_RESULT\" = success" in windows_aggregate
for suite in ("boundaries", "repository-library"):
    assert f"name: ci-revision-binding-windows-{suite}" in windows_aggregate
assert windows_aggregate.count("skip-decompress: true") == 2
assert windows_aggregate.count("digest-mismatch: error") == 2

# Execute the exact aggregation program with bounded same-run ZIP fixtures;
# metadata existence alone must not hide a failed, cancelled or missing suite.
import copy
import os
import stat
import zipfile
aggregate_step = windows_aggregate.split("      - name: Verify complete same-run Windows suite receipts\n", 1)[1]
aggregate_lines = aggregate_step.split("        run: |\n", 1)[1].splitlines()
assert all(not line or line.startswith("          ") for line in aggregate_lines)
aggregate_shell = "\n".join(line[10:] if line else "" for line in aggregate_lines) + "\n"
with tempfile.TemporaryDirectory(prefix="windows-suite-aggregation-") as temporary:
    fixture_root = Path(temporary)
    tested, source, base = "a"*40, "b"*40, "c"*40
    binding = {"schemaVersion":1,"kind":"ci_revision_binding","eventName":"pull_request",
        "sourceHeadRevision":source,"routeHeadRevision":source,"testedRevision":tested,
        "eventBaseRevision":base,"testedBaseRevision":base,"testedParents":[base,source]}
    suites = ("boundaries", "repository-library")
    def exercise(case, mutate=None, result_state="success", skip=None, broken_zip=None, push=False):
        directory = fixture_root / case
        directory.mkdir()
        for suite in suites:
            if skip == suite:
                continue
            output = directory / "target/windows-results" / suite
            output.mkdir(parents=True)
            actual_binding = copy.deepcopy(binding)
            if push:
                actual_binding.update(eventName="push",sourceHeadRevision=tested,
                    routeHeadRevision=tested,eventBaseRevision="",testedParents=[base])
            result = {"schemaVersion":1,"state":"passed","suite":suite,"binding":actual_binding,
                "repository":"fixture/repo","runId":"1234","runAttempt":"1"}
            if mutate and suite == "boundaries":
                mutate(actual_binding, result)
            with zipfile.ZipFile(output/"artifact.zip", "w") as archive:
                archive.writestr("ci-revision-binding.json",json.dumps(actual_binding))
                if broken_zip != "missing-result" or suite != "boundaries":
                    info = zipfile.ZipInfo("windows-suite-result.json")
                    if broken_zip == "symlink" and suite == "boundaries":
                        info.external_attr = (stat.S_IFLNK | 0o777) << 16
                    archive.writestr(info,json.dumps(result))
                if broken_zip == "extra" and suite == "boundaries":
                    archive.writestr("unrelated.json", "{}")
        environment = dict(os.environ,SUITES_RESULT=result_state,EVENT_NAME="push" if push else "pull_request",
            GITHUB_SHA=tested,PR_HEAD_SHA=source,PR_BASE_SHA=base,GITHUB_REPOSITORY="fixture/repo",
            GITHUB_RUN_ID="1234",GITHUB_RUN_ATTEMPT="1")
        return subprocess.run(["bash","-c",aggregate_shell],cwd=directory,env=environment,
            capture_output=True,text=True)
    for case, push in (("complete-pr",False),("complete-push",True)):
        valid = exercise(case,push=push)
        assert valid.returncode == 0, valid.stderr
    for state in ("failure","cancelled","skipped",""):
        assert exercise("matrix-"+(state or "missing"),result_state=state).returncode != 0
    for suite in suites:
        assert exercise("missing-"+suite,skip=suite).returncode != 0
    for broken in ("missing-result","symlink","extra"):
        assert exercise("archive-"+broken,broken_zip=broken).returncode != 0
    for field,value in (("state","failed"),("repository","foreign/repo"),("runId","old"),
                        ("runAttempt","2"),("suite","foreign")):
        assert exercise("result-"+field,lambda b,r,f=field,v=value:r.update({f:v})).returncode != 0
    for field,value in (("sourceHeadRevision","d"*40),("testedRevision","d"*40),
                        ("eventBaseRevision","d"*40),("testedParents",[base]),
                        ("testedBaseRevision","d"*40)):
        assert exercise("binding-"+field,lambda b,r,f=field,v=value:b.update({f:v})).returncode != 0

# Independent hosted validations fan out from the immutable route artifact;
# no validation job waits for another validation job.
validation_jobs = ("quality", "windows-runtime-suites", "v1-behavioral-oracle")
for name in validation_jobs:
    block = job_block(name)
    assert "\n    needs: route\n" in block, f"{name} must depend on the shared route only"
    assert "needs: [" not in block, f"{name} must not serialize behind another validation job"
assert "\n    needs: quality\n" not in workflow, "independent CI validation must not wait for quality"

ordinary_guide = Path(sys.argv[1]).parents[2] / "agents/skills/ordinary-work-item.md"
ordinary_guide_text = re.sub(r"[\s`]+", " ", ordinary_guide.read_text(encoding="utf-8").lower())
for required_rule in (
    "keep lifecycle, snapshot-changing, and receipt producer-consumer actions serial",
    "verification defaults to --workers 1",
    "parallelism requires runtime-verified dependency readiness and output isolation",
    "independent ci jobs may fan out only with ready dependencies, isolated outputs, and bounded resources",
    "cross-work-item work requires supported cli/mcp",
    "declarations alone do not prove support",
    "otherwise use admitted serial work or stop",
    "reuse only fresh, complete evidence",
    "ready dependencies",
    "isolated outputs",
    "bounded resources",
    "before dependent actions, check environment drift read-only; record changes before refreshing admission",
):
    assert required_rule in ordinary_guide_text, f"ordinary guide omits parallel boundary: {required_rule}"

workflow_guide = re.sub(
    r"\s+", " ",
    (Path(sys.argv[1]).parents[2] / "docs/reference/agent-workflow.md").read_text(encoding="utf-8").lower(),
)
for required_rule in (
    "work item verification defaults to one worker",
    "requests above one worker are rejected until runtime can prove per-node dependency readiness and isolated outputs",
    "independent ci jobs can run as siblings",
    "package-coverage consumer waits for and validates the exact hosted runtime receipt",
):
    assert required_rule in workflow_guide, f"agent workflow omits parallel boundary: {required_rule}"

reference_root = Path(sys.argv[1]).parents[2] / "docs/reference"
command_guidance = {
    "commands.md": (
        "work item-bound verification is serial by default",
        "explicit --workers >1 fails closed",
        "per-node dependency readiness and isolated outputs",
    ),
    "commands.zh-CN.md": (
        "绑定 work item 的 verify 默认串行",
        "显式 --workers >1 会 fail closed",
        "逐节点依赖就绪和隔离输出",
    ),
    "commands.ja.md": (
        "work item に bind された verify は既定で serial です",
        "明示的な --workers >1 は fail closed",
        "node ごとの dependency readiness と isolated output",
    ),
}
for filename, required_rules in command_guidance.items():
    text = re.sub(r"[\s`]+", " ", (reference_root / filename).read_text(encoding="utf-8").lower())
    for required_rule in required_rules:
        assert required_rule in text, f"{filename} omits Work Item verification concurrency boundary: {required_rule}"

assert "GITHUB_EVENT_PATH" in workflow
assert "eventBaseRevision" in workflow and "testedBaseRevision" in workflow
assert "git merge-base --is-ancestor" in workflow
assert "exactly two parents" in workflow
assert "checked-out commit does not match GITHUB_SHA" in workflow
windows_job = job_block("windows-runtime-suites")
windows_checkout = windows_job.split("      - name: Bind source and tested revisions", 1)[0]
assert "fetch-depth: 0" in windows_checkout
assert "runtime_preflight_waits_for_lifecycle_lock_and_rechecks_admission" in windows_job
assert "runtime_controls_wait_for_lifecycle_lock_and_recheck_admission_before_receipt_write" in windows_job
windows_composition_step = windows_job.split(
    "      - name: verify Windows process and composition lifecycle boundaries", 1
)[1].split("      - name:", 1)[0]
assert "cargo test --locked -p cockpit-verification --lib --test execution --test composition -- --test-threads=1" in windows_composition_step
windows_upload_step = windows_job.split(
    "      - name: upload Windows revision binding", 1
)[1].split("      - name:", 1)[0]
assert "if: always()" in windows_upload_step
windows_binding = windows_job.split("      - name: Bind source and tested revisions", 1)[1].split(
    "      - uses: dtolnay/rust-toolchain", 1
)[0]
assert "Test-CiRevisionBindingCases" in windows_binding
for case in (
    "stale event base",
    "caller-supplied base",
    "non-ancestor event base",
    "wrong PR head",
    "checkout/GITHUB_SHA mismatch",
    "one-parent commit",
    "three-parent commit",
):
    assert case in windows_binding, f"Windows binding test omits {case}"
assert windows_binding.index("Set-Content -Encoding utf8 -LiteralPath $BindingPath") < windows_binding.index(
    "throw 'tested PR merge commit must have exactly two parents'"
), "write Windows revision diagnostics before lineage assertions"
quality_binding = workflow.split(
    "      - name: Bind source and tested revisions; validate the shared typed quality route", 1
)[1].split("      - name: verify immutable Runtime shadow", 1)[0]
assert quality_binding.index("> target/ci-revision-binding.json") < quality_binding.index(
    'test "$receipt_head" = "$source_revision"'
), "write revision diagnostics before any source/route binding assertion"

# Execute the actual route-binding shell from the workflow. An archived PR
# must be admitted to its read-only lane, while an unknown or mismatched
# method must fail before downstream package gates can start.
run_marker = "        run: |\n"
assert run_marker in quality_binding
run_lines = []
for line in quality_binding.split(run_marker, 1)[1].splitlines():
    if line.startswith("      - "):
        break
    if line and not line.startswith("          "):
        raise AssertionError(f"unexpected route-binding indentation: {line}")
    run_lines.append(line[10:] if line else "")
route_binding_script = "\n".join(run_lines) + "\n"
with tempfile.TemporaryDirectory(prefix="ai-cockpit-stage-lanes-") as temporary:
    repository = Path(temporary)
    def git(*args):
        return subprocess.check_output(["git", "-C", str(repository), *args], text=True).strip()
    subprocess.run(["git", "init", "-q", "-b", "main", str(repository)], check=True)
    git("config", "user.name", "Stage Lane Test")
    git("config", "user.email", "stage-lane@example.invalid")
    (repository / "README.md").write_text("base\n", encoding="utf-8")
    git("add", "README.md")
    git("commit", "-qm", "base")
    base = git("rev-parse", "HEAD")
    git("switch", "-q", "-c", "feature")
    (repository / "feature.txt").write_text("feature\n", encoding="utf-8")
    git("add", "feature.txt")
    git("commit", "-qm", "feature")
    source = git("rev-parse", "HEAD")
    git("switch", "-q", "main")
    (repository / "main.txt").write_text("advanced main\n", encoding="utf-8")
    git("add", "main.txt")
    git("commit", "-qm", "advance main after the PR event")
    tested_base = git("rev-parse", "HEAD")
    git("switch", "-q", "-c", "alternate", base)
    (repository / "alternate.txt").write_text("alternate head\n", encoding="utf-8")
    git("add", "alternate.txt")
    git("commit", "-qm", "alternate source head")
    alternate = git("rev-parse", "HEAD")
    git("switch", "-q", "main")
    git("merge", "-q", "--no-ff", "feature", "-m", "tested merge")
    tested = git("rev-parse", "HEAD")
    merge_tree = git("rev-parse", f"{tested}^{{tree}}")
    wrong_head_merge = git(
        "commit-tree", merge_tree, "-p", tested_base, "-p", alternate, "-m", "wrong PR head"
    )
    three_parent_merge = git(
        "commit-tree", merge_tree, "-p", tested_base, "-p", source, "-p", alternate,
        "-m", "three-parent merge",
    )
    unrelated_base = git("commit-tree", merge_tree, "-m", "unrelated event base")
    (repository / "target").mkdir()
    for area in ("active", "archive"):
        contract = repository / f".ai/work-items/{area}/WI-LANE.contract.json"
        contract.parent.mkdir(parents=True)
        contract.write_text("{}\n", encoding="utf-8")

    def run_lane(
        selection_method,
        contract_path,
        event="pull_request",
        *,
        checkout_revision=None,
        github_sha=None,
        event_base=None,
        context_base=None,
        event_head=None,
        context_head=None,
    ):
        checkout_revision = checkout_revision or tested
        github_sha = github_sha or checkout_revision
        event_base = event_base or base
        context_base = context_base or event_base
        event_head = event_head or source
        context_head = context_head or event_head
        git("checkout", "-q", "--detach", checkout_revision)
        stage = "pull_request" if event == "pull_request" else "merge"
        head = source if event == "pull_request" else tested
        (repository / "target/quality-selection.json").write_text(json.dumps({
            "selectionMethod": selection_method,
            "contractPath": contract_path,
        }), encoding="utf-8")
        (repository / "target/quality-route.json").write_text(json.dumps({
            "headRevision": head,
            "baseRevision": event_base if event == "pull_request" else base,
            "stage": stage,
            "contractPath": contract_path,
            "selectedProfile": "strict",
        }), encoding="utf-8")
        event_path = repository / "target/github-event.json"
        if event == "pull_request":
            event_path.write_text(json.dumps({
                "pull_request": {
                    "base": {"sha": event_base},
                    "head": {"sha": event_head},
                },
            }), encoding="utf-8")
        output = repository / "target/step-output"
        output.write_text("", encoding="utf-8")
        package_marker = repository / "target/package-process-started"
        package_marker.unlink(missing_ok=True)
        package_spy = repository / "target/package-spy.sh"
        package_spy.write_text(
            '#!/usr/bin/env bash\ntouch "$1"\n', encoding="utf-8"
        )
        environment = dict(__import__("os").environ)
        environment.update({
            "EVENT_NAME": event,
            "GITHUB_SHA": github_sha,
            "GITHUB_EVENT_PATH": str(event_path),
            "PR_HEAD_SHA": context_head,
            "PR_BASE_SHA": context_base,
            "GITHUB_OUTPUT": str(output),
        })
        result = subprocess.run(
            ["bash", "-c", route_binding_script + '\nbash target/package-spy.sh target/package-process-started\n'], cwd=repository,
            env=environment, capture_output=True, text=True,
        )
        binding_path = repository / "target/ci-revision-binding.json"
        binding = json.loads(binding_path.read_text(encoding="utf-8")) if binding_path.exists() else None
        return result, output.read_text(encoding="utf-8"), package_marker.exists(), binding

    active_path = ".ai/work-items/active/WI-LANE.contract.json"
    archived_path = ".ai/work-items/archive/WI-LANE.contract.json"
    for method, path, event, expected in (
        ("pull_request_binding", active_path, "pull_request", "active"),
        ("archived_contract_read_only", archived_path, "pull_request", "archived"),
        ("ordinary_repository_route", "", "pull_request", "ordinary"),
        ("ordinary_repository_route", "", "push", "ordinary"),
    ):
        result, output, package_started, binding = run_lane(method, path, event)
        assert result.returncode == 0, (method, result.stderr)
        assert f"lane={expected}\n" in output, (method, output)
        assert package_started, method
        if event == "pull_request":
            assert binding["eventBaseRevision"] == base
            assert binding["testedBaseRevision"] == tested_base
            assert binding["testedParents"] == [tested_base, source]

    invalid_bindings = (
        ("wrong PR head", {"event_head": alternate, "context_head": alternate}),
        ("forged base context", {"context_base": tested_base}),
        ("non-ancestor event base", {"event_base": unrelated_base}),
        ("checkout/GITHUB_SHA mismatch", {"github_sha": source}),
        ("one-parent commit", {"checkout_revision": source}),
        ("three-parent commit", {"checkout_revision": three_parent_merge}),
        ("wrong merge second parent", {"checkout_revision": wrong_head_merge}),
    )
    for name, overrides in invalid_bindings:
        result, output, package_started, _ = run_lane(
            "ordinary_repository_route", "", **overrides
        )
        assert result.returncode != 0, (name, output)
        assert not package_started, name

    for method, path, event in (
        ("unexpected_route", archived_path, "pull_request"),
        ("archived_contract_read_only", active_path, "pull_request"),
        ("archived_contract_read_only", archived_path, "push"),
    ):
        result, output, package_started, _ = run_lane(method, path, event)
        assert result.returncode != 0, (method, output)
        assert "lane=" not in output, (method, output)
        assert not package_started, method

assert "name: ci-quality-revision-binding" in workflow
assert "if: always()" in workflow.split(
    "      - name: Upload quality revision binding", 1
)[1].split("      - name:", 1)[0]
hosted_verification = workflow.split(
    "      - name: Run admitted hosted Work Item verification with the candidate Runtime", 1
)[1].split("      - name: Upload hosted Work Item verification evidence", 1)[0]
assert "if: steps.quality_route.outputs.lane == 'active'" in hosted_verification.split("        run: |", 1)[0]
assert "tests/ci/run_hosted_runtime_verification.sh" in hosted_verification
assert "target/release/ai-cockpit" in hosted_verification
assert "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER" in hosted_verification
assert "${{ github.workspace }}/tests/ci/run_process_observer_test_runner.sh" in hosted_verification
assert 'caller_file="$RUNNER_TEMP/wi1113-task-caller.json"' in hosted_verification
assert '"schemaVersion": 1' in hosted_verification and '"actor": "agent:codex"' in hosted_verification
assert '"assurance": "self_declared"' in hosted_verification
assert 'caller_args=(--task-caller "$caller_file")' in hosted_verification
assert '"${caller_args[@]}"' in hosted_verification
caller_match = re.search(r"<<'TASK_CALLER_JSON'\n(.*?)\n\s*TASK_CALLER_JSON", hosted_verification, re.S)
assert caller_match, "WI-1113 caller must be an explicit temporary JSON input"
caller = json.loads(caller_match.group(1))
contract_path = Path(sys.argv[1]).parents[2] / ".ai/work-items/active/WI-1113-delivery-continuity-parallel-authorization.contract.json"
task_authorization = json.loads(contract_path.read_text(encoding="utf-8"))["governanceProfile"]["taskAuthorization"]
assert caller == {
    "schemaVersion": 1,
    "actor": task_authorization["agentActor"],
    "authoritySource": task_authorization["authoritySource"],
    "assurance": "self_declared",
}
quality_permissions = workflow.split("  quality:", 1)[1].split("  windows-runtime-suites:", 1)[0]
assert re.search(r"permissions:\s+contents:\s+read", quality_permissions)
assert "id-token:" not in workflow
assert "target/hosted-runtime-verification.json" in workflow
hosted_runner = Path(sys.argv[3]).with_name("run_hosted_runtime_verification.sh").read_text(encoding="utf-8")
hosted_behavior_test = Path(sys.argv[3]).with_name("hosted_runtime_verification_test.sh").read_text(encoding="utf-8")
cleanup_runner = Path(sys.argv[3]).with_name("cleanup_hosted_runtime_verification_worktree.sh").read_text(encoding="utf-8")
quality_gate = workflow.split(
    "      - name: Evaluate Rust Contract-aware quality gate", 1
)[1].split("      - name: Build legacy Runtime for coordination compatibility", 1)[0]
quality_gate_flat = re.sub(r"\s+", " ", quality_gate)
assert 'ROUTE_LANE: ${{ steps.quality_route.outputs.lane }}' not in quality_gate
assert 'verification_repository' not in quality_gate_flat
assert "run_preflight" in hosted_runner and "run_verification" in hosted_runner
assert "--workers 1" in hosted_runner
assert 'status_allows "$status_after" run_verification' in hosted_runner
assert 'AI_COCKPIT_DEFER_WORKTREE_CLEANUP:-false' in hosted_runner
assert "deferred_for_consumer" in hosted_runner
assert 'formalReceiptDigest' in hosted_runner and 'executionRepository' in hosted_runner
assert "run_helper stale true" in hosted_behavior_test and "run_helper fresh true" in hosted_behavior_test
assert "run_helper stale-deferred true true" in hosted_behavior_test
assert "run_workspace_package_tests.sh" in hosted_behavior_test
assert "tampered-worktree-cleanup.json" in hosted_behavior_test
assert 'git -C "$source_repository" worktree remove --force "$isolated_repository"' in cleanup_runner
assert 'rev-parse --git-common-dir' in cleanup_runner
assert 'source_head" != "$worktree_head' in cleanup_runner
assert workflow.index("name: Run admitted hosted Work Item verification with the candidate Runtime") < workflow.index(
    "name: Evaluate Rust Contract-aware quality gate"
), "refresh and verify with the exact hosted Runtime before the Contract gate"
assert r'cleanup_hosted_runtime_verification_worktree.sh \ --resolve' not in quality_gate_flat
assert '--repo "$GITHUB_WORKSPACE"' in quality_gate_flat
assert 'contract_path="$ROUTE_CONTRACT_PATH"' in quality_gate_flat
assert 'contract_path="$GITHUB_WORKSPACE/$contract_path"' in quality_gate_flat
assert '--contract "$contract_path"' in quality_gate_flat
assert '--report "$GITHUB_WORKSPACE/target/rust-contract-quality-gate.json"' in quality_gate_flat
quality_job = job_block("quality")
assert quality_job.index("name: Run admitted hosted Work Item verification with the candidate Runtime") < quality_job.index(
    "name: run repository gates exactly once"
) < quality_job.index("name: verify workspace package coverage receipt") < quality_job.index(
    "name: Cleanup hosted Runtime verification worktree"
) < quality_job.index("name: Upload hosted Work Item verification evidence"), (
    "hosted receipt production, coverage consumption, cleanup, and evidence upload must remain ordered"
)
repository_gates_step = workflow.split(
    "      - name: run repository gates exactly once", 1
)[1].split("      - name: verify workspace package coverage receipt", 1)[0]
repository_gates_step_flat = re.sub(r"\s+", " ", repository_gates_step)
assert "steps.quality_route.outputs.lane == 'active' && 'target/hosted-runtime-verification.json' || ''" in repository_gates_step
assert "steps.quality_route.outputs.lane == 'active' && 'target/hosted-runtime-orchestration.json' || ''" in repository_gates_step
assert 'ROUTE_LANE: ${{ steps.quality_route.outputs.lane }}' in repository_gates_step
assert 'if [[ "$ROUTE_LANE" == active ]]; then' in repository_gates_step_flat
assert "AI_COCKPIT_VERIFICATION_RECEIPT" in repository_gates_step
assert "AI_COCKPIT_VERIFICATION_ORCHESTRATION" in repository_gates_step
assert "AI_COCKPIT_RUNTIME_BIN: target/release/ai-cockpit" in repository_gates_step
assert "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER" in repository_gates_step
assert "${{ github.workspace }}/tests/ci/run_process_observer_test_runner.sh" in repository_gates_step
assert r'cleanup_hosted_runtime_verification_worktree.sh \ --resolve' in repository_gates_step_flat
assert 'AI_COCKPIT_VERIFICATION_REPOSITORY="$verification_repository"' in repository_gates_step_flat
assert 'run_repository_gates.py --repo "$GITHUB_WORKSPACE"' in repository_gates_step_flat
cleanup_step = workflow.split("      - name: Cleanup hosted Runtime verification worktree", 1)[1].split("      - name:", 1)[0]
upload_hosted = workflow.split("      - name: Upload hosted Work Item verification evidence", 1)[1].split("      - name:", 1)[0]
assert "if: always() && steps.quality_route.outputs.lane == 'active'" in cleanup_step
assert "if: always() && steps.quality_route.outputs.lane == 'active'" in upload_hosted
coverage_runner = Path(sys.argv[3]).with_name("run_workspace_package_tests.sh")
assert "hosted_verification_receipt" in coverage_runner.read_text(encoding="utf-8")

# The route boundary must reject known illegal lifecycle transitions before
# repository gates run, with a stable code and remediation rather than a raw
# traceback or a second copy of the same failure.
assert "lifecycle_transition_invalid" in route
assert "lifecycle_transition_stale" in route
assert "remediation" in route
assert "failure_code" in runner
assert "failureRoots" in runner
assert "--rust-bin" in route
assert "--gate-plan-bin" in runner
assert "ci-gate-plan-tool" in workflow

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
