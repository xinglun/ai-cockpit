#!/usr/bin/env python3
"""Exercise governed coordination with real CLI processes and linked worktrees."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


def git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=repo, text=True, capture_output=True, check=False
    )
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {result.stderr}")
    return result.stdout.strip()


def run_cli(binary: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(binary), *args], text=True, capture_output=True, check=False
    )


def require_cli(binary: Path, args: list[str]) -> str:
    result = run_cli(binary, args)
    if result.returncode != 0:
        raise RuntimeError(
            f"CLI {' '.join(args)} failed ({result.returncode}): {result.stderr}"
        )
    return result.stdout


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def digest_json(value: object) -> str:
    # Rust's protocol canonical_json is serde_json::to_vec: no whitespace,
    # preserving the generated struct/object field order.
    return digest_bytes(json.dumps(value, separators=(",", ":")).encode())


def contract_digest(path: Path) -> str:
    return digest_json(json.loads(path.read_text()))


def runtime_version() -> str:
    cargo = Path("Cargo.toml").read_text()
    match = re.search(r"^version\s*=\s*\"([^\"]+)\"", cargo, re.MULTILINE)
    if match is None:
        raise RuntimeError("workspace package version is not observable")
    return match.group(1)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--binary",
        type=Path,
        default=Path("target/release/ai-cockpit"),
        help="CLI binary; defaults to the canonical release build",
    )
    args = parser.parse_args()
    binary = args.binary.resolve()
    runtime = {
        "schemaVersion": 1,
        "runtimeVersion": runtime_version(),
        "runtimeDigest": digest_bytes(binary.read_bytes()),
        "capability": "cross_wi_coordination_v1",
    }

    with tempfile.TemporaryDirectory(prefix="ai-cockpit-cross-wi-") as temporary:
        temporary_path = Path(temporary)
        root = temporary_path / "root"
        worktree_a = temporary_path / "wi-a"
        worktree_b = temporary_path / "wi-b"
        root.mkdir()
        git(root, "init", "-q")
        git(root, "config", "user.email", "acceptance@example.invalid")
        git(root, "config", "user.name", "Acceptance")
        (root / "README.md").write_text("acceptance\n")
        git(root, "add", ".")
        git(root, "commit", "-qm", "initial")
        git(root, "branch", "-M", "main")
        remote = temporary_path / "origin.git"
        git(remote.parent, "init", "--bare", "-q", str(remote))
        git(root, "remote", "add", "origin", str(remote))
        git(root, "push", "-qu", "origin", "main")
        git(root, "remote", "set-head", "origin", "main")
        git(root, "worktree", "add", "-qb", "codex/wi-a", str(worktree_a), "main")
        git(root, "worktree", "add", "-qb", "codex/wi-b", str(worktree_b), "main")
        git(worktree_a, "branch", "--set-upstream-to=origin/main")
        git(worktree_b, "branch", "--set-upstream-to=origin/main")

        # Attach each checkout through the CLI, then share only the durable
        # repository identity. Contracts remain distinct and are created by
        # the Runtime in their respective linked worktrees.
        require_cli(binary, ["attach", "--repo", str(worktree_a)])
        require_cli(binary, ["attach", "--repo", str(worktree_b)])
        for shared_file in ["cockpit.toml", "project.json", "agent-interface.json"]:
            shutil.copy2(worktree_a / ".ai" / shared_file, worktree_b / ".ai" / shared_file)
        repository_id = re.search(
            r'^repository_id\s*=\s*"([^"]+)"$',
            (worktree_a / ".ai/cockpit.toml").read_text(),
            re.MULTILINE,
        )
        if repository_id is None:
            raise RuntimeError("attached repository identity is not observable")
        repository_id_value = repository_id.group(1)

        for work_item_id, worktree in [("WI-A", worktree_a), ("WI-B", worktree_b)]:
            require_cli(
                binary,
                [
                    "start",
                    "--repo",
                    str(worktree),
                    "--id",
                    work_item_id,
                    "--intent",
                    "process acceptance",
                    "--goal",
                    "bind coordination to observed repository facts",
                    "--scope",
                    ".ai/**,README.md",
                    "--out-of-scope",
                    "target/**",
                    "--authority",
                    "authorized",
                    "--acceptance",
                    "registration and composition identity are verified",
                ],
            )

        def registration(work_item_id: str, worktree: Path) -> dict:
            contract = worktree / ".ai/work-items/active" / f"{work_item_id}.contract.json"
            branch = git(worktree, "branch", "--show-current")
            head = git(worktree, "rev-parse", "HEAD")
            return {
                "schemaVersion": 1,
                "repositoryId": repository_id_value,
                "workItemId": work_item_id,
                "contractDigest": contract_digest(contract),
                "worktreePath": str(worktree.resolve()),
                "branch": branch,
                "head": head,
                "generation": 1,
                "declaration": {
                    "providedOutcomes": [],
                    "consumedOutcomes": [],
                    "resourceClaims": [],
                    "integrationResponsibility": {
                        "responsibleWorkItemId": work_item_id,
                        "targetBranch": branch,
                        "compositionOrder": [work_item_id],
                        "rationale": "process acceptance",
                    },
                    "compositionVerification": {
                        "compatibilityConstraints": [],
                        "requiredScenarios": [],
                        "reusableNodes": [],
                    },
                },
                "runtime": runtime,
            }

        input_a = temporary_path / "registration-a.json"
        input_b = temporary_path / "registration-b.json"
        input_a.write_text(json.dumps(registration("WI-A", worktree_a)))
        input_b.write_text(json.dumps(registration("WI-B", worktree_b)))
        registration_commands = [
            ["work-item", "coordination", "register", "--repo", str(worktree_a), "--input", str(input_a)],
            ["work-item", "coordination", "register", "--repo", str(worktree_b), "--input", str(input_b)],
        ]
        processes = [
            subprocess.Popen(
                [str(binary), *command],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            for command in registration_commands
        ]
        results = [process.communicate() for process in processes]
        assert all(process.returncode == 0 for process in processes), results

        event = {
            "schemaVersion": 1,
            "eventId": "impact-concurrent",
            "repositoryId": repository_id_value,
            "workItemId": "WI-A",
            "generation": 1,
            "kind": "impact",
            "source": "process-acceptance",
            "evidenceRefs": [],
        }
        event_path = temporary_path / "event.json"
        event_path.write_text(json.dumps(event))
        report_commands = [
            ["work-item", "coordination", "report-impact", "--repo", str(worktree_a), "--input", str(event_path)],
            ["work-item", "coordination", "report-impact", "--repo", str(worktree_a), "--input", str(event_path)],
        ]
        processes = [
            subprocess.Popen(
                [str(binary), *command],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            for command in report_commands
        ]
        results = [process.communicate() for process in processes]
        assert all(process.returncode == 0 for process in processes), results

        # Run the same composition twice through the real CLI. The input
        # contains a deliberately false caller precondition; the admitted path
        # must replace it with facts recomputed from the registrations/Git.
        head_b = git(worktree_b, "rev-parse", "HEAD")
        branch_b = git(worktree_b, "branch", "--show-current")
        contract_b = worktree_b / ".ai/work-items/active/WI-B.contract.json"
        command = {"nodeId": "required-check", "program": "sh", "args": ["-c", "true"]}
        composition_input = {
            "repositoryRoot": str(worktree_b),
            "stateDir": str(temporary_path / "caller-state"),
            "binding": {
                "schemaVersion": 1,
                "repositoryId": repository_id_value,
                "bindingId": "process-composition",
                "targetBranch": branch_b,
                "targetSha": head_b,
                "participantWorkItems": ["WI-B"],
                "participantHeads": [head_b],
                "contractDigests": [contract_digest(contract_b)],
                "verifier": runtime,
            },
            "identity": {
                "sourceDigest": digest_bytes(b"source"),
                "dependencyDigest": digest_bytes(b"dependency"),
                "interfaceDigest": digest_bytes(b"interface"),
                "configurationDigest": digest_bytes(b"configuration"),
                "toolchainDigest": digest_bytes(b"toolchain"),
                "lockfileDigest": digest_bytes(b"lockfile"),
                "generatedInputDigest": digest_bytes(b"generated"),
                "environmentDigest": digest_bytes(b"environment"),
                "verifierDigest": digest_bytes(b"verifier"),
                "commandDigest": digest_json([command]),
            },
            "commands": [command],
            "preconditions": [
                {"name": "caller-claim", "satisfied": False, "reason": "not authoritative"}
            ],
            "timeoutSeconds": 1,
        }
        composition_path = temporary_path / "composition.json"
        composition_path.write_text(json.dumps(composition_input))
        composition_args = [
            "work-item",
            "composition",
            "--repo",
            str(worktree_b),
            "--id",
            "WI-B",
            "--generation",
            "1",
            "--input",
            str(composition_path),
        ]
        first = json.loads(require_cli(binary, composition_args))
        second = json.loads(require_cli(binary, composition_args))
        first_result = first["result"]
        second_result = second["result"]
        assert first_result["passed"] is True, first
        assert second_result["passed"] is True, second
        assert first_result["processesSpawned"] == 1, first_result
        assert second_result["processesSpawned"] == 0, second_result
        assert second_result["executionRecords"][0]["reused"] is True, second_result

        inspection = json.loads(
            require_cli(binary, ["work-item", "coordination", "inspect", "--repo", str(root)])
        )
        assert len(inspection["registrations"]) == 2, inspection
        assert len(inspection["events"]) == 1, inspection
        print(
            json.dumps(
                {
                    "state": "passed",
                    "realProcesses": 6,
                    "linkedWorktrees": 2,
                    "registrations": len(inspection["registrations"]),
                    "deduplicatedEvents": len(inspection["events"]),
                    "firstCompositionProcesses": first_result["processesSpawned"],
                    "secondCompositionProcesses": second_result["processesSpawned"],
                }
            )
        )


if __name__ == "__main__":
    main()
