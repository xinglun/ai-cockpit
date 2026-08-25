#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path


root = Path(__file__).resolve().parents[2]
manifest_path = root / "tests/ci/repository_gate_manifest.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
assert manifest["schemaVersion"] == 2
assert manifest["profileOrder"] == ["light", "standard", "strict"]
entries = manifest["gates"]
ids = [entry["id"] for entry in entries]
commands = [tuple(entry["command"]) for entry in entries]
assert ids == sorted(ids), "gate IDs must be deterministic"
assert len(ids) == len(set(ids)), "duplicate gate ID"
assert len(commands) == len(set(commands)), "duplicate gate command"
assert all(entry["minimumProfile"] in manifest["profileOrder"] for entry in entries)

registered = {entry["command"][0] for entry in entries}
registered.update(
    entry["command"][1]
    for entry in entries
    if entry["command"][0] in {"python", "python3"} and len(entry["command"]) > 1
)
registered.update(path for entry in entries for path in entry.get("covers", []))
required = {
    str(path.relative_to(root))
    for path in (root / "tests").rglob("*_test.sh")
}
required.update(
    {
        "tests/docs/documentation_acceptance.sh",
        "tests/release/annotated_tag_identity.sh",
        "tests/release/workflow_policy.sh",
    }
)
missing = sorted(required - registered)
assert not missing, f"unregistered repository gates: {missing}"

# List-only inspection is allowed an explicit profile, but never an arbitrary
# command. The report still binds the canonical manifest and selected gate IDs.
listed_report = root / "target/repository-gate-manifest-test.json"
subprocess.run(
    [
        sys.executable,
        "tests/ci/run_repository_gates.py",
        "--repo",
        str(root),
        "--manifest",
        str(manifest_path),
        "--profile",
        "strict",
        "--report",
        str(listed_report),
        "--list-only",
    ],
    cwd=root,
    check=True,
)
listed = json.loads(listed_report.read_text(encoding="utf-8"))
assert listed["state"] == "listed"
assert listed["route"]["selectedProfile"] == "strict"
assert listed["route"]["requiredGateIds"] == ids
assert listed["route"]["manifestDigest"].startswith("sha256:")

override = subprocess.run(
    [
        sys.executable,
        "tests/ci/run_repository_gates.py",
        "--repo",
        str(root),
        "--manifest",
        str(manifest_path),
        "--profile",
        "light",
        "--report",
        str(root / "target/forbidden-command.json"),
        "--command",
        "true",
    ],
    cwd=root,
    check=False,
    capture_output=True,
    text=True,
)
assert override.returncode != 0
assert "unrecognized arguments" in override.stderr

# Exercise a real receipt against a small repository. The runner may execute
# only the command stored in the manifest whose digest the receipt binds.
spec = importlib.util.spec_from_file_location("quality_route", root / "tests/ci/quality_route.py")
assert spec is not None and spec.loader is not None
route = importlib.util.module_from_spec(spec)
spec.loader.exec_module(route)
with tempfile.TemporaryDirectory(prefix="ai-cockpit-gate-runner-") as temporary:
    fixture = Path(temporary)
    repository = fixture / "repo"
    repository.mkdir()
    subprocess.run(["git", "init", "-q", str(repository)], check=True)
    subprocess.run(["git", "-C", str(repository), "config", "user.name", "Gate Runner Test"], check=True)
    subprocess.run(["git", "-C", str(repository), "config", "user.email", "gate@example.invalid"], check=True)
    (repository / "docs").mkdir()
    (repository / "docs/readme.md").write_text("before\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(repository), "add", "."], check=True)
    subprocess.run(["git", "-C", str(repository), "commit", "-qm", "base"], check=True)
    base = subprocess.check_output(["git", "-C", str(repository), "rev-parse", "HEAD"], text=True).strip()
    (repository / "docs/readme.md").write_text("after\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(repository), "add", "."], check=True)
    subprocess.run(["git", "-C", str(repository), "commit", "-qm", "head"], check=True)
    head = subprocess.check_output(["git", "-C", str(repository), "rev-parse", "HEAD"], text=True).strip()
    fixture_manifest = fixture / "manifest.json"
    fixture_manifest.write_text(
        json.dumps(
            {
                "schemaVersion": 2,
                "profileOrder": ["light", "standard", "strict"],
                "unknownProfile": "strict",
                "pathProfiles": {
                    "light": ["docs/**"],
                    "standard": ["src/**"],
                    "strict": [".github/**"],
                },
                "releaseOwnedPatterns": ["release/**"],
                "stageFloors": {
                    "task": "light",
                    "pre_ci": "light",
                    "pull_request": "light",
                    "merge": "strict",
                    "release": "strict",
                },
                "gates": [
                    {
                        "category": "fixture",
                        "command": ["true"],
                        "id": "fixture_true",
                        "minimumProfile": "light",
                    }
                ],
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    receipt_path = fixture / "route.json"
    receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=fixture_manifest,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile=None,
    )
    receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    report_path = fixture / "report.json"
    subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(fixture_manifest),
            "--route-receipt",
            str(receipt_path),
            "--report",
            str(report_path),
        ],
        check=True,
    )
    report = json.loads(report_path.read_text(encoding="utf-8"))
    assert report["state"] == "passed"
    assert report["route"]["receiptDigest"] == receipt["receiptDigest"]
    assert [gate["id"] for gate in report["gates"]] == ["fixture_true"]

    # A non-light Contract route must carry a green Rust Contract gate report
    # bound to the same Contract file, base revision, repository identity, and
    # provider stage before command execution is accepted.
    (repository / ".ai/work-items/active").mkdir(parents=True)
    (repository / ".ai/cockpit.toml").write_text(
        'protocol_version = 1\nrepository_schema_version = 2\n'
        'repository_id = "sha256:' + "1" * 64 + '"\n',
        encoding="utf-8",
    )
    contract_file = repository / ".ai/work-items/active/WI-CI-FIX.contract.json"
    contract_file.write_text('{"risk":"normal"}\n', encoding="utf-8")
    contract_receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=fixture_manifest,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=Path(".ai/work-items/active/WI-CI-FIX.contract.json"),
        requested_profile=None,
    )
    contract_route_path = fixture / "contract-route.json"
    contract_route_path.write_text(
        json.dumps(contract_receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    contract_gate_path = fixture / "contract-gate.json"
    contract_gate_path.write_text(
        json.dumps(
            {
                "schemaVersion": 1,
                "kind": "repository_contract_quality_gate",
                "state": "passed",
                "repositoryId": "sha256:" + "1" * 64,
                "workItemId": "WI-CI-FIX",
                "contractDigest": "sha256:" + "2" * 64,
                "contractFileDigest": contract_receipt["contractDigest"],
                "repositorySnapshotDigest": "sha256:" + "3" * 64,
                "baseRevision": base,
                "headRevision": head,
                "changedPaths": [],
                "stage": "pr",
                "runner": "hosted",
                "operation": "modify_source",
                "verificationTier": "T2",
                "evidenceAssurance": "provider_verified",
                "dependencyConfidence": "unknown",
                "decisionState": "green",
                "blockers": [],
                "unknowns": [],
                "requiredChecks": [],
                "runtimeVersion": "test",
                "runtimeDigest": "sha256:" + "4" * 64,
                "receiptDigest": "sha256:" + "5" * 64,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    contract_report_path = fixture / "contract-report.json"
    subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(fixture_manifest),
            "--route-receipt",
            str(contract_route_path),
            "--contract-gate-report",
            str(contract_gate_path),
            "--report",
            str(contract_report_path),
        ],
        check=True,
    )
    assert json.loads(contract_report_path.read_text(encoding="utf-8"))["state"] == "passed"
    blocked_gate = json.loads(contract_gate_path.read_text(encoding="utf-8"))
    blocked_gate["state"] = "blocked"
    contract_gate_path.write_text(json.dumps(blocked_gate), encoding="utf-8")
    rejected_contract_gate = subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(fixture_manifest),
            "--route-receipt",
            str(contract_route_path),
            "--contract-gate-report",
            str(contract_gate_path),
            "--report",
            str(fixture / "blocked-contract-report.json"),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert rejected_contract_gate.returncode != 0
    assert "Contract gate" in rejected_contract_gate.stderr

    tampered = dict(receipt)
    tampered["requiredGateIds"] = ["foreign_gate"]
    tampered_path = fixture / "tampered.json"
    tampered_path.write_text(json.dumps(tampered), encoding="utf-8")
    rejected = subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(fixture_manifest),
            "--route-receipt",
            str(tampered_path),
            "--report",
            str(fixture / "tampered-report.json"),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert rejected.returncode != 0
    assert "receipt" in rejected.stderr.lower() or "gate" in rejected.stderr.lower()

    def run_single_gate(command: list[str], gate_id: str, report_name: str):
        fixture_manifest.write_text(
            json.dumps(
                {
                    "schemaVersion": 2,
                    "profileOrder": ["light", "standard", "strict"],
                    "unknownProfile": "strict",
                    "pathProfiles": {
                        "light": ["docs/**"],
                        "standard": ["src/**"],
                        "strict": [".github/**"],
                    },
                    "releaseOwnedPatterns": ["release/**"],
                    "stageFloors": {
                        "task": "light",
                        "pre_ci": "light",
                        "pull_request": "light",
                        "merge": "strict",
                        "release": "strict",
                    },
                    "gates": [
                        {
                            "category": "fixture",
                            "command": command,
                            "id": gate_id,
                            "minimumProfile": "light",
                        }
                    ],
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        planned = route.plan_repository_route(
            repository=repository,
            manifest_path=fixture_manifest,
            base=base,
            head=head,
            stage="pull_request",
            risk="normal",
            contract_path=None,
            requested_profile=None,
        )
        receipt_path.write_text(json.dumps(planned, sort_keys=True), encoding="utf-8")
        single_report = fixture / report_name
        completed = subprocess.run(
            [
                sys.executable,
                str(root / "tests/ci/run_repository_gates.py"),
                "--repo",
                str(repository),
                "--manifest",
                str(fixture_manifest),
                "--route-receipt",
                str(receipt_path),
                "--report",
                str(single_report),
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        return completed, json.loads(single_report.read_text(encoding="utf-8"))

    missing_run, missing_report = run_single_gate(
        ["tests/missing-gate"], "fixture_missing", "missing-report.json"
    )
    assert missing_run.returncode == 1
    assert "Traceback" not in missing_run.stderr
    assert missing_report["state"] == "failed"
    assert missing_report["gates"][0]["state"] == "failed"
    assert "launchError" in missing_report["gates"][0]

    nonexec = fixture / "non-executable-gate.sh"
    nonexec.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    nonexec.chmod(0o644)
    nonexec_run, nonexec_report = run_single_gate(
        [str(nonexec)], "fixture_nonexec", "nonexec-report.json"
    )
    assert nonexec_run.returncode == 0
    assert nonexec_report["state"] == "passed"

print("repository gate manifest regression passed")
