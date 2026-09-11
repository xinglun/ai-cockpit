#!/usr/bin/env python3
from __future__ import annotations

import copy
import importlib.util
import json
import os
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "tests/ci/quality_route.py"
MANIFEST_PATH = ROOT / "tests/ci/repository_gate_manifest.json"


def load_module():
    spec = importlib.util.spec_from_file_location("quality_route", MODULE_PATH)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


route = load_module()
manifest = route.load_manifest(MANIFEST_PATH)
assert manifest["schemaVersion"] == 2
assert manifest["profileOrder"] == ["light", "standard", "strict"]

empty_covers = copy.deepcopy(manifest)
empty_covers["gates"][0]["covers"] = []
with tempfile.TemporaryDirectory(prefix="ai-cockpit-empty-covers-") as temporary_directory:
    empty_covers_path = Path(temporary_directory) / "manifest.json"
    empty_covers_path.write_text(json.dumps(empty_covers), encoding="utf-8")
    try:
        route.load_manifest(empty_covers_path)
    except ValueError as error:
        assert "non-empty list" in str(error)
    else:
        raise AssertionError("Python route must reject covers: []")

assert route.parse_structured_failure(
    '{"state":"failed","failureCode":"quality_route_failed","remediation":"retry"}'
) == ("quality_route_failed", "retry")


def assert_invalid_gate_order(gates: list[dict], temporary: Path) -> None:
    fixture = copy.deepcopy(manifest)
    fixture["gates"] = gates
    fixture_path = temporary / "invalid-manifest.json"
    fixture_path.write_text(json.dumps(fixture), encoding="utf-8")
    try:
        route.load_manifest(fixture_path)
    except ValueError as error:
        assert str(error) == "gate IDs must be sorted and unique"
    else:
        raise AssertionError("invalid gate IDs must fail before route selection")


with tempfile.TemporaryDirectory(prefix="ai-cockpit-gate-order-") as temporary_directory:
    temporary = Path(temporary_directory)
    out_of_order = copy.deepcopy(manifest["gates"])
    out_of_order[0], out_of_order[1] = out_of_order[1], out_of_order[0]
    assert_invalid_gate_order(out_of_order, temporary)

    duplicate = copy.deepcopy(manifest["gates"])
    duplicate_gate = copy.deepcopy(duplicate[0])
    duplicate_gate["command"] = ["true"]
    duplicate.insert(1, duplicate_gate)
    assert_invalid_gate_order(duplicate, temporary)

    cyclic = copy.deepcopy(manifest)
    cyclic["gates"][0]["dependsOn"] = [cyclic["gates"][1]["id"]]
    cyclic["gates"][1]["dependsOn"] = [cyclic["gates"][0]["id"]]
    cyclic_path = temporary / "cyclic-manifest.json"
    cyclic_path.write_text(json.dumps(cyclic), encoding="utf-8")
    try:
        route.load_manifest(cyclic_path)
    except ValueError as error:
        assert "acyclic" in str(error)
    else:
        raise AssertionError("cyclic gate dependencies must fail before route selection")


def selected(paths: list[str], *, risk: str = "normal", stage: str = "pull_request") -> str:
    return route.select_route(
        manifest,
        paths=paths,
        risk=risk,
        stage=stage,
        requested_profile=None,
    )["selectedProfile"]


assert selected(["docs/release/distribution.md"]) == "light"
assert selected(["crates/cockpit-cli/src/main.rs"]) == "standard"
assert selected([".github/workflows/release.yml"]) == "strict"
assert selected(["tests/release/adopter_acceptance.sh"]) == "strict"
assert selected(["docs/release/distribution.md", "Cargo.lock"]) == "strict"
assert selected(["unclassified/new-surface.xyz"]) == "strict"
assert selected(["docs/release/distribution.md"], risk="high") == "strict"
assert selected(["docs/release/distribution.md"], stage="release") == "strict"

closure_manifest = copy.deepcopy(manifest)
closure_manifest["gates"] = [
    {
        "category": "fixture",
        "command": ["true"],
        "id": "fixture_a_strict",
        "minimumProfile": "strict",
    },
    {
        "category": "fixture",
        "command": ["false"],
        "dependsOn": ["fixture_a_strict"],
        "id": "fixture_b_standard",
        "minimumProfile": "standard",
    },
]
closure = route.select_route(
    closure_manifest,
    paths=["crates/example.rs"],
    risk="normal",
    stage="pull_request",
    requested_profile=None,
)
assert closure["automaticProfile"] == "standard"
assert closure["selectedProfile"] == "strict"
assert closure["requiredGateIds"] == ["fixture_a_strict", "fixture_b_standard"]

automatic = route.select_route(
    manifest,
    paths=["Cargo.lock"],
    risk="normal",
    stage="pull_request",
    requested_profile=None,
)
try:
    route.select_route(
        manifest,
        paths=["Cargo.lock"],
        risk="normal",
        stage="pull_request",
        requested_profile="light",
    )
except ValueError as error:
    assert "lower" in str(error) or "downgrade" in str(error)
else:
    raise AssertionError("an explicit profile must not lower the automatic route")

strict_gate_ids = [
    gate["id"]
    for gate in manifest["gates"]
    if route.profile_includes(manifest, "strict", gate["minimumProfile"])
]
assert automatic["requiredGateIds"] == strict_gate_ids
assert "workspace_package_tests" in strict_gate_ids
assert "release_adopter" in strict_gate_ids
strict_order = automatic["executionOrder"]
assert strict_order.index("release_action_runtime") < strict_order.index("performance_p0_regression")
assert strict_order.index("performance_p0_regression") < strict_order.index("workspace_clippy")
for profile in ("light", "standard", "strict"):
    profile_gate_ids = [
        gate["id"]
        for gate in manifest["gates"]
        if route.profile_includes(manifest, profile, gate["minimumProfile"])
    ]
    assert "docs_pending_parity_registry_regression" in profile_gate_ids

with tempfile.TemporaryDirectory(prefix="ai-cockpit-quality-route-") as temporary:
    repository = Path(temporary)
    subprocess.run(["git", "init", "-q", str(repository)], check=True)
    subprocess.run(
        ["git", "-C", str(repository), "config", "user.name", "Quality Route Test"],
        check=True,
    )
    subprocess.run(
        ["git", "-C", str(repository), "config", "user.email", "route@example.invalid"],
        check=True,
    )
    (repository / "docs").mkdir()
    (repository / "docs/readme.md").write_text("before\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(repository), "add", "."], check=True)
    subprocess.run(["git", "-C", str(repository), "commit", "-qm", "base"], check=True)
    base = subprocess.check_output(
        ["git", "-C", str(repository), "rev-parse", "HEAD"], text=True
    ).strip()
    (repository / "docs/readme.md").write_text("after\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(repository), "add", "."], check=True)
    subprocess.run(["git", "-C", str(repository), "commit", "-qm", "head"], check=True)
    head = subprocess.check_output(
        ["git", "-C", str(repository), "rev-parse", "HEAD"], text=True
    ).strip()
    receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=MANIFEST_PATH,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile=None,
    )
    assert receipt["selectedProfile"] == "light"
    assert receipt["changedPaths"] == ["docs/readme.md"]
    assert receipt["manifestDigest"].startswith("sha256:")
    route.validate_route_receipt(
        receipt,
        repository=repository,
        manifest_path=MANIFEST_PATH,
    )

    tampered = copy.deepcopy(receipt)
    tampered["requiredGateIds"].append("release_adopter")
    try:
        route.validate_route_receipt(
            tampered,
            repository=repository,
            manifest_path=MANIFEST_PATH,
        )
    except ValueError as error:
        assert "gate" in str(error).lower() or "receipt" in str(error).lower()
    else:
        raise AssertionError("tampered required gate IDs must fail closed")

    tampered = copy.deepcopy(receipt)
    tampered["manifestDigest"] = "sha256:" + ("0" * 64)
    try:
        route.validate_route_receipt(
            tampered,
            repository=repository,
            manifest_path=MANIFEST_PATH,
        )
    except ValueError as error:
        assert "manifest" in str(error).lower()
    else:
        raise AssertionError("a foreign manifest digest must fail closed")

    runtime_reuse = repository / ".ai/evidence/reuse"
    runtime_reuse.mkdir(parents=True)
    (runtime_reuse / "index.json").write_text("{}\n", encoding="utf-8")
    try:
        route.validate_route_receipt(
            receipt,
            repository=repository,
            manifest_path=MANIFEST_PATH,
        )
    except ValueError as error:
        assert "repository facts" in str(error)
    else:
        raise AssertionError("a pre-shadow route receipt must be stale after Runtime writes")
    final_receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=MANIFEST_PATH,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile=None,
    )
    assert final_receipt["selectedProfile"] == "strict"
    assert ".ai/evidence/reuse/index.json" in final_receipt["changedPaths"]
    route.validate_route_receipt(
        final_receipt,
        repository=repository,
        manifest_path=MANIFEST_PATH,
    )

ci_workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
release_workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
assert "tests/ci/quality_route.py" in ci_workflow
for workflow in (ci_workflow,):
    assert "tests/ci/quality_route.py" in workflow
    assert "--route-receipt" in workflow
    assert "tests/ci/repository_gate_manifest.json" in workflow
    assert "--command" not in workflow
assert "stage=pull_request" in ci_workflow
assert '--stage "$stage"' in ci_workflow
assert 'contract_base_revision="$(jq -er' in ci_workflow
assert 'base_revision="$(git rev-parse "${contract_base_revision}^{commit}")"' in ci_workflow
assert "  push:\n    branches:\n      - main" in ci_workflow
assert "--stage release" in release_workflow
assert "--profile strict" in release_workflow
assert "target/quality-route.json" in ci_workflow
assert "gate" in ci_workflow
assert "--runner hosted" in ci_workflow
assert "target/rust-contract-quality-gate.json" in ci_workflow
assert "--contract-gate-report" in ci_workflow
assert ci_workflow.count("--route-receipt target/quality-route.json") == 1
assert "target/release-quality-route.json" in release_workflow
assert "target/release/ai-cockpit gate-plan" in release_workflow
assert "python3 tests/ci/quality_route.py" not in release_workflow
assert "contracts=()" in release_workflow
assert "if [[ -d .ai/work-items/active ]]; then" in release_workflow
assert "manual to_tag does not match staged candidate identity" in release_workflow
assert "name: workspace-package-coverage" in ci_workflow
assert "name: Bind the shared typed repository quality route" in ci_workflow
assert "if: steps.quality_route.outputs.profile != 'light'" in ci_workflow
assert (
    "if: steps.quality_route.outputs.profile != 'light' && "
    "steps.quality_route.outputs.contract_path != ''"
) in ci_workflow
assert ci_workflow.count(
    "steps.quality_route.outputs.profile != 'light' && "
    "steps.quality_route.outputs.contract_path != ''"
) == 4
assert "name: Plan the initial typed repository quality route" not in ci_workflow
assert "name: Finalize the typed repository quality route" not in ci_workflow
assert "target/quality-route-initial.json" not in ci_workflow
assert "name: verify workspace package coverage receipt" in ci_workflow
assert (
    "if: always() && steps.quality_route.outputs.profile != 'light' && "
    "hashFiles('target/workspace-package-coverage.json') != ''"
) in ci_workflow
bound_route = ci_workflow.index("name: Bind the shared typed repository quality route")
runtime_shadow = ci_workflow.index("name: verify immutable Runtime shadow")
rust_gate = ci_workflow.index("name: Evaluate Rust Contract-aware quality gate")
gate_execution = ci_workflow.index("name: run repository gates exactly once")
assert bound_route < runtime_shadow < rust_gate < gate_execution
assert "target/release/ai-cockpit" in ci_workflow
assert "--rust-bin target/ai-cockpit" not in ci_workflow
assert "--rust-bin target/release/ai-cockpit" in ci_workflow
assert "name: ci-gate-plan-tool" in ci_workflow
assert "--gate-plan-bin target/release/ai-cockpit" in ci_workflow

for relative in (
    "docs/reference/ci-runtime-shadow.md",
    "docs/reference/ci-runtime-shadow.zh-CN.md",
    "docs/reference/ci-runtime-shadow.ja.md",
    "docs/release/distribution.md",
    "docs/release/distribution.zh-CN.md",
    "docs/release/distribution.ja.md",
):
    documentation = (ROOT / relative).read_text(encoding="utf-8")
    for fact in ("light", "standard", "strict", "v0.2.28", "--command", "deferred"):
        assert fact in documentation, f"{relative} is missing route boundary fact: {fact}"
for relative in (
    "docs/release/distribution.md",
    "docs/release/distribution.zh-CN.md",
    "docs/release/distribution.ja.md",
):
    documentation = (ROOT / relative).read_text(encoding="utf-8")
    for fact in ("Makefile", ".gitattributes", "staged_adopter_acceptance"):
        assert fact in documentation, f"{relative} is missing release parity fact: {fact}"

# The CI boundary must reject an impossible lifecycle projection before it
# starts any repository gate. This catches the same stale/unnormalized state
# that previously appeared only after a hosted run had already started.
with tempfile.TemporaryDirectory(prefix="ai-cockpit-lifecycle-boundary-") as temporary:
    repository = Path(temporary)
    subprocess.run(["git", "init", "-q", str(repository)], check=True)
    subprocess.run(["git", "-C", str(repository), "config", "user.name", "Lifecycle Route Test"], check=True)
    subprocess.run(["git", "-C", str(repository), "config", "user.email", "lifecycle@example.invalid"], check=True)
    (repository / "README.md").write_text("base\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(repository), "add", "README.md"], check=True)
    subprocess.run(["git", "-C", str(repository), "commit", "-qm", "base"], check=True)
    base = subprocess.check_output(["git", "-C", str(repository), "rev-parse", "HEAD"], text=True).strip()
    (repository / ".ai/work-items/active").mkdir(parents=True)
    contract_path = repository / ".ai/work-items/active/WI-LIFECYCLE.contract.json"
    contract_path.write_text(json.dumps({"risk": "normal"}), encoding="utf-8")
    summary_path = repository / ".ai/work-items/active/WI-LIFECYCLE.summary.json"
    summary_path.write_text(
        json.dumps({"state": "checkpointed", "checkpointCount": 0, "preflightState": "yellow"}),
        encoding="utf-8",
    )
    try:
        route.plan_repository_route(
            repository=repository,
            manifest_path=MANIFEST_PATH,
            base=base,
            head=base,
            stage="pull_request",
            risk="normal",
            contract_path=Path(".ai/work-items/active/WI-LIFECYCLE.contract.json"),
            requested_profile=None,
        )
    except ValueError as error:
        assert "lifecycle_transition_invalid" in str(error)
    else:
        raise AssertionError("checkpointed lifecycle with zero checkpoints must fail before gates")

    summary_path.write_text(
        json.dumps({
            "state": "implementation_active",
            "checkpointCount": 0,
            "preflightState": "yellow",
            "failedGate": "finish.lifecycle",
        }),
        encoding="utf-8",
    )
    try:
        route.plan_repository_route(
            repository=repository,
            manifest_path=MANIFEST_PATH,
            base=base,
            head=base,
            stage="pull_request",
            risk="normal",
            contract_path=Path(".ai/work-items/active/WI-LIFECYCLE.contract.json"),
            requested_profile=None,
        )
    except ValueError as error:
        assert "lifecycle_transition_stale" in str(error)
    else:
        raise AssertionError("a failed lifecycle transition must fail before gates")

    summary_path.unlink()
    os.symlink(repository / "README.md", summary_path)
    try:
        route.plan_repository_route(
            repository=repository,
            manifest_path=MANIFEST_PATH,
            base=base,
            head=base,
            stage="pull_request",
            risk="normal",
            contract_path=Path(".ai/work-items/active/WI-LIFECYCLE.contract.json"),
            requested_profile=None,
        )
    except route.RouteValidationError as error:
        assert error.code == "lifecycle_transition_invalid"
        assert error.remediation == (
            "restore the repository-local Summary and rerun preflight before pushing"
        )
    else:
        raise AssertionError("a Summary symlink must fail before gates")

print("repository quality route regression passed")
