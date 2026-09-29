#!/usr/bin/env python3
"""Exercise governed coordination with real CLI processes and linked worktrees."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
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


def run_cli(
    binary: Path,
    args: list[str],
    environment: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    child_environment = None
    if environment is not None:
        child_environment = os.environ.copy()
        child_environment.update(environment)
    return subprocess.run(
        [str(binary), *args],
        text=True,
        capture_output=True,
        check=False,
        env=child_environment,
    )


def require_cli(
    binary: Path,
    args: list[str],
    environment: dict[str, str] | None = None,
) -> str:
    result = run_cli(binary, args, environment)
    if result.returncode != 0:
        raise RuntimeError(
            f"CLI {' '.join(args)} failed ({result.returncode}): {result.stderr}"
        )
    return result.stdout


def mcp_tools(binary: Path, repo: Path) -> tuple[list[dict], dict]:
    requests = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
    ]
    result = subprocess.run(
        [str(binary), "mcp", "--repo", str(repo)],
        input="\n".join(json.dumps(request) for request in requests) + "\n",
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"MCP tools/list failed: {result.stderr}")
    responses = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    by_id = {response.get("id"): response for response in responses}
    initialize = by_id.get(1, {}).get("result", {})
    tools = by_id.get(2, {}).get("result", {}).get("tools")
    if not isinstance(tools, list):
        raise RuntimeError(f"MCP tools/list returned no tool array: {result.stdout}")
    return tools, initialize.get("serverInfo", {})


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def digest_json(value: object) -> str:
    # Runtime digests serde_json::Value, whose object keys serialize in sorted
    # order; match its compact byte representation.
    return digest_bytes(json.dumps(value, sort_keys=True, separators=(",", ":")).encode())


def contract_digest(path: Path) -> str:
    return digest_json(json.loads(path.read_text()))


def runtime_version() -> str:
    cargo = Path("Cargo.toml").read_text()
    match = re.search(r"^version\s*=\s*\"([^\"]+)\"", cargo, re.MULTILINE)
    if match is None:
        raise RuntimeError("workspace package version is not observable")
    return match.group(1)


def cli_subcommands(help_text: str) -> set[str]:
    commands = set()
    for line in help_text.splitlines():
        fields = line.split()
        if line.startswith("  ") and fields and fields[0][0].islower():
            commands.add(fields[0])
    return commands


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--binary",
        type=Path,
        default=Path("target/release/ai-cockpit"),
        help="CLI binary; defaults to the canonical release build",
    )
    parser.add_argument(
        "--legacy-binary",
        type=Path,
        required=True,
        help="fixed-lifecycle Runtime used to prove read compatibility is not capability support",
    )
    args = parser.parse_args()
    binary = args.binary.resolve()
    repository_root = Path(__file__).resolve().parents[2]
    ordinary_guide = (repository_root / "agents/skills/ordinary-work-item.md").read_text(
        encoding="utf-8"
    )
    ordinary_guide = re.sub(r"\s+", " ", ordinary_guide)
    guide_checks = {
        "serialByDefault": "One Work Item keeps a serial path;" in ordinary_guide,
        "discoversCliAndMcp": all(
            phrase in ordinary_guide
            for phrase in (
                "capability show --repo <repository>",
                "current CLI help",
                "MCP `tools/list` schemas",
            )
        ),
        "requiresRuntimeSlotLease": "Runtime slot lease" in ordinary_guide,
        "serialFallbackWhenUnsupported": all(
            phrase in ordinary_guide
            for phrase in (
                "If unsupported/unknown, use admitted serial work or stop;",
                "unsupported constraints",
                "ignore/emulate unsupported constraints",
                "never infer support from fields",
            )
        ),
    }
    assert all(guide_checks.values()), guide_checks
    runtime = {
        "schemaVersion": 1,
        "runtimeVersion": runtime_version(),
        "runtimeDigest": digest_bytes(binary.read_bytes()),
        "capability": "cross_wi_coordination_v1",
    }

    with tempfile.TemporaryDirectory(prefix="ai-cockpit-cross-wi-") as temporary:
        temporary_path = Path(temporary)
        composition_tmp_one = temporary_path / "composition-tmp-one"
        composition_tmp_two = temporary_path / "composition-tmp-two"
        composition_tmp_one.mkdir()
        composition_tmp_two.mkdir()
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

        # Capability metadata is discoverability only. Verify the actual MCP
        # tool surface and its strict nested argument schemas before exercising
        # the write APIs through real CLI processes below.
        manifest = json.loads((worktree_a / ".ai/agent-interface.json").read_text())
        capabilities = set(manifest.get("capabilities", []))
        assert {"work-item-coordination", "work-item-parallel"} <= capabilities
        listed_tools, server_info = mcp_tools(binary, worktree_a)
        listed_by_name = {tool["name"]: tool for tool in listed_tools}
        assert {"work_item_coordination", "work_item_parallel", "work_item_composition"} <= set(listed_by_name)
        candidate_doctor_result = run_cli(
            binary,
            ["agent", "doctor", "--repo", str(worktree_a), "--json"],
        )
        candidate_doctor = json.loads(candidate_doctor_result.stdout)
        assert candidate_doctor["manifest"]["state"] == "valid", candidate_doctor
        assert candidate_doctor["state"] in {"VERIFIED", "ATTACHED"}, candidate_doctor
        assert not candidate_doctor["problems"], candidate_doctor

        coordination_help = require_cli(
            binary, ["work-item", "coordination", "--help"]
        )
        coordination_cli_commands = cli_subcommands(coordination_help)
        assert coordination_cli_commands == {
            "inspect", "register", "report-impact", "publish-outcome",
            "request-pause", "acknowledge", "resume", "recover", "help",
        }, coordination_help
        slot_help = require_cli(binary, ["work-item", "slot", "--help"])
        slot_cli_commands = cli_subcommands(slot_help)
        assert {"acquire", "release", "list"} <= slot_cli_commands, slot_help
        composition_help = require_cli(binary, ["work-item", "composition", "--help"])
        assert all(
            option in composition_help
            for option in ["--repo", "--id", "--generation", "--input"]
        ), composition_help

        coordination_tool_schema = listed_by_name["work_item_coordination"]["inputSchema"]
        coordination_schema = coordination_tool_schema
        for property_name in ["registration", "event", "request"]:
            nested = coordination_schema["properties"][property_name]
            assert nested["type"] == "object", (property_name, nested)
            assert nested["additionalProperties"] is False, (property_name, nested)
            assert nested["properties"], (property_name, nested)
        registration_schema = coordination_schema["properties"]["registration"]
        assert registration_schema["properties"]["runtime"]["properties"]["runtimeDigest"]["pattern"]
        assert registration_schema["properties"]["declaration"]["properties"]["providedOutcomes"]["items"]["properties"]["stage"]["enum"] == [
            "interface_stable", "composable_head", "merged_target"
        ]
        assert set(registration_schema["required"]) == {
            "repositoryId", "workItemId", "contractDigest", "worktreePath",
            "branch", "head", "generation", "declaration", "runtime"
        }
        event_schema = coordination_schema["properties"]["event"]
        assert event_schema["properties"]["kind"]["enum"] == [
            "outcome_published", "interface_changed", "resource_changed",
            "execution_changed", "verification_changed", "impact"
        ]
        request_schema = coordination_schema["properties"]["request"]
        assert request_schema["properties"]["intent"]["enum"] == [
            "wait_for_dependency", "request_safe_pause",
            "adjust_integration_order", "resume_re_evaluate"
        ]
        assert request_schema["properties"]["state"]["enum"] == [
            "requested", "acknowledged", "safely_paused", "unavailable", "expired", "resumed"
        ]
        implicit_action_variants = [
            variant for variant in coordination_schema["oneOf"]
            if "action" not in variant.get("properties", {})
        ]
        assert len(implicit_action_variants) == 1
        coordination_actions = {
            variant["properties"]["action"]["const"]
            for variant in coordination_schema["oneOf"]
            if "action" in variant.get("properties", {})
        }
        coordination_actions.add("inspect")
        assert coordination_actions == {
            "inspect", "register", "report-impact", "publish-outcome",
            "request-pause", "acknowledge", "resume", "recover"
        }

        legacy_observation = None
        if args.legacy_binary is not None:
            legacy_binary = args.legacy_binary.resolve()
            legacy_doctor = run_cli(
                legacy_binary,
                ["agent", "doctor", "--repo", str(worktree_a), "--json"],
            )
            legacy_doctor_report = json.loads(legacy_doctor.stdout)
            assert legacy_doctor_report["manifest"]["state"] == "valid", legacy_doctor_report
            assert not legacy_doctor_report["problems"], legacy_doctor_report
            assert legacy_doctor_report["state"] in {"VERIFIED", "ATTACHED"}, legacy_doctor_report
            legacy_tools, legacy_info = mcp_tools(legacy_binary, worktree_a)
            legacy_names = {tool["name"] for tool in legacy_tools}
            assert "work_item_coordination" not in legacy_names
            assert "work_item_composition" not in legacy_names
            legacy_version = legacy_info.get("version")
            candidate_version = server_info.get("version")
            assert isinstance(legacy_version, str) and legacy_version
            assert isinstance(candidate_version, str) and candidate_version
            assert legacy_info.get("runtimeDigest") != server_info.get("runtimeDigest")
            legacy_observation = {
                "manifestReadable": True,
                "agentDoctorState": legacy_doctor_report["state"],
                "agentDoctorExitCode": legacy_doctor.returncode,
                "runtimeVersion": legacy_version,
                "candidateRuntimeVersion": candidate_version,
                "runtimeDigest": legacy_info.get("runtimeDigest"),
                "coordinationToolAdvertised": False,
                "compositionToolAdvertised": False,
                "sameVersion": legacy_version == candidate_version,
                "differentRuntimeDigest": True,
            }

        # A single-WI serial execution remains available without any parallel
        # boundary or slot lease. Parallel eligibility is not serial readiness.
        serial_root = temporary_path / "serial-root"
        serial_root.mkdir()
        git(serial_root, "init", "-q")
        git(serial_root, "config", "user.email", "acceptance@example.invalid")
        git(serial_root, "config", "user.name", "Acceptance")
        (serial_root / "README.md").write_text("serial acceptance\n")
        git(serial_root, "add", ".")
        git(serial_root, "commit", "-qm", "initial")
        git(serial_root, "branch", "-M", "main")
        require_cli(binary, ["attach", "--repo", str(serial_root)])
        require_cli(
            binary,
            [
                "start", "--repo", str(serial_root), "--id", "WI-SERIAL",
                "--intent", "single WI serial fallback acceptance",
                "--goal", "prove ordinary serial execution does not require parallel eligibility",
                "--scope", "README.md", "--out-of-scope", "target/**",
                "--authority", "authorized",
                "--acceptance", "one serial verification is admitted without a parallel boundary",
                "--verification", "true", "--prepare",
            ],
        )
        serial_inspect = json.loads(
            require_cli(binary, ["work-item", "inspect", "--repo", str(serial_root), "--id", "WI-SERIAL"])
        )
        assert serial_inspect["compatibility"]["compatible"] is False
        assert "parallel_compatibility_not_declared" in serial_inspect["compatibility"]["reasons"]
        denied_slot = run_cli(
            binary,
            ["work-item", "slot", "acquire", "--repo", str(serial_root), "--id", "WI-SERIAL"],
        )
        assert denied_slot.returncode != 0
        assert "concurrency boundary is not declared" in denied_slot.stderr
        serial_status = json.loads(
            require_cli(
                binary,
                ["work-item", "status", "--repo", str(serial_root), "--id", "WI-SERIAL", "--json"],
            )
        )
        assert serial_status["humanDecisionRequired"] is False
        assert "run_verification" in serial_status["safeActions"]
        serial_verify = json.loads(
            require_cli(
                binary,
                ["verify", "--repo", str(serial_root), "--work-item", "WI-SERIAL", "--command", "true"],
            )
        )
        assert serial_verify["passed"] is True, serial_verify
        assert serial_verify["processesSpawned"] == 1, serial_verify

        # Bind WI-B's composition command to the actual Contract required check.
        contract_b = worktree_b / ".ai/work-items/active/WI-B.contract.json"
        contract_value = json.loads(contract_b.read_text())
        contract_value["verification"] = [{"check": "env", "required": True}]
        contract_b.write_text(json.dumps(contract_value, indent=2) + "\n")

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
                        "targetBranch": "main",
                        "compositionOrder": [work_item_id],
                        "rationale": "process acceptance",
                    },
                    "compositionVerification": {
                        "compatibilityConstraints": [],
                        "requiredScenarios": [],
                        "reusableNodes": ["required-check"] if work_item_id == "WI-B" else [],
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
        command = {
            "nodeId": "required-check",
            "program": "env",
            "args": [],
            "dependsOn": [],
            "environment": {},
            "inputPaths": ["README.md"],
            "coveredScenarios": [],
            "coveredConstraints": [],
        }
        composition_input = {
            "repositoryRoot": str(worktree_b),
            "stateDir": str(temporary_path / "caller-state"),
            "binding": {
                "schemaVersion": 1,
                "repositoryId": repository_id_value,
                "bindingId": "process-composition",
                "targetBranch": "main",
                "targetSha": git(root, "rev-parse", "refs/heads/main"),
                "participantWorkItems": ["WI-B"],
                "participantHeads": [head_b],
                "contractDigests": [contract_digest(contract_b)],
                "verifier": runtime,
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
        first = json.loads(
            require_cli(binary, composition_args, {"TMPDIR": str(composition_tmp_one)})
        )
        composition_bytes = composition_path.read_bytes()
        second = json.loads(
            require_cli(binary, composition_args, {"TMPDIR": str(composition_tmp_one)})
        )
        assert composition_path.read_bytes() == composition_bytes
        changed_environment = json.loads(
            require_cli(binary, composition_args, {"TMPDIR": str(composition_tmp_two)})
        )
        first_result = first["result"]
        second_result = second["result"]
        changed_environment_result = changed_environment["result"]
        assert first_result["passed"] is True, first
        assert first_result["schemaVersion"] == 2, first_result
        assert second_result["passed"] is True, second
        assert first_result["processesSpawned"] == 1, first_result
        assert second_result["processesSpawned"] == 0, second_result
        assert second_result["executionRecords"][0]["reused"] is True, second_result
        assert changed_environment_result["passed"] is True, changed_environment
        assert changed_environment_result["processesSpawned"] == 1, changed_environment_result
        assert changed_environment_result["executionRecords"][0]["reused"] is False, changed_environment_result
        assert (
            changed_environment_result["identity"]["environmentDigest"]
            != second_result["identity"]["environmentDigest"]
        ), changed_environment_result

        inspection = json.loads(
            require_cli(binary, ["work-item", "coordination", "inspect", "--repo", str(root)])
        )
        assert len(inspection["registrations"]) == 2, inspection
        assert len(inspection["events"]) == 1, inspection
        print(
            json.dumps(
                {
                    "state": "passed",
                    "linkedWorktrees": 2,
                    "registrations": len(inspection["registrations"]),
                    "deduplicatedEvents": len(inspection["events"]),
                    "firstCompositionProcesses": first_result["processesSpawned"],
                    "secondCompositionProcesses": second_result["processesSpawned"],
                    "changedEnvironmentProcesses": changed_environment_result["processesSpawned"],
                    "manifestCapabilities": sorted(capabilities),
                    "mcpToolNames": sorted(listed_by_name),
                    "mcpCoordinationSchemaComplete": True,
                    "mcpCoordinationSchemaDigest": digest_json(coordination_tool_schema),
                    "mcpCoordinationSchema": coordination_tool_schema,
                    "currentRuntimeVersion": server_info.get("version"),
                    "currentRuntimeDigest": server_info.get("runtimeDigest"),
                    "candidateAgentDoctorState": candidate_doctor["state"],
                    "candidateAgentDoctorExitCode": candidate_doctor_result.returncode,
                    "candidateAgentDoctorManifestState": candidate_doctor["manifest"]["state"],
                    "coordinationCliCommands": sorted(coordination_cli_commands),
                    "slotCliCommands": sorted(slot_cli_commands),
                    "compositionCliArgsDiscovered": ["--repo", "--id", "--generation", "--input"],
                    "ordinaryWorkItemGuideChecks": guide_checks,
                    "legacyRuntimeReadCompatibility": legacy_observation,
                    "serialAllowedWithoutParallelDeclaration": serial_verify["passed"],
                    "serialAdmissionObserved": "run_verification" in serial_status["safeActions"],
                    "serialProcesses": serial_verify["processesSpawned"],
                    "undeclaredParallelLeaseRejected": denied_slot.returncode != 0,
                }
            )
        )


if __name__ == "__main__":
    main()
