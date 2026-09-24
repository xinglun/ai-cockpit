#!/usr/bin/env python3
"""Exercise coordination with real CLI processes and linked worktrees."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path


def git(repo: Path, *args: str) -> str:
    result = subprocess.run(["git", *args], cwd=repo, text=True, capture_output=True, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {result.stderr}")
    return result.stdout.strip()


def run_cli(binary: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(binary), *args], text=True, capture_output=True, check=False)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    runtime_digest = "sha256:" + hashlib.sha256(binary.read_bytes()).hexdigest()
    runtime = {
        "schemaVersion": 1,
        "runtimeVersion": "0.2.113",
        "runtimeDigest": runtime_digest,
        "capability": "cross_wi_coordination_v1",
    }
    repository_id = "sha256:" + hashlib.sha256(b"acceptance-repository").hexdigest()

    with tempfile.TemporaryDirectory(prefix="ai-cockpit-cross-wi-") as temporary:
        root = Path(temporary) / "root"
        worktree_a = Path(temporary) / "wi-a"
        worktree_b = Path(temporary) / "wi-b"
        root.mkdir()
        git(root, "init", "-q")
        git(root, "config", "user.email", "acceptance@example.invalid")
        git(root, "config", "user.name", "Acceptance")
        (root / "README.md").write_text("acceptance\n")
        git(root, "add", ".")
        git(root, "commit", "-qm", "initial")
        git(root, "branch", "-M", "main")
        git(root, "worktree", "add", "-qb", "codex/wi-a", str(worktree_a), "main")
        git(root, "worktree", "add", "-qb", "codex/wi-b", str(worktree_b), "main")

        def registration(work_item_id: str, worktree: Path) -> dict:
            return {
                "schemaVersion": 1,
                "repositoryId": repository_id,
                "workItemId": work_item_id,
                "contractDigest": "sha256:" + hashlib.sha256(work_item_id.encode()).hexdigest(),
                "worktreePath": str(worktree),
                "branch": f"codex/{work_item_id.lower()}",
                "head": git(worktree, "rev-parse", "HEAD"),
                "generation": 1,
                "declaration": {
                    "providedOutcomes": [],
                    "consumedOutcomes": [],
                    "resourceClaims": [],
                    "integrationResponsibility": {
                        "responsibleWorkItemId": work_item_id,
                        "targetBranch": "main",
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

        input_a = Path(temporary) / "registration-a.json"
        input_b = Path(temporary) / "registration-b.json"
        input_a.write_text(json.dumps(registration("WI-A", worktree_a)))
        input_b.write_text(json.dumps(registration("WI-B", worktree_b)))
        commands = [
            ["work-item", "coordination", "register", "--repo", str(worktree_a), "--input", str(input_a)],
            ["work-item", "coordination", "register", "--repo", str(worktree_b), "--input", str(input_b)],
        ]
        processes = [subprocess.Popen([str(binary), *command], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE) for command in commands]
        results = [process.communicate() for process in processes]
        assert all(process.returncode == 0 for process in processes), results

        event = {
            "schemaVersion": 1,
            "eventId": "impact-concurrent",
            "repositoryId": repository_id,
            "workItemId": "WI-A",
            "generation": 1,
            "kind": "impact",
            "source": "process-acceptance",
            "evidenceRefs": ["acceptance/processes.json"],
        }
        event_path = Path(temporary) / "event.json"
        event_path.write_text(json.dumps(event))
        report_commands = [
            ["work-item", "coordination", "report-impact", "--repo", str(worktree_a), "--input", str(event_path)],
            ["work-item", "coordination", "report-impact", "--repo", str(worktree_a), "--input", str(event_path)],
        ]
        processes = [subprocess.Popen([str(binary), *command], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE) for command in report_commands]
        results = [process.communicate() for process in processes]
        assert all(process.returncode == 0 for process in processes), results

        inspection = run_cli(binary, ["work-item", "coordination", "inspect", "--repo", str(root)])
        assert inspection.returncode == 0, inspection.stderr
        projection = json.loads(inspection.stdout)
        assert len(projection["registrations"]) == 2
        assert len(projection["events"]) == 1
        print(json.dumps({
            "state": "passed",
            "realProcesses": 4,
            "linkedWorktrees": 2,
            "registrations": len(projection["registrations"]),
            "deduplicatedEvents": len(projection["events"]),
        }))


if __name__ == "__main__":
    main()
