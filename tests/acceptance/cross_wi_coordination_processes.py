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
import sys
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


def mcp_tool_call(binary: Path, repo: Path, name: str, arguments: dict) -> dict:
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments},
    }
    result = subprocess.run(
        [str(binary), "mcp", "--repo", str(repo)],
        input=json.dumps(request) + "\n",
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"MCP {name} process failed: {result.stderr}")
    responses = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    response = responses[0] if responses else {}
    if response.get("result", {}).get("isError") is True:
        raise RuntimeError(f"MCP {name} failed: {response}")
    content = response.get("result", {}).get("structuredContent")
    if not isinstance(content, dict):
        raise RuntimeError(f"MCP {name} returned no structured content: {response}")
    return content


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def digest_json(value: object) -> str:
    # Runtime digests serde_json::Value, whose object keys serialize in sorted
    # order; match its compact byte representation.
    return digest_bytes(json.dumps(value, sort_keys=True, separators=(",", ":")).encode())


def execution_records_digest(records: list[dict]) -> str:
    # CompositionExecutionRecord is a typed serde struct, so its fields use
    # declaration order rather than serde_json::Value's sorted-key order.
    fields = (
        "nodeId",
        "program",
        "args",
        "identityDigest",
        "spawned",
        "reused",
        "passed",
        "exitCode",
        "terminationSignal",
        "stdout",
        "stderr",
        "outputDigest",
        "timedOut",
        "predecessorAttemptId",
    )

    def compact(value: object) -> str:
        return json.dumps(value, ensure_ascii=False, separators=(",", ":"))

    serialized_records = []
    for record in records:
        assert set(record).issubset(fields), record
        serialized_records.append(
            "{"
            + ",".join(
                f"{compact(field)}:{compact(record[field])}"
                for field in fields
                if field in record
            )
            + "}"
        )
    return digest_bytes(("[" + ",".join(serialized_records) + "]").encode())


def assert_successful_composition_result(
    result: dict,
    runtime: dict,
    *,
    expected_generation: int,
) -> None:
    assert result["schemaVersion"] == 3, result
    assert result["passed"] is True, result
    assert result["failure"] is None, result
    assert result["executionOutcome"] == "passed", result
    assert result["executionEvidenceComplete"] is True, result
    assert result["cleanupDisposition"] == "cleaned", result
    cleanup = result["cleanup"]
    assert cleanup["attempted"] is True, result
    assert cleanup["removed"] is True, result
    assert cleanup["error"] is None, result
    assert result["executionRecords"], result
    assert all(
        record["passed"] is True for record in result["executionRecords"]
    ), result

    binding = result["binding"]
    assert binding["schemaVersion"] == 1, binding
    verifier = binding["verifier"]
    receipt = result["supervisorReceipt"]
    assert isinstance(receipt, dict), result
    assert receipt["schemaVersion"] == 1, receipt
    assert receipt["attemptId"] == result["attemptId"], result
    assert receipt["runNonce"], receipt
    assert receipt["generation"] == expected_generation, receipt
    assert (
        receipt["runtimeVersion"]
        == runtime["runtimeVersion"]
        == verifier["runtimeVersion"]
    ), receipt
    assert (
        receipt["runtimeDigest"]
        == runtime["runtimeDigest"]
        == verifier["runtimeDigest"]
    ), receipt
    assert receipt["repositoryId"] == binding["repositoryId"], receipt
    assert receipt["targetSha"] == binding["targetSha"], receipt
    assert receipt["commandPlanDigest"] == result["identity"]["commandDigest"], receipt
    assert receipt["executionRecordsDigest"] == execution_records_digest(
        result["executionRecords"]
    ), receipt
    assert receipt["supervisor"]["processId"] == result["ownerPid"], receipt

    if sys.platform == "linux":
        assert receipt["backend"] == "linux_subreaper", receipt
        assert receipt["descendantsReapedToEchild"] is True, receipt
        assert receipt["linuxBootId"], receipt


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
    workflow_reference = (repository_root / "docs/reference/agent-workflow.md").read_text(
        encoding="utf-8"
    )
    workflow_reference = re.sub(r"\s+", " ", workflow_reference)
    combined_guidance = f"{ordinary_guide} {workflow_reference}"
    guide_checks = {
        "serialByDefault": "One Work Item is serial by default;" in ordinary_guide,
        "discoversCliAndMcp": all(
            phrase in ordinary_guide
            for phrase in (
                "capability show",
                "CLI help",
                "MCP `tools/list`",
            )
        ),
        "referencesCoordinationWorkflow": (
            "docs/reference/agent-workflow.md" in ordinary_guide
            and "### Serial fallback and cross-Work-Item coordination" in workflow_reference
        ),
        "requiresRuntimeSlotLease": "current slot lease" in workflow_reference,
        "serialFallbackWhenUnsupported": all(
            phrase in combined_guidance
            for phrase in (
                "If unavailable, use admitted serial work or stop.",
                "continue serially if admitted, or stop if parallel coordination is required.",
                "do not silently ignore or imitate the protocol",
                "fields or an older Runtime do not prove support",
            )
        ),
    }
    assert all(guide_checks.values()), guide_checks
    runtime = {
        "schemaVersion": 1,
        "runtimeVersion": runtime_version(),
        "runtimeDigest": digest_bytes(binary.read_bytes()),
        "capability": "observed_environment_drift_v1",
    }

    with tempfile.TemporaryDirectory(prefix="ai-cockpit-cross-wi-") as temporary:
        temporary_path = Path(temporary)
        composition_tmp_one = temporary_path / "composition-tmp-one"
        composition_tmp_two = temporary_path / "composition-tmp-two"
        composition_tmp_one.mkdir()
        composition_tmp_two.mkdir()
        env_marker = temporary_path / "environment-command-started"
        marker_script = "mark-process.sh"
        root = temporary_path / "root"
        worktree_a = temporary_path / "wi-a"
        worktree_b = temporary_path / "wi-b"
        worktree_c = temporary_path / "wi-c"
        root.mkdir()
        (root / marker_script).write_text(f"touch {env_marker}\n")
        (root / "Cargo.toml").write_text(
            '[package]\nname = "cross-wi-acceptance"\nversion = "0.1.0"\nedition = "2024"\n'
        )
        (root / "README.md").write_text("acceptance\n")
        git(root, "init", "-q")
        git(root, "config", "user.email", "acceptance@example.invalid")
        git(root, "config", "user.name", "Acceptance")
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
        git(root, "worktree", "add", "-qb", "codex/wi-c", str(worktree_c), "main")
        git(worktree_a, "branch", "--set-upstream-to=origin/main")
        git(worktree_b, "branch", "--set-upstream-to=origin/main")
        git(worktree_c, "branch", "--set-upstream-to=origin/main")

        # Attach each checkout through the CLI, then share only the durable
        # repository identity. Contracts remain distinct and are created by
        # the Runtime in their respective linked worktrees.
        require_cli(binary, ["attach", "--repo", str(worktree_a)])
        require_cli(binary, ["attach", "--repo", str(worktree_b)])
        require_cli(binary, ["attach", "--repo", str(worktree_c)])
        for checkout in [worktree_b, worktree_c]:
            for shared_file in ["cockpit.toml", "project.json", "agent-interface.json"]:
                shutil.copy2(worktree_a / ".ai" / shared_file, checkout / ".ai" / shared_file)
        repository_id = re.search(
            r'^repository_id\s*=\s*"([^"]+)"$',
            (worktree_a / ".ai/cockpit.toml").read_text(),
            re.MULTILINE,
        )
        if repository_id is None:
            raise RuntimeError("attached repository identity is not observable")
        repository_id_value = repository_id.group(1)

        for work_item_id, worktree in [
            ("WI-A", worktree_a),
            ("WI-B", worktree_b),
            ("WI-C", worktree_c),
        ]:
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
        assert {
            "work-item-coordination",
            "work-item-parallel",
            "work-item-contract-amendment",
            "work-item-environment-drift",
        } <= capabilities
        listed_tools, server_info = mcp_tools(binary, worktree_a)
        listed_by_name = {tool["name"]: tool for tool in listed_tools}
        assert {
            "work_item_coordination",
            "work_item_parallel",
            "work_item_composition",
            "work_item_amend",
            "work_item_amendments",
            "work_item_environment_drift",
        } <= set(listed_by_name)
        amend_help = require_cli(binary, ["work-item", "amend", "--help"])
        assert "--request" in amend_help, amend_help
        assert "--input" in amend_help and "--reason" in amend_help, amend_help
        for tool_name in ["work_item_amend", "work_item_amendments"]:
            schema = listed_by_name[tool_name]["inputSchema"]
            assert schema["additionalProperties"] is False, (tool_name, schema)
        amendment_schema = listed_by_name["work_item_amend"]["inputSchema"]["properties"]["request"]
        assert amendment_schema["additionalProperties"] is False, amendment_schema
        assert amendment_schema["properties"]["changes"]["items"]["additionalProperties"] is False
        environment_schema = listed_by_name["work_item_environment_drift"]["inputSchema"]
        assert environment_schema["additionalProperties"] is False, environment_schema
        assert environment_schema["properties"]["action"]["enum"] == ["check", "record"]
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
            "request-pause", "acknowledge", "resume", "recover",
            "check-environment-drift", "record-environment-drift", "help",
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
        assert registration_schema["properties"]["runtime"]["properties"]["capability"]["const"] == "observed_environment_drift_v1"
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
            assert "work_item_amend" not in legacy_names
            assert "work_item_amendments" not in legacy_names
            assert "work_item_environment_drift" not in legacy_names
            legacy_amend_help = require_cli(
                legacy_binary, ["work-item", "amend", "--help"]
            )
            assert "--request" not in legacy_amend_help, legacy_amend_help
            legacy_version = legacy_info.get("version")
            candidate_version = server_info.get("version")
            assert isinstance(legacy_version, str) and legacy_version
            assert isinstance(candidate_version, str) and candidate_version
            assert legacy_info.get("runtimeDigest") != server_info.get("runtimeDigest")

            # Exercise the actual compatibility boundary, not just tool
            # discovery: an amended Contract carries a required Runtime
            # capability marker, and the exact predecessor parser must reject
            # it rather than silently ignore the new constraint.
            amended_repo = temporary_path / "legacy-amended-contract"
            amended_repo.mkdir()
            git(amended_repo, "init", "-q")
            git(amended_repo, "config", "user.email", "acceptance@example.invalid")
            git(amended_repo, "config", "user.name", "Acceptance")
            (amended_repo / "README.md").write_text("legacy compatibility acceptance\n")
            git(amended_repo, "add", ".")
            git(amended_repo, "commit", "-qm", "initial")
            git(amended_repo, "branch", "-M", "main")
            require_cli(binary, ["attach", "--repo", str(amended_repo)])
            work_item_id = "WI-LEGACY-AMENDMENT-CAPABILITY"
            require_cli(
                binary,
                [
                    "start", "--repo", str(amended_repo), "--id", work_item_id,
                    "--intent", "test predecessor Contract compatibility",
                    "--goal", "an amended Contract declares its required Runtime capabilities",
                    "--scope", "README.md", "--out-of-scope", "target/**",
                    "--authority", "authorized",
                    "--acceptance", "the exact predecessor rejects unsupported capabilities",
                    "--verification", "true", "--prepare",
                ],
            )
            amended_contract = (
                amended_repo / ".ai/work-items/active" / f"{work_item_id}.contract.json"
            )
            amendment_request = temporary_path / "legacy-amendment-request.json"
            amendment_request.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "changeId": "legacy-capability-boundary",
                        "expectedContractDigest": contract_digest(amended_contract),
                        "reason": "test exact predecessor rejection of a newly constrained Contract",
                        "changes": [
                            {
                                "path": "/goal",
                                "operation": "replace",
                                "value": "the predecessor cannot execute this amended Contract",
                            }
                        ],
                    }
                )
            )
            require_cli(
                binary,
                [
                    "work-item", "amend", "--repo", str(amended_repo), "--id", work_item_id,
                    "--request", str(amendment_request),
                ],
            )
            amended_contract_value = json.loads(amended_contract.read_text())
            assert amended_contract_value["requiredRuntimeCapabilities"] == [
                "work-item-contract-amendment", "work-item-environment-drift"
            ], amended_contract_value

            # Use a separate unattached repository so the old Runtime reaches
            # Contract deserialization instead of stopping at the repository
            # capability manifest. The Contract itself is the tested boundary.
            legacy_target = temporary_path / "legacy-preflight-target"
            legacy_target.mkdir()
            git(legacy_target, "init", "-q")
            legacy_preflight = run_cli(
                legacy_binary,
                [
                    "preflight", "--repo", str(legacy_target),
                    "--contract", str(amended_contract),
                ],
            )
            legacy_preflight_output = legacy_preflight.stdout + legacy_preflight.stderr
            assert legacy_preflight.returncode != 0, legacy_preflight_output
            assert "unknown field" in legacy_preflight_output, legacy_preflight_output
            assert "requiredRuntimeCapabilities" in legacy_preflight_output, legacy_preflight_output

            legacy_observation = {
                "manifestReadable": True,
                "agentDoctorState": legacy_doctor_report["state"],
                "agentDoctorExitCode": legacy_doctor.returncode,
                "runtimeVersion": legacy_version,
                "candidateRuntimeVersion": candidate_version,
                "runtimeDigest": legacy_info.get("runtimeDigest"),
                "coordinationToolAdvertised": False,
                "compositionToolAdvertised": False,
                "amendedContractCapabilityMarker": amended_contract_value["requiredRuntimeCapabilities"],
                "amendedContractRejectedByParser": True,
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
        contract_value["verification"] = [{"check": f"sh {marker_script}", "required": True}]
        contract_b.write_text(json.dumps(contract_value, indent=2) + "\n")
        contract_a = worktree_a / ".ai/work-items/active/WI-A.contract.json"
        contract_a_value = json.loads(contract_a.read_text())
        contract_a_value["verification"] = [{"check": "true", "required": True}]
        contract_a.write_text(json.dumps(contract_a_value, indent=2) + "\n")
        contract_c = worktree_c / ".ai/work-items/active/WI-C.contract.json"
        contract_c_value = json.loads(contract_c.read_text())
        contract_c_value["verification"] = [{"check": "env", "required": True}]
        contract_c.write_text(json.dumps(contract_c_value, indent=2) + "\n")

        def registration(work_item_id: str, worktree: Path, generation: int = 1) -> dict:
            contract = worktree / ".ai/work-items/active" / f"{work_item_id}.contract.json"
            branch = git(worktree, "branch", "--show-current")
            head = git(worktree, "rev-parse", "HEAD")
            provided_outcomes = []
            consumed_outcomes = []
            if work_item_id == "WI-A":
                provided_outcomes = [{
                    "outcomeId": "WI-A-api",
                    "interfaceContract": "stable process acceptance API",
                    "behaviorContract": "environment changes invalidate consumers",
                    "publishedHead": head,
                    "stage": "composable_head",
                    "evidenceRefs": [],
                }]
            if work_item_id == "WI-B":
                consumed_outcomes = [{
                    "providerWorkItemId": "WI-A",
                    "outcomeId": "WI-A-api",
                    "minimumStage": "composable_head",
                    "verificationRequired": False,
                }]
            return {
                "schemaVersion": 1,
                "repositoryId": repository_id_value,
                "workItemId": work_item_id,
                "contractDigest": contract_digest(contract),
                "worktreePath": str(worktree.resolve()),
                "branch": branch,
                "head": head,
                "generation": generation,
                "declaration": {
                    "providedOutcomes": provided_outcomes,
                    "consumedOutcomes": consumed_outcomes,
                    "resourceClaims": [],
                    "integrationResponsibility": {
                        "responsibleWorkItemId": work_item_id,
                        "targetBranch": "main",
                        "compositionOrder": ["WI-A", "WI-B"] if work_item_id == "WI-B" else [work_item_id],
                        "rationale": "process acceptance",
                    },
                    "compositionVerification": {
                        "compatibilityConstraints": [],
                        "requiredScenarios": [],
                        "reusableNodes": (
                            ["required-check-a"] if work_item_id == "WI-A"
                            else ["required-check"] if work_item_id == "WI-B"
                            else ["unrelated-check"] if work_item_id == "WI-C"
                            else []
                        ),
                    },
                },
                "runtime": runtime,
            }

        input_a = temporary_path / "registration-a.json"
        input_b = temporary_path / "registration-b.json"
        input_c = temporary_path / "registration-c.json"
        input_a.write_text(json.dumps(registration("WI-A", worktree_a)))
        input_b.write_text(json.dumps(registration("WI-B", worktree_b)))
        input_c.write_text(json.dumps(registration("WI-C", worktree_c)))
        registration_commands = [
            ["work-item", "coordination", "register", "--repo", str(worktree_a), "--input", str(input_a)],
            ["work-item", "coordination", "register", "--repo", str(worktree_b), "--input", str(input_b)],
            ["work-item", "coordination", "register", "--repo", str(worktree_c), "--input", str(input_c)],
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
            "workItemId": "WI-C",
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

        # Run the dependent composition twice through real CLI processes. The
        # input contains a deliberately false caller precondition; Runtime must
        # replace it with facts recomputed from registrations and Git.
        head_b = git(worktree_b, "rev-parse", "HEAD")
        head_a = git(worktree_a, "rev-parse", "HEAD")
        affected_marker = env_marker
        command = {
            "nodeId": "required-check",
            "program": "sh",
            "args": [marker_script],
            "dependsOn": [],
            "environment": {},
            "inputPaths": ["README.md", marker_script],
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
                "participantWorkItems": ["WI-A", "WI-B"],
                "participantHeads": [head_a, head_b],
                "contractDigests": [contract_digest(contract_a), contract_digest(contract_b)],
                "verifier": runtime,
            },
            "commands": [
                {
                    "nodeId": "required-check-a",
                    "program": "true",
                    "args": [],
                    "dependsOn": [],
                    "environment": {},
                    "inputPaths": ["README.md"],
                    "coveredScenarios": [],
                    "coveredConstraints": [],
                },
                command,
            ],
            "preconditions": [
                {"name": "caller-claim", "satisfied": False, "reason": "not authoritative"}
            ],
            "timeoutSeconds": 1,
        }
        composition_path = temporary_path / "composition.json"
        composition_path.write_text(json.dumps(composition_input))
        env_marker.unlink(missing_ok=True)
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
        composition_bytes = composition_path.read_bytes()
        first_result = first["result"]
        assert first_result["passed"] is True, first
        assert first_result["schemaVersion"] == 3, first_result
        assert_successful_composition_result(
            first_result, runtime, expected_generation=1
        )
        assert first_result["processesSpawned"] == 2, first_result
        assert affected_marker.exists(), "first dependent composition did not execute its process"
        affected_marker.unlink()
        registration_bytes = input_a.read_bytes()

        # A Runtime-observed dependency drift is reported without changing the
        # registration or composition JSON supplied by the caller.
        cargo_manifest = worktree_a / "Cargo.toml"
        cargo_manifest.write_text(cargo_manifest.read_text() + "\n# observed environment drift\n")
        drift_request = temporary_path / "environment-drift.json"
        drift_request.write_text(json.dumps({
            "schemaVersion": 1,
            "workItemId": "WI-A",
            "expectedGeneration": 1,
        }))
        events_before_drift_check = json.loads(require_cli(binary, [
            "work-item", "coordination", "inspect", "--repo", str(root),
        ]))["events"]
        check_drift = json.loads(require_cli(binary, [
            "work-item", "coordination", "check-environment-drift",
            "--repo", str(worktree_b), "--input", str(drift_request),
        ]))
        assert check_drift["pending"] is True, check_drift
        mcp_check_drift = mcp_tool_call(
            binary,
            worktree_b,
            "work_item_environment_drift",
            {"action": "check", "workItemId": "WI-A", "generation": 1},
        )
        assert mcp_check_drift == {"pending": True, "writePerformed": False}, mcp_check_drift
        events_after_drift_check = json.loads(require_cli(binary, [
            "work-item", "coordination", "inspect", "--repo", str(root),
        ]))["events"]
        assert events_after_drift_check == events_before_drift_check, "read-only drift check wrote an event"
        record_drift = mcp_tool_call(
            binary,
            worktree_a,
            "work_item_environment_drift",
            {"action": "record", "workItemId": "WI-A", "generation": 1},
        )
        drift_event_id = record_drift["result"]["eventId"]
        assert drift_event_id, record_drift
        cli_record_retry = json.loads(require_cli(binary, [
            "work-item", "coordination", "record-environment-drift",
            "--repo", str(worktree_a), "--input", str(drift_request),
        ]))
        assert cli_record_retry["result"]["eventId"] == drift_event_id, cli_record_retry
        assert input_a.read_bytes() == registration_bytes
        assert composition_path.read_bytes() == composition_bytes

        # Affected work is denied before the touch process can start, while an
        # unrelated Work Item continues to execute in the same shared repo.
        denied = run_cli(binary, composition_args)
        assert denied.returncode != 0, denied.stdout
        assert not affected_marker.exists(), "affected command spawned despite drift blocker"
        affected_denied_before_spawn = denied.returncode != 0 and not affected_marker.exists()

        contract_c = worktree_c / ".ai/work-items/active/WI-C.contract.json"
        head_c = git(worktree_c, "rev-parse", "HEAD")
        unrelated_input = {
            "repositoryRoot": str(worktree_c),
            "stateDir": str(temporary_path / "caller-state-c"),
            "binding": {
                "schemaVersion": 1,
                "repositoryId": repository_id_value,
                "bindingId": "unrelated-composition",
                "targetBranch": "main",
                "targetSha": git(root, "rev-parse", "refs/heads/main"),
                "participantWorkItems": ["WI-C"],
                "participantHeads": [head_c],
                "contractDigests": [contract_digest(contract_c)],
                "verifier": runtime,
            },
            "commands": [{
                "nodeId": "unrelated-check",
                "program": "env",
                "args": [],
                "dependsOn": [],
                "environment": {},
                "inputPaths": ["README.md"],
                "coveredScenarios": [],
                "coveredConstraints": [],
            }],
            "preconditions": [],
            "timeoutSeconds": 1,
        }
        unrelated_path = temporary_path / "unrelated-composition.json"
        unrelated_path.write_text(json.dumps(unrelated_input))
        unrelated_args = [
            "work-item", "composition", "--repo", str(worktree_c),
            "--id", "WI-C", "--generation", "1", "--input", str(unrelated_path),
        ]
        unrelated_first = json.loads(require_cli(binary, unrelated_args))
        unrelated_second = json.loads(require_cli(binary, unrelated_args))
        changed_environment = json.loads(
            require_cli(binary, unrelated_args, {"TMPDIR": str(composition_tmp_two)})
        )
        unrelated_result = unrelated_first["result"]
        changed_environment_result = changed_environment["result"]
        assert unrelated_result["passed"] is True, unrelated_first
        assert_successful_composition_result(
            unrelated_result, runtime, expected_generation=1
        )
        assert unrelated_result["processesSpawned"] == 1, unrelated_result
        unrelated_second_result = unrelated_second["result"]
        assert_successful_composition_result(
            unrelated_second_result, runtime, expected_generation=1
        )
        assert unrelated_second["result"]["processesSpawned"] == 0, unrelated_second
        assert unrelated_second["result"]["executionRecords"][0]["reused"] is True, unrelated_second
        assert changed_environment_result["passed"] is True, changed_environment
        assert_successful_composition_result(
            changed_environment_result, runtime, expected_generation=1
        )
        assert changed_environment_result["processesSpawned"] == 1, changed_environment_result
        assert changed_environment_result["executionRecords"][0]["reused"] is False, changed_environment_result
        assert changed_environment_result["identity"]["environmentDigest"] != unrelated_second["result"]["identity"]["environmentDigest"]
        assert unrelated_result["executionRecords"][0]["spawned"] is True, unrelated_first

        # Advance the provider generation with a fresh Runtime observation,
        # then append a recovery for the original event and consumer generation.
        registration_a_v2 = temporary_path / "registration-a-generation-2.json"
        registration_a_v2.write_text(json.dumps(registration("WI-A", worktree_a, 2)))
        require_cli(binary, [
            "work-item", "coordination", "register", "--repo", str(worktree_a),
            "--input", str(registration_a_v2),
        ])
        recovery = json.loads(require_cli(binary, [
            "work-item", "coordination", "recover", "--repo", str(worktree_b),
            "--event-id", drift_event_id, "--consumer-work-item-id", "WI-B",
            "--consumer-generation", "1",
        ]))
        assert recovery["result"]["eventId"] == drift_event_id, recovery

        # Recovery permits a fresh dependent validation. Give it a new node
        # identity so the pre-drift receipt cannot be reused for this run.
        post_recovery_marker = env_marker
        composition_input["binding"]["bindingId"] = "post-recovery-composition"
        composition_path.write_text(json.dumps(composition_input))
        post_recovery_marker.unlink(missing_ok=True)
        post_recovery = json.loads(require_cli(binary, composition_args))
        assert post_recovery["result"]["passed"] is True, post_recovery
        assert_successful_composition_result(
            post_recovery["result"], runtime, expected_generation=1
        )
        assert post_recovery["result"]["processesSpawned"] == 2, post_recovery
        assert post_recovery_marker.exists(), "post-recovery validation did not spawn"

        inspection = json.loads(
            require_cli(binary, ["work-item", "coordination", "inspect", "--repo", str(root)])
        )
        assert len(inspection["registrations"]) == 3, inspection
        assert len(inspection["events"]) == 2, inspection
        print(
            json.dumps(
                {
                    "state": "passed",
                    "linkedWorktrees": 3,
                    "registrations": len(inspection["registrations"]),
                    "deduplicatedEvents": len(inspection["events"]),
                    "firstCompositionProcesses": first_result["processesSpawned"],
                    "secondCompositionProcesses": unrelated_second["result"]["processesSpawned"],
                    "environmentDriftCheckPending": check_drift["pending"],
                    "mcpEnvironmentDriftReadOnly": mcp_check_drift["writePerformed"] is False,
                    "mcpEnvironmentDriftEventId": drift_event_id,
                    "affectedProcessDeniedBeforeSpawn": affected_denied_before_spawn,
                    "unrelatedProcessContinued": unrelated_result["processesSpawned"] == 1,
                    "changedEnvironmentReexecutedUnrelatedCheck": changed_environment_result["processesSpawned"] == 1,
                    "recoveryRecorded": recovery["result"]["eventId"] == drift_event_id,
                    "postRecoveryProcesses": post_recovery["result"]["processesSpawned"],
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
