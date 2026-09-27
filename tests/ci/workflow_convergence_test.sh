#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
workflow="$root/.github/workflows/ci.yml"
route="$root/tests/ci/quality_route.py"
runner="$root/tests/ci/run_repository_gates.py"
resolver="$root/tests/ci/resolve_work_item.sh"

python3 - "$workflow" "$route" "$runner" "$resolver" <<'PY'
from pathlib import Path
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

def job_block(name):
    marker = f"  {name}:\n"
    start = workflow.index(marker, workflow.index("jobs:\n"))
    remaining = workflow[start + len(marker):]
    next_job = re.search(r"(?m)^  [A-Za-z0-9_-]+:\s*$", remaining)
    return workflow[start:] if next_job is None else workflow[start:start + len(marker) + next_job.start()]

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

# Independent hosted validations fan out from the immutable route artifact;
# no validation job waits for another validation job.
validation_jobs = ("quality", "windows-runtime", "v1-behavioral-oracle")
for name in validation_jobs:
    block = job_block(name)
    assert "\n    needs: route\n" in block, f"{name} must depend on the shared route only"
    assert "needs: [" not in block, f"{name} must not serialize behind another validation job"
assert "\n    needs: quality\n" not in workflow, "independent CI validation must not wait for quality"

ordinary_guide = Path(sys.argv[1]).parents[2] / "agents/skills/ordinary-work-item.md"
ordinary_guide_text = re.sub(r"\s+", " ", ordinary_guide.read_text(encoding="utf-8").lower())
for required_rule in (
    "keep lifecycle and snapshot-changing writes serial",
    "independent checks may run in parallel on a fixed input snapshot",
    "dependencies are ready",
    "outputs are isolated",
    "resource limits allow it",
    "reuse a fresh producer receipt",
):
    assert required_rule in ordinary_guide_text, f"ordinary guide omits parallel boundary: {required_rule}"
assert 'merge_parents[2]' in workflow
windows_job = workflow[workflow.index("  windows-runtime:"):]
windows_checkout = windows_job.split("      - name: Bind source and tested revisions", 1)[0]
assert "fetch-depth: 0" in windows_checkout
assert "runtime_preflight_waits_for_lifecycle_lock_and_rechecks_admission" in windows_job
assert "runtime_controls_wait_for_lifecycle_lock_and_recheck_admission_before_receipt_write" in windows_job
windows_composition_step = windows_job.split(
    "      - name: verify Windows process and composition lifecycle boundaries", 1
)[1].split("      - name:", 1)[0]
assert "cargo test --locked -p cockpit-verification --lib --test execution --test composition" in windows_composition_step
windows_upload_step = windows_job.split(
    "      - name: upload Windows revision binding", 1
)[1].split("      - name:", 1)[0]
assert "if: always()" in windows_upload_step
windows_binding = windows_job.split("      - name: Bind source and tested revisions", 1)[1].split(
    "      - uses: dtolnay/rust-toolchain", 1
)[0]
assert windows_binding.index("Set-Content -Encoding utf8 target/ci-revision-binding.json") < windows_binding.index(
    "throw 'tested PR merge commit does not bind"
), "write Windows revision diagnostics before lineage assertions"
quality_binding = workflow.split(
    "      - name: Bind source and tested revisions; validate the shared typed quality route", 1
)[1].split("      - name: verify immutable Runtime shadow", 1)[0]
assert quality_binding.index("> target/ci-revision-binding.json") < quality_binding.index(
    'test "$receipt_head" = "$source_revision"'
), "write revision diagnostics before any source/route binding assertion"
assert "name: ci-quality-revision-binding" in workflow
assert "if: always()" in workflow.split(
    "      - name: Upload quality revision binding", 1
)[1].split("      - name:", 1)[0]
hosted_verification = workflow.split(
    "      - name: Run admitted hosted Work Item verification with the candidate Runtime", 1
)[1].split("      - name: Upload hosted Work Item verification evidence", 1)[0]
assert "tests/ci/run_hosted_runtime_verification.sh" in hosted_verification
assert "target/release/ai-cockpit" in hosted_verification
assert "target/hosted-runtime-verification.json" in workflow
hosted_runner = Path(sys.argv[3]).with_name("run_hosted_runtime_verification.sh").read_text(encoding="utf-8")
hosted_behavior_test = Path(sys.argv[3]).with_name("hosted_runtime_verification_test.sh").read_text(encoding="utf-8")
assert "run_preflight" in hosted_runner and "run_verification" in hosted_runner
assert "--workers 2" in hosted_runner
assert 'status_allows "$status_after" run_verification' in hosted_runner
assert "run_helper stale true" in hosted_behavior_test and "run_helper fresh true" in hosted_behavior_test
assert workflow.index("name: Run admitted hosted Work Item verification with the candidate Runtime") < workflow.index(
    "name: Evaluate Rust Contract-aware quality gate"
), "refresh and verify with the exact hosted Runtime before the Contract gate"
quality_job = job_block("quality")
assert quality_job.index("name: Run admitted hosted Work Item verification with the candidate Runtime") < quality_job.index(
    "name: run repository gates exactly once"
) < quality_job.index("name: verify workspace package coverage receipt"), (
    "hosted receipt production, package gates, and receipt consumption must remain ordered"
)
repository_gates_step = workflow.split(
    "      - name: run repository gates exactly once", 1
)[1].split("      - name: verify workspace package coverage receipt", 1)[0]
assert "AI_COCKPIT_VERIFICATION_RECEIPT" in repository_gates_step
assert "AI_COCKPIT_RUNTIME_BIN: target/release/ai-cockpit" in repository_gates_step
assert "AI_COCKPIT_VERIFICATION_REPOSITORY" in repository_gates_step
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
