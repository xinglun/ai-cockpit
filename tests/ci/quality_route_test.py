#!/usr/bin/env python3
from __future__ import annotations

import copy
import importlib.util
import json
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

ci_workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
release_workflow = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
for workflow in (ci_workflow, release_workflow):
    assert "tests/ci/quality_route.py" in workflow
    assert "--route-receipt" in workflow
    assert "tests/ci/repository_gate_manifest.json" in workflow
    assert "--command" not in workflow
assert "stage=pull_request" in ci_workflow
assert '--stage "$stage"' in ci_workflow
assert "--stage release" in release_workflow
assert "--profile strict" in release_workflow
assert "target/quality-route.json" in ci_workflow
assert "target/release-quality-route.json" in release_workflow
assert "manual to_tag does not match staged candidate identity" in release_workflow
assert "name: workspace-package-coverage" in ci_workflow
assert "if: always() && steps.quality_route.outputs.profile != 'light'" in ci_workflow

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

print("repository quality route regression passed")
