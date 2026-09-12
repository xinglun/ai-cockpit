#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import signal
import shlex
import subprocess
import sys
import tempfile
import time
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
workspace_clippy = next(entry for entry in entries if entry["id"] == "workspace_clippy")
workspace_format = next(entry for entry in entries if entry["id"] == "workspace_format")
workspace_tests = next(entry for entry in entries if entry["id"] == "workspace_package_tests")
assert "docs_governance_integrity" in workspace_clippy["dependsOn"]
assert workspace_format["dependsOn"] == workspace_clippy["dependsOn"]
assert workspace_tests["dependsOn"] == ["workspace_clippy", "workspace_format"]
performance_regression = next(entry for entry in entries if entry["id"] == "performance_regression")
assert performance_regression["dependsOn"] == [
    "docs_governance_integrity",
    "performance_p0_regression",
]
release_adopter = next(entry for entry in entries if entry["id"] == "release_adopter")
assert "release_version_consistency" in release_adopter["dependsOn"]

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
runner_spec = importlib.util.spec_from_file_location(
    "repository_gate_runner", root / "tests/ci/run_repository_gates.py"
)
assert runner_spec is not None and runner_spec.loader is not None
runner = importlib.util.module_from_spec(runner_spec)
runner_spec.loader.exec_module(runner)
assert runner.failure_code(
    "workspace_package_tests",
    detail="test oversized_reference_inventory_uses_its_strict_conformance_gate ... FAILED",
) == "quality_gate_failed:workspace_package_tests"
assert runner.failure_code(
    "conformance_reference_file_inventory",
    detail="reference inventory mismatch",
) == "reference_inventory_mismatch"
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
    contract_file.write_text(
        json.dumps({"risk": "normal", "baseRevision": base}) + "\n", encoding="utf-8"
    )
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
                "comparisonBaseRevision": base,
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
    blocked_report = json.loads(
        (fixture / "blocked-contract-report.json").read_text(encoding="utf-8")
    )
    assert "Contract gate" in blocked_report["failureMessage"]
    assert blocked_report["failurePhase"] == "preflight"
    assert blocked_report["launchedGateIds"] == []

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
    tampered_report = json.loads(
        (fixture / "tampered-report.json").read_text(encoding="utf-8")
    )
    assert tampered_report["failurePhase"] == "preflight"
    assert tampered_report["launchedGateIds"] == []

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

    dependency_manifest = fixture / "dependency-manifest.json"
    dependency_manifest.write_text(
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
                        "command": ["sh", "-c", "exit 99"],
                        "dependsOn": ["fixture_b_failure"],
                        "id": "fixture_a_dependent",
                        "minimumProfile": "light",
                    },
                    {
                        "category": "fixture",
                        "command": ["false"],
                        "id": "fixture_b_failure",
                        "minimumProfile": "light",
                    },
                ],
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    route.load_manifest(dependency_manifest)
    dependency_receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=dependency_manifest,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile=None,
    )
    dependency_receipt_path = fixture / "dependency-route.json"
    dependency_receipt_path.write_text(
        json.dumps(dependency_receipt, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    dependency_report_path = fixture / "dependency-report.json"
    dependency_run = subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(dependency_manifest),
            "--route-receipt",
            str(dependency_receipt_path),
            "--report",
            str(dependency_report_path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert dependency_run.returncode == 1
    dependency_report = json.loads(dependency_report_path.read_text(encoding="utf-8"))
    assert dependency_report["state"] == "failed"
    assert [gate["id"] for gate in dependency_report["gates"]] == [
        "fixture_b_failure",
        "fixture_a_dependent",
    ]
    assert dependency_report["gates"][0]["state"] == "failed"
    assert dependency_report["gates"][1]["state"] == "blocked"
    assert dependency_report["gates"][1]["blockedBy"] == ["fixture_b_failure"]
    assert dependency_report["gates"][1]["failureCode"] == "prerequisite_failed"

    profile_conditional_manifest = fixture / "profile-conditional-manifest.json"
    profile_conditional = json.loads(dependency_manifest.read_text(encoding="utf-8"))
    profile_conditional["unknownProfile"] = "standard"
    profile_conditional["gates"][0]["dependsOn"] = []
    profile_conditional["gates"][0]["minimumProfile"] = "strict"
    profile_conditional["gates"][1]["command"] = ["true"]
    profile_conditional["gates"][1]["minimumProfile"] = "standard"
    profile_conditional["gates"][1]["dependsOn"] = ["fixture_a_dependent"]
    profile_conditional_manifest.write_text(
        json.dumps(profile_conditional, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    profile_conditional = route.load_manifest(profile_conditional_manifest)
    assert profile_conditional["gates"][0]["minimumProfile"] == "strict"
    conditional_receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=profile_conditional_manifest,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile="standard",
    )
    conditional_receipt_path = fixture / "profile-conditional-route.json"
    conditional_receipt_path.write_text(
        json.dumps(conditional_receipt, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    conditional_report_path = fixture / "profile-conditional-report.json"
    conditional_run = subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(profile_conditional_manifest),
            "--route-receipt",
            str(conditional_receipt_path),
            "--report",
            str(conditional_report_path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert conditional_receipt["selectedProfile"] == "strict"
    assert conditional_run.returncode == 1
    conditional_report = json.loads(conditional_report_path.read_text(encoding="utf-8"))
    assert [gate["id"] for gate in conditional_report["gates"]] == [
        "fixture_a_dependent",
        "fixture_b_failure",
    ]
    assert conditional_report["gates"][0]["state"] == "failed"
    assert conditional_report["gates"][1]["state"] == "blocked"

    resume_manifest = fixture / "resume-manifest.json"
    marker_a = fixture / "resume-a.marker"
    marker_b = fixture / "resume-b.marker"
    marker_a_arg = shlex.quote(str(marker_a))
    marker_b_arg = shlex.quote(str(marker_b))
    resume_manifest.write_text(
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
                        "command": [
                            "sh",
                            "-c",
                            f"test ! -e {marker_a_arg} && touch {marker_a_arg}",
                        ],
                        "id": "fixture_resume_a_passed",
                        "minimumProfile": "light",
                    },
                    {
                        "category": "fixture",
                        "command": ["sh", "-c", f"test -e {marker_b_arg}"],
                        "dependsOn": ["fixture_resume_a_passed"],
                        "id": "fixture_resume_b_repairable",
                        "minimumProfile": "light",
                    },
                    {
                        "category": "fixture",
                        "command": ["true"],
                        "dependsOn": ["fixture_resume_b_repairable"],
                        "id": "fixture_resume_c_dependent",
                        "minimumProfile": "light",
                    },
                ],
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    route.load_manifest(resume_manifest)
    resume_receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=resume_manifest,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile=None,
    )
    resume_receipt_path = fixture / "resume-route.json"
    resume_receipt_path.write_text(
        json.dumps(resume_receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    first_resume_report_path = fixture / "resume-first-report.json"
    first_resume = subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(resume_manifest),
            "--route-receipt",
            str(resume_receipt_path),
            "--report",
            str(first_resume_report_path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert first_resume.returncode == 1
    first_resume_report = json.loads(first_resume_report_path.read_text(encoding="utf-8"))
    assert first_resume_report["gates"][0]["state"] == "passed"
    assert first_resume_report["gates"][1]["state"] == "failed"
    assert first_resume_report["gates"][2]["state"] == "blocked"
    marker_b.touch()
    resumed_report_path = fixture / "resume-second-report.json"
    resumed = subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(resume_manifest),
            "--route-receipt",
            str(resume_receipt_path),
            "--resume-report",
            str(first_resume_report_path),
            "--report",
            str(resumed_report_path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert resumed.returncode == 0
    resumed_report = json.loads(resumed_report_path.read_text(encoding="utf-8"))
    assert resumed_report["state"] == "passed"
    assert resumed_report["reusedGateIds"] == ["fixture_resume_a_passed"]
    assert resumed_report["launchedGateIds"] == [
        "fixture_resume_b_repairable",
        "fixture_resume_c_dependent",
    ]
    assert resumed_report["gates"][0]["reused"] is True

    timeout_marker = fixture / "timeout-child-marker"
    child_code = (
        "import pathlib,time; time.sleep(2); pathlib.Path(" + repr(str(timeout_marker)) + ").touch()"
    )
    parent_code = (
        "import subprocess,time; subprocess.Popen(["
        + repr(sys.executable)
        + ", '-c', "
        + repr(child_code)
        + "]); time.sleep(10)"
    )
    timeout_manifest = json.loads(resume_manifest.read_text(encoding="utf-8"))
    timeout_manifest["gates"] = [
        {
            "category": "fixture",
            "command": [sys.executable, "-c", parent_code],
            "id": "fixture_timeout",
            "minimumProfile": "light",
        }
    ]
    timeout_manifest_path = fixture / "timeout-manifest.json"
    timeout_manifest_path.write_text(
        json.dumps(timeout_manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    route.load_manifest(timeout_manifest_path)
    timeout_receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=timeout_manifest_path,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile=None,
    )
    timeout_receipt_path = fixture / "timeout-route.json"
    timeout_receipt_path.write_text(
        json.dumps(timeout_receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    timeout_report_path = fixture / "timeout-report.json"
    timed_out = subprocess.run(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(timeout_manifest_path),
            "--route-receipt",
            str(timeout_receipt_path),
            "--report",
            str(timeout_report_path),
            "--gate-timeout-seconds",
            "0.2",
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert timed_out.returncode == 1
    timeout_report = json.loads(timeout_report_path.read_text(encoding="utf-8"))
    assert timeout_report["gates"][0]["timedOut"] is True
    time.sleep(2.2)
    assert not timeout_marker.exists(), "timed-out gate leaked a child process"

    interrupt_marker = fixture / "interrupt-started.marker"
    interrupt_code = (
        "import pathlib,time; pathlib.Path(" + repr(str(interrupt_marker)) + ").touch(); time.sleep(10)"
    )
    interrupt_manifest = json.loads(timeout_manifest_path.read_text(encoding="utf-8"))
    interrupt_manifest["gates"] = [
        {
            "category": "fixture",
            "command": [sys.executable, "-c", interrupt_code],
            "id": "fixture_interrupt",
            "minimumProfile": "light",
        }
    ]
    interrupt_manifest_path = fixture / "interrupt-manifest.json"
    interrupt_manifest_path.write_text(
        json.dumps(interrupt_manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    route.load_manifest(interrupt_manifest_path)
    interrupt_receipt = route.plan_repository_route(
        repository=repository,
        manifest_path=interrupt_manifest_path,
        base=base,
        head=head,
        stage="pull_request",
        risk="normal",
        contract_path=None,
        requested_profile=None,
    )
    interrupt_receipt_path = fixture / "interrupt-route.json"
    interrupt_receipt_path.write_text(
        json.dumps(interrupt_receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    interrupt_report_path = fixture / "interrupt-report.json"
    interrupted = subprocess.Popen(
        [
            sys.executable,
            str(root / "tests/ci/run_repository_gates.py"),
            "--repo",
            str(repository),
            "--manifest",
            str(interrupt_manifest_path),
            "--route-receipt",
            str(interrupt_receipt_path),
            "--report",
            str(interrupt_report_path),
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    for _ in range(30):
        if interrupt_marker.exists():
            break
        time.sleep(0.1)
    assert interrupt_marker.exists(), "interrupt fixture did not start"
    interrupted.send_signal(signal.SIGTERM)
    interrupted_stdout, interrupted_stderr = interrupted.communicate(timeout=5)
    assert interrupted.returncode == 130, (interrupted_stdout, interrupted_stderr)
    interrupt_report = json.loads(interrupt_report_path.read_text(encoding="utf-8"))
    assert interrupt_report["state"] == "interrupted"
    assert interrupt_report["gates"][0]["state"] == "interrupted"
    assert interrupt_report["gates"][0]["failureCode"] == "gate_interrupted"

print("repository gate manifest regression passed")
