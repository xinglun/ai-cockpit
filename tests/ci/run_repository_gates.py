#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import signal
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from quality_route import (
    PROFILE_ORDER,
    RouteValidationError,
    file_digest,
    load_manifest,
    parse_structured_failure,
    required_gate_ids as resolve_required_gate_ids,
    execution_order as resolve_execution_order,
    validate_route_receipt,
)


def load_receipt(path: Path) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError("route receipt must be a regular file")
    try:
        receipt = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"unable to load route receipt: {error}") from error
    if not isinstance(receipt, dict):
        raise ValueError("route receipt must be an object")
    return receipt


def write_report(path: Path, report: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    descriptor, temporary = tempfile.mkstemp(
        prefix=f".{path.name}.", suffix=".tmp", dir=path.parent
    )
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    except BaseException:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise


def preflight_failure(
    report_path: Path,
    code: str,
    detail: str,
    remediation: str,
    *,
    route_binding: dict[str, Any] | None = None,
) -> int:
    """Persist a failure before any repository gate command is launched."""
    write_report(
        report_path,
        {
            "failureMessage": detail,
            "failurePhase": "preflight",
            "failureRoots": [
                {
                    "code": code,
                    "gateId": "preflight",
                    "remediation": remediation,
                    "stage": "preflight",
                }
            ],
            "gates": [],
            "launchedGateIds": [],
            "route": route_binding or {},
            "schemaVersion": 2,
            "state": "failed",
        },
    )
    print(
        json.dumps(
            {"state": "failed", "failureCode": code, "remediation": remediation},
            sort_keys=True,
        ),
        file=sys.stderr,
    )
    return 1


def validate_route_with_rust(
    binary: str,
    *,
    repository: Path,
    manifest_path: Path,
    receipt_path: Path,
    receipt: dict[str, Any],
) -> None:
    """Use the shared Rust fact/rule boundary for production receipt checks.

    The Python validator remains the compatibility fallback for offline
    fixtures and older callers. The hosted path supplies the prebuilt binary,
    so route validation does not re-implement Git/manifest/Contract planning.
    """
    command = [
        binary,
        "gate-plan",
        "--repo",
        str(repository),
        "--manifest",
        str(manifest_path),
        "--base",
        str(receipt.get("baseRevision", "")),
        "--head",
        str(receipt.get("headRevision", "")),
        "--stage",
        str(receipt.get("stage", "")),
        "--risk",
        str(receipt.get("requestedRisk", "")),
        "--receipt",
        str(receipt_path),
        "--validate-receipt",
    ]
    contract = receipt.get("contractPath")
    if isinstance(contract, str) and contract:
        command.extend(["--contract", str(repository / contract)])
    completed = subprocess.run(command, check=False, capture_output=True, text=True)
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip() or "Rust gate-plan validation failed"
        structured = parse_structured_failure(completed.stderr + "\n" + completed.stdout)
        if structured is not None:
            code, remediation = structured
            raise RouteValidationError(
                code,
                "Rust gate-plan validation failed",
                remediation,
            )
        raise ValueError(detail)


def failure_code(gate_id: str, *, launch_error: bool = False, detail: str = "") -> str:
    """Return one deterministic root code for a failed repository gate."""
    normalized = detail.lower()
    if "invalid_premerge_finalize" in normalized:
        return "invalid_premerge_finalize"
    if "required_evidence_missing" in normalized:
        return "required_evidence_missing"
    if (
        gate_id == "conformance_reference_file_inventory"
        and "reference" in normalized
        and "inventory" in normalized
    ):
        return "reference_inventory_mismatch"
    if "lifecycle_transition_stale" in normalized:
        return "lifecycle_transition_stale"
    if "lifecycle_transition_invalid" in normalized:
        return "lifecycle_transition_invalid"
    prefix = "gate_launch_failed" if launch_error else "quality_gate_failed"
    return f"{prefix}:{gate_id}"


def failure_remediation(code: str, gate_id: str) -> str:
    if code == "invalid_premerge_finalize":
        return "repair the finalization binding and rerun the current Work Item checks"
    if code == "required_evidence_missing":
        return "collect the Contract-required evidence and rerun the declared verification"
    if code == "reference_inventory_mismatch":
        return "refresh the pinned reference inventory and rerun the conformance check"
    if code == "lifecycle_transition_stale":
        return "use the Runtime recovery path, refresh evidence, and push only the repaired state"
    if code == "lifecycle_transition_invalid":
        return "restore the declared lifecycle order and checkpoint/preflight bindings before pushing"
    if code.startswith("gate_launch_failed:"):
        return f"restore the executable command for gate {gate_id} and rerun this route"
    return f"run gate {gate_id} locally and repair its declared failing check"


def attach_workspace_package_failure(
    result: dict[str, Any], *, repository: Path
) -> None:
    """Copy bounded package failure facts into the canonical gate receipt."""
    if result.get("id") != "workspace_package_tests":
        return
    report_path = repository / "target/workspace-package-coverage.json"
    try:
        report = load_receipt(report_path)
    except (OSError, ValueError, KeyError, TypeError):
        return
    for key in ("failedPackage", "failedExitCode", "failureDiagnosticTail"):
        if key in report:
            result[key] = report[key]


def run_gate(command: list[str], repository: Path, timeout: float | None) -> subprocess.CompletedProcess[str]:
    """Run a gate in its own process group so timeout cleanup is bounded."""
    process = subprocess.Popen(
        command,
        cwd=repository,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=os.name == "posix",
    )
    try:
        stdout, stderr = process.communicate(timeout=timeout)
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
        if os.name == "posix":
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        else:
            process.kill()
        stdout, stderr = process.communicate()
        if isinstance(error, KeyboardInterrupt):
            raise
        raise subprocess.TimeoutExpired(
            command,
            timeout,
            output=stdout or error.output,
            stderr=stderr or error.stderr,
        ) from error
    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


def raise_keyboard_interrupt(signum: int, frame: Any) -> None:
    """Turn runner termination into the same bounded cleanup path as Ctrl-C."""
    del signum, frame
    raise KeyboardInterrupt


def load_contract_gate_report(
    path: Path,
    *,
    repository: Path,
    route: dict[str, Any],
) -> dict[str, Any]:
    report = load_receipt(path)
    required = {
        "schemaVersion",
        "kind",
        "state",
        "repositoryId",
        "workItemId",
        "contractDigest",
        "contractFileDigest",
        "repositorySnapshotDigest",
        "baseRevision",
        "comparisonBaseRevision",
        "headRevision",
        "changedPaths",
        "stage",
        "runner",
        "operation",
        "verificationTier",
        "evidenceAssurance",
        "dependencyConfidence",
        "decisionState",
        "blockers",
        "unknowns",
        "requiredChecks",
        "runtimeVersion",
        "runtimeDigest",
        "receiptDigest",
    }
    if set(report) != required or report["schemaVersion"] != 1:
        raise ValueError("Contract gate report fields do not match schemaVersion 1")
    if report["kind"] != "repository_contract_quality_gate":
        raise ValueError("Contract gate report kind is invalid")
    if report["state"] != "passed" or report["decisionState"] != "green":
        raise ValueError("Contract gate did not produce a green passing decision")
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", report["repositoryId"]):
        raise ValueError("Contract gate repositoryId is invalid")
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", report["receiptDigest"]):
        raise ValueError("Contract gate receiptDigest is invalid")
    config = repository / ".ai/cockpit.toml"
    text = config.read_text(encoding="utf-8") if config.is_file() else ""
    match = re.search(r"^repository_id\s*=\s*\"([^\"]+)\"\s*$", text, re.MULTILINE)
    if match is None or report["repositoryId"] != match.group(1):
        raise ValueError("Contract gate repositoryId is not bound to repository config")
    contract_path = route.get("contractPath")
    if not isinstance(contract_path, str) or not contract_path:
        raise ValueError("Contract gate report requires the route Contract")
    if report["contractFileDigest"] != file_digest(repository / contract_path):
        raise ValueError("Contract gate file digest does not match route Contract")
    try:
        contract_value = json.loads((repository / contract_path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"unable to load route Contract for baseline binding: {error}") from error
    if report["baseRevision"] != contract_value.get("baseRevision"):
        raise ValueError("Contract gate baseRevision does not match route Contract baseline")
    if report["comparisonBaseRevision"] != route.get("baseRevision"):
        raise ValueError("Contract gate comparisonBaseRevision does not match route receipt")
    expected_stage = "pr" if route.get("stage") == "pull_request" else route.get("stage")
    if report["stage"] != expected_stage or report["runner"] != "hosted":
        raise ValueError("Contract gate stage or runner does not match CI route")
    expected_work_item = Path(contract_path).name.removesuffix(".contract.json")
    if report["workItemId"] != expected_work_item:
        raise ValueError("Contract gate workItemId does not match route Contract")
    if report["blockers"] or report["unknowns"]:
        raise ValueError("green Contract gate cannot contain blockers or unknowns")
    return report


def main() -> int:
    signal.signal(signal.SIGINT, raise_keyboard_interrupt)
    if hasattr(signal, "SIGTERM"):
        signal.signal(signal.SIGTERM, raise_keyboard_interrupt)
    parser = argparse.ArgumentParser(
        description="Run only canonical repository gates selected by a typed route receipt"
    )
    parser.add_argument("--repo", required=True)
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--report", required=True)
    parser.add_argument("--route-receipt")
    parser.add_argument("--contract-gate-report")
    parser.add_argument("--gate-plan-bin")
    parser.add_argument("--profile", choices=PROFILE_ORDER)
    parser.add_argument("--list-only", action="store_true")
    parser.add_argument(
        "--resume-report",
        help="reuse passed gates from a bound partial or failed report",
    )
    parser.add_argument(
        "--gate-timeout-seconds",
        type=float,
        help="fail a gate and clean up its process after this many seconds",
    )
    args = parser.parse_args()

    if args.gate_timeout_seconds is not None and args.gate_timeout_seconds <= 0:
        parser.error("--gate-timeout-seconds must be positive")
    if args.list_only and args.resume_report:
        parser.error("--resume-report cannot be combined with --list-only")

    repository = Path(args.repo).resolve()
    manifest_path = Path(args.manifest).resolve()
    report_path = Path(args.report)
    if not report_path.is_absolute():
        report_path = repository / report_path

    try:
        manifest = load_manifest(manifest_path)
        if args.list_only:
            if args.route_receipt:
                raise ValueError("--route-receipt cannot be combined with --list-only")
            selected_profile = args.profile or "strict"
            selected_profile, required_gate_ids = resolve_required_gate_ids(
                manifest, selected_profile
            )
            route_binding: dict[str, Any] = {
                "manifestDigest": file_digest(manifest_path),
                "requiredGateIds": required_gate_ids,
                "executionOrder": resolve_execution_order(manifest, required_gate_ids),
                "selectedProfile": selected_profile,
            }
        else:
            if args.profile:
                raise ValueError("--profile is diagnostic-only and requires --list-only")
            if not args.route_receipt:
                raise ValueError("execution requires --route-receipt")
            receipt = load_receipt(Path(args.route_receipt))
            if args.gate_plan_bin:
                validate_route_with_rust(
                    args.gate_plan_bin,
                    repository=repository,
                    manifest_path=manifest_path,
                    receipt_path=Path(args.route_receipt).resolve(),
                    receipt=receipt,
                )
            else:
                validate_route_receipt(
                    receipt,
                    repository=repository,
                    manifest_path=manifest_path,
                )
            selected_profile = receipt["selectedProfile"]
            required_gate_ids = receipt["requiredGateIds"]
            route_binding = {
                "manifestDigest": receipt["manifestDigest"],
                "receiptDigest": receipt["receiptDigest"],
                "requiredGateIds": required_gate_ids,
                "executionOrder": receipt.get("executionOrder"),
                "selectedProfile": selected_profile,
            }
            if args.contract_gate_report:
                gate_report = load_contract_gate_report(
                    Path(args.contract_gate_report),
                    repository=repository,
                    route=receipt,
                )
                route_binding["contractGateReportDigest"] = file_digest(
                    Path(args.contract_gate_report)
                )
                route_binding["contractGateState"] = gate_report["state"]
            elif receipt.get("contractPath") and selected_profile != "light":
                raise ValueError(
                    "standard/strict Contract routes require --contract-gate-report"
                )
    except (OSError, ValueError, KeyError, TypeError) as error:
        if isinstance(error, RouteValidationError):
            return preflight_failure(
                report_path,
                error.code,
                str(error),
                error.remediation,
                route_binding=locals().get("route_binding"),
            )
        return preflight_failure(
            report_path,
            "gate_runner_preflight_failed",
            str(error),
            "repair the route, manifest, Contract, or bound evidence before rerunning",
            route_binding=locals().get("route_binding"),
        )

    selected_ids = set(required_gate_ids)
    gates_by_id = {gate["id"]: gate for gate in manifest["gates"]}
    if len(selected_ids) != len(required_gate_ids) or selected_ids - gates_by_id.keys():
        return preflight_failure(
            report_path,
            "route_receipt_gate_set_invalid",
            "route receipt contains duplicate or unknown required gate IDs",
            "regenerate the route receipt from the current manifest and repository facts",
            route_binding=route_binding,
        )
    selected_gates = [gate for gate in manifest["gates"] if gate["id"] in selected_ids]
    if [gate["id"] for gate in selected_gates] != required_gate_ids:
        return preflight_failure(
            report_path,
            "route_receipt_order_invalid",
            "route receipt gate order does not match the canonical manifest",
            "regenerate the route receipt from the current manifest and repository facts",
            route_binding=route_binding,
        )

    for gate in selected_gates:
        for dependency in gate.get("dependsOn", []):
            if dependency not in selected_ids:
                return preflight_failure(
                    report_path,
                    "route_dependency_missing",
                    f"selected gate {gate['id']} depends on unselected gate {dependency}",
                    "regenerate the route receipt with the complete dependency closure",
                    route_binding=route_binding,
                )
    expected_execution_order = resolve_execution_order(manifest, required_gate_ids)
    if not args.list_only:
        receipt_execution_order = route_binding.get("executionOrder")
        if receipt_execution_order != expected_execution_order:
            return preflight_failure(
                report_path,
                "route_receipt_execution_order_invalid",
                "route receipt execution order does not match the canonical cost-aware dependency order",
                "regenerate the route receipt from the current manifest and repository facts",
                route_binding=route_binding,
            )
    execution_order = expected_execution_order
    position = {gate_id: index for index, gate_id in enumerate(execution_order)}
    if any(
        position[dependency] >= position[gate["id"]]
        for gate in selected_gates
        for dependency in gate.get("dependsOn", [])
        if dependency in position
    ):
        return preflight_failure(
            report_path,
            "selected_gate_dependencies_cyclic",
            "route execution order violates selected gate dependencies",
            "regenerate the route receipt with the complete dependency closure",
            route_binding=route_binding,
        )
    gates = [gates_by_id[gate_id] for gate_id in execution_order]

    if args.list_only:
        write_report(
            report_path,
            {
                "executionOrder": execution_order,
                "gates": [
                    {
                        "category": gate["category"],
                        "command": gate["command"],
                        "dependsOn": gate.get("dependsOn", []),
                        "id": gate["id"],
                        "state": "listed",
                    }
                    for gate in gates
                ],
                "launchedGateIds": [],
                "reusedGateIds": [],
                "route": route_binding,
                "schemaVersion": 2,
                "state": "listed",
            },
        )
        return 0
    results: list[dict[str, Any]] = []
    results_by_id: dict[str, dict[str, Any]] = {}
    failure_roots: list[dict[str, str]] = []
    launched_gate_ids: list[str] = []
    reused_gate_ids: list[str] = []
    failed = False
    active_result: dict[str, Any] | None = None
    resume_results: dict[str, dict[str, Any]] = {}
    if args.resume_report:
        try:
            resume = load_receipt(Path(args.resume_report).resolve())
            if resume.get("schemaVersion") != 2 or resume.get("route") != route_binding:
                raise ValueError("resume report route binding does not match current route")
            if resume.get("executionOrder") != execution_order:
                raise ValueError("resume report execution order does not match current route")
            if resume.get("state") not in {"failed", "interrupted", "running"}:
                raise ValueError("resume report must be partial, failed, or interrupted")
            gate_by_id = {gate["id"]: gate for gate in gates}
            for prior in resume.get("gates", []):
                if (
                    isinstance(prior, dict)
                    and prior.get("state") == "passed"
                    and prior.get("id") in gate_by_id
                    and prior.get("command") == gate_by_id[prior["id"]]["command"]
                ):
                    resume_results[prior["id"]] = prior
        except (OSError, ValueError, KeyError, TypeError) as error:
            return preflight_failure(
                report_path,
                "resume_report_invalid",
                str(error),
                "use a partial report bound to the same route, manifest, and commands",
                route_binding=route_binding,
            )

    def checkpoint(state: str, *, failure_phase: str | None = None) -> None:
        report: dict[str, Any] = {
            "executionOrder": execution_order,
            "gates": results,
            "launchedGateIds": launched_gate_ids,
            "reusedGateIds": reused_gate_ids,
            "route": route_binding,
            "schemaVersion": 2,
            "state": state,
        }
        if failure_phase is not None:
            report["failurePhase"] = failure_phase
        if failure_roots:
            report["failureRoots"] = failure_roots
        write_report(report_path, report)

    try:
        for gate in gates:
            result: dict[str, Any] = {
                "category": gate["category"],
                "command": gate["command"],
                "id": gate["id"],
            }
            if gate.get("covers"):
                result["covers"] = gate["covers"]
            dependencies = gate.get("dependsOn", [])
            if dependencies:
                result["dependsOn"] = dependencies
            if gate["id"] in resume_results:
                result = dict(resume_results[gate["id"]])
                result["reused"] = True
                reused_gate_ids.append(gate["id"])
                print(f"repository gate {gate['id']}: reused", flush=True)
            else:
                blocked_by = [
                    dependency
                    for dependency in dependencies
                    if dependency in results_by_id
                    and results_by_id[dependency].get("state") != "passed"
                ]
                if blocked_by:
                    result["blockedBy"] = blocked_by
                    result["failureCode"] = "prerequisite_failed"
                    result["remediation"] = (
                        "repair the prerequisite gate(s), then rerun this route once"
                    )
                    result["state"] = "blocked"
                    failed = True
                    print(
                        f"repository gate {result['id']}: blocked [prerequisite_failed]",
                        flush=True,
                    )
                else:
                    command = list(gate["command"])
                    if command[0].endswith(".sh"):
                        command.insert(0, "bash")
                    launched_gate_ids.append(result["id"])
                    active_result = result
                    try:
                        completed = run_gate(command, repository, args.gate_timeout_seconds)
                    except subprocess.TimeoutExpired:
                        result["timedOut"] = True
                        result["state"] = "failed"
                        result["failureCode"] = "gate_timeout"
                        result["remediation"] = (
                            f"repair or split gate {result['id']}, then rerun this route"
                        )
                        failed = True
                    except OSError as error:
                        detail = str(error)
                        result["launchError"] = "gate command could not be started"
                        result["state"] = "failed"
                        code = failure_code(result["id"], launch_error=True, detail=detail)
                        result["failureCode"] = code
                        result["remediation"] = failure_remediation(code, result["id"])
                        failed = True
                    else:
                        result["exitCode"] = completed.returncode
                        result["state"] = "passed" if completed.returncode == 0 else "failed"
                        if completed.returncode != 0:
                            detail = (completed.stderr or completed.stdout or "").strip()
                            code = failure_code(result["id"], detail=detail)
                            result["failureCode"] = code
                            result["remediation"] = failure_remediation(code, result["id"])
                            attach_workspace_package_failure(result, repository=repository)
                            if detail:
                                result["diagnosticDigest"] = "sha256:" + hashlib.sha256(
                                    detail.encode("utf-8", errors="replace")
                                ).hexdigest()
                            failed = True
                    active_result = None
                if result["state"] == "failed":
                    code = result["failureCode"]
                    if not any(root["code"] == code for root in failure_roots):
                        failure_roots.append(
                            {
                                "code": code,
                                "gateId": result["id"],
                                "remediation": result["remediation"],
                            }
                        )
                status = result["state"]
                code_suffix = f" [{result['failureCode']}]" if status == "failed" else ""
                print(f"repository gate {result['id']}: {status}{code_suffix}", flush=True)
            results.append(result)
            results_by_id[result["id"]] = result
            active_result = None
            checkpoint("failed" if failed else "running", failure_phase="execution")
    except KeyboardInterrupt:
        if active_result is not None and active_result["id"] not in results_by_id:
            active_result["state"] = "interrupted"
            active_result["exitCode"] = 130
            active_result["failureCode"] = "gate_interrupted"
            active_result["remediation"] = (
                "resume from the partial report after repairing the interruption"
            )
            results.append(active_result)
            results_by_id[active_result["id"]] = active_result
            failure_roots.append(
                {
                    "code": "gate_interrupted",
                    "gateId": active_result["id"],
                    "remediation": active_result["remediation"],
                }
            )
        if active_result is None:
            failure_roots.append(
                {
                    "code": "gate_interrupted",
                    "gateId": "execution",
                    "remediation": "resume from the partial report after repairing the interruption",
                }
            )
        checkpoint("interrupted", failure_phase="execution")
        return 130

    report = {
        "gates": results,
        "executionOrder": execution_order,
        "launchedGateIds": launched_gate_ids,
        "route": route_binding,
        "schemaVersion": 2,
        "state": "listed" if args.list_only else ("failed" if failed else "passed"),
        "reusedGateIds": reused_gate_ids,
    }
    if failure_roots:
        report["failureRoots"] = failure_roots
        report["failurePhase"] = "execution"
    write_report(report_path, report)
    return 1 if failed else 0


if __name__ == "__main__":
    getattr(sys, "exit")(main())
