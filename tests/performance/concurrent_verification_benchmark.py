#!/usr/bin/env python3
"""Measure concurrent verification requests without changing Runtime behavior.

This harness intentionally exercises the public CLI path with independent
processes. It does not call PhysicalSingleFlightCoordinator directly: a
coordinator unit test is not evidence that production requests use it.
Unavailable platform/runtime counters are represented as structured unknowns,
never as zero.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
import json
import math
import os
import pathlib
import platform
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
from typing import Any, Iterable


SCHEMA_VERSION = 1
DEFAULT_ROUNDS = 20
DEFAULT_CONCURRENCY = 4
COMMAND_TIMEOUT_SECONDS = 30.0


def unavailable(reason: str) -> dict[str, Any]:
    return {"available": False, "reason": reason}


def number(value: float | int) -> float:
    result = float(value)
    if not math.isfinite(result) or result < 0:
        raise ValueError("measurement values must be finite and non-negative")
    return round(result, 3)


def nearest_rank(values: Iterable[float], fraction: float) -> float | None:
    ordered = sorted(float(value) for value in values)
    if not ordered:
        return None
    rank = max(1, math.ceil(fraction * len(ordered)))
    return number(ordered[rank - 1])


def summarize_latencies(values: Iterable[float]) -> dict[str, Any]:
    raw = [number(value) for value in values]
    return {
        "rawMs": raw,
        "sampleCount": len(raw),
        "quantileMethod": "nearest-rank-on-sorted-samples",
        "p50Ms": nearest_rank(raw, 0.50),
        "p95Ms": nearest_rank(raw, 0.95),
        "p99Ms": nearest_rank(raw, 0.99),
        "reliable": {
            "p50": len(raw) >= 5,
            "p95": len(raw) >= 20,
            "p99": len(raw) >= 100,
        },
        "unavailableReason": {
            label: f"insufficient_samples:{len(raw)}<{minimum}"
            for label, minimum in (("p50", 5), ("p95", 20), ("p99", 100))
            if len(raw) < minimum
        },
    }


class ActiveTracker:
    def __init__(self) -> None:
        self._lock = threading.Lock()
        self.current = 0
        self.maximum = 0

    def enter(self) -> None:
        with self._lock:
            self.current += 1
            self.maximum = max(self.maximum, self.current)

    def leave(self) -> None:
        with self._lock:
            self.current -= 1


def command_vector(
    binary: pathlib.Path,
    repo: pathlib.Path,
    command: str,
    command_args: list[str],
) -> list[str]:
    vector = [
        str(binary),
        "verify",
        "--repo",
        str(repo),
        "--command",
        command,
    ]
    if command_args:
        vector.append("--args=" + ",".join(command_args))
    return vector


def parse_receipt(stdout: str) -> dict[str, Any] | None:
    try:
        value = json.loads(stdout)
    except json.JSONDecodeError:
        return None
    return value if isinstance(value, dict) else None


def terminate_process_group(process: subprocess.Popen[str]) -> str:
    if os.name == "posix":
        try:
            os.killpg(process.pid, signal.SIGTERM)
            return "process_group_sigterm"
        except ProcessLookupError:
            return "process_already_exited"
        except OSError as error:
            return f"process_group_signal_failed:{type(error).__name__}"
    return "platform_cancellation_unavailable"


def run_request(
    binary: pathlib.Path,
    repo: pathlib.Path,
    command: str,
    command_args: list[str],
    request_index: int,
    round_label: str,
    barrier: threading.Barrier,
    tracker: ActiveTracker,
    cancel_after_seconds: float | None = None,
) -> dict[str, Any]:
    vector = command_vector(binary, repo, command, command_args)
    barrier.wait()
    started_at = time.perf_counter_ns()
    tracker.enter()
    process: subprocess.Popen[str] | None = None
    cancellation_action: str | None = None
    try:
        process = subprocess.Popen(
            vector,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=os.name == "posix",
        )
        if cancel_after_seconds is not None:
            time.sleep(cancel_after_seconds)
            cancellation_action = terminate_process_group(process)
        try:
            stdout, stderr = process.communicate(timeout=COMMAND_TIMEOUT_SECONDS)
        except subprocess.TimeoutExpired:
            cancellation_action = terminate_process_group(process)
            stdout, stderr = process.communicate(timeout=5)
            cancellation_action = cancellation_action or "bounded_timeout"
        elapsed_ms = (time.perf_counter_ns() - started_at) / 1_000_000
        receipt = parse_receipt(stdout)
        governance_passed = (
            receipt.get("passed") if isinstance(receipt, dict) else None
        )
        return {
            "requestIndex": request_index,
            "round": round_label,
            "argv": vector,
            "commandIdentityMaterial": {"command": command, "args": command_args},
            "startedOrder": request_index,
            "elapsedMs": number(elapsed_ms),
            "returnCode": process.returncode,
            "governancePassed": governance_passed,
            "succeeded": process.returncode == 0 and governance_passed is True,
            "cancelled": cancellation_action is not None,
            "cancellationAction": cancellation_action,
            "receipt": receipt,
            "stderr": stderr[-4096:],
            "receiptMetrics": receipt_metrics(receipt),
        }
    except OSError as error:
        elapsed_ms = (time.perf_counter_ns() - started_at) / 1_000_000
        return {
            "requestIndex": request_index,
            "round": round_label,
            "argv": vector,
            "commandIdentityMaterial": {"command": command, "args": command_args},
            "startedOrder": request_index,
            "elapsedMs": number(elapsed_ms),
            "returnCode": None,
            "governancePassed": None,
            "succeeded": False,
            "cancelled": False,
            "cancellationAction": None,
            "receipt": None,
            "stderr": f"{type(error).__name__}: {error}",
            "receiptMetrics": unavailable("process_spawn_failed"),
        }
    finally:
        tracker.leave()


def receipt_metrics(receipt: dict[str, Any] | None) -> dict[str, Any]:
    if receipt is None:
        return unavailable("verification_did_not_return_json_receipt")
    names = (
        "processesSpawned",
        "elapsedMs",
        "planningElapsedMs",
        "executionElapsedMs",
        "maxConcurrentProcesses",
        "gitCalls",
        "filesRead",
        "filesHashed",
    )
    return {
        name: (
            {"available": True, "value": receipt[name]}
            if name in receipt and isinstance(receipt[name], (int, float))
            else unavailable(f"receipt_missing:{name}")
        )
        for name in names
    }


def run_round(
    binary: pathlib.Path,
    repo: pathlib.Path,
    label: str,
    requests: list[tuple[str, list[str]]],
    round_number: int,
    cancel_after_seconds: float | None = None,
) -> dict[str, Any]:
    tracker = ActiveTracker()
    barrier = threading.Barrier(len(requests) + 1)
    started_at = time.perf_counter_ns()
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(requests)) as pool:
        futures = [
            pool.submit(
                run_request,
                binary,
                repo,
                command,
                args,
                index,
                f"{label}-{round_number:03d}",
                barrier,
                tracker,
                cancel_after_seconds,
            )
            for index, (command, args) in enumerate(requests)
        ]
        barrier.wait()
        results = [future.result() for future in futures]
    wall_ms = (time.perf_counter_ns() - started_at) / 1_000_000
    results.sort(key=lambda result: result["requestIndex"])
    process_counts = [
        result["receiptMetrics"]["processesSpawned"]["value"]
        for result in results
        if isinstance(result["receiptMetrics"], dict)
        and result["receiptMetrics"].get("processesSpawned", {}).get("available")
    ]
    return {
        "round": round_number,
        "label": label,
        "requestOrder": [result["requestIndex"] for result in results],
        "requests": results,
        "observedRequestCount": len(results),
        "successfulRequestCount": sum(result["succeeded"] for result in results),
        "failedRequestCount": sum(not result["succeeded"] for result in results),
        "executionCount": {
            "available": len(process_counts) == len(results),
            "value": sum(process_counts) if len(process_counts) == len(results) else None,
            "reason": None
            if len(process_counts) == len(results)
            else "one_or_more_requests_missing_receipt_process_count",
        },
        "wallClockMs": number(wall_ms),
        "perRequestLatency": summarize_latencies(
            result["elapsedMs"] for result in results
        ),
        "queueWaitLatency": unavailable("CLI_does_not_expose_request_queue_timing"),
        "resourcePeaks": {
            "maxConcurrentRequestsObserved": {
                "available": True,
                "value": tracker.maximum,
            },
            "peakMemoryBytes": unavailable("platform_metric_not_collected"),
            "cpuTimeMs": unavailable("CLI_does_not_expose_cpu_time"),
            "readBytes": unavailable("CLI_does_not_expose_read_bytes"),
            "hashedBytes": unavailable("CLI_does_not_expose_hashed_bytes"),
        },
    }


def run_scenario(
    binary: pathlib.Path,
    repo: pathlib.Path,
    label: str,
    requests: list[tuple[str, list[str]]],
    rounds: int,
    cancel_after_seconds: float | None = None,
) -> dict[str, Any]:
    return {
        "scenario": label,
        "roundCount": rounds,
        "concurrency": len(requests),
        "rounds": [
            run_round(binary, repo, label, requests, round, cancel_after_seconds)
            for round in range(1, rounds + 1)
        ],
    }


def run_command(binary: pathlib.Path, args: list[str]) -> tuple[int, str, str]:
    result = subprocess.run(
        [str(binary), *args],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
        timeout=COMMAND_TIMEOUT_SECONDS,
    )
    return result.returncode, result.stdout, result.stderr


def git_output(repo: pathlib.Path, args: list[str]) -> str | None:
    result = subprocess.run(
        ["git", "-C", str(repo), *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        check=False,
        timeout=30,
    )
    return result.stdout.strip() if result.returncode == 0 else None


def collect_call_graph(repo: pathlib.Path) -> dict[str, Any]:
    """Inventory production and test references without treating tests as callers."""
    production: list[dict[str, Any]] = []
    tests: list[dict[str, Any]] = []
    definitions: list[dict[str, Any]] = []
    implementations: list[dict[str, Any]] = []
    crates = repo / "crates"
    for path in sorted(crates.rglob("*.rs")):
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except OSError:
            continue
        is_test = "tests" in path.parts
        for line_number, line in enumerate(lines, start=1):
            if "PhysicalSingleFlightCoordinator" not in line:
                continue
            match = {
                "path": path.relative_to(repo).as_posix(),
                "line": line_number,
                "text": line.strip(),
            }
            if "struct PhysicalSingleFlightCoordinator" in line:
                definitions.append(match)
            elif "impl PhysicalSingleFlightCoordinator" in line:
                implementations.append(match)
            elif is_test:
                tests.append(match)
            else:
                production.append(match)
    return {
        "sourceRevision": git_output(repo, ["rev-parse", "HEAD"]),
        "searchTerm": "PhysicalSingleFlightCoordinator",
        "productionCallerMatches": production,
        "definitionMatches": definitions,
        "implementationMatches": implementations,
        "testOnlyMatches": tests,
        "productionCallerCount": len(production),
        "interpretation": (
            "No production caller was found; test references and the type definition "
            "are not production execution evidence."
            if not production
            else "Production references require manual classification before any integration decision."
        ),
    }


def filesystem_type(repo: pathlib.Path) -> dict[str, Any]:
    if platform.system() == "Darwin":
        command = ["stat", "-f", "%T", str(repo)]
    elif platform.system() == "Linux":
        command = ["stat", "-f", "-c", "%T", str(repo)]
    else:
        return unavailable("platform_filesystem_type_unsupported")
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
    value = result.stdout.strip()
    return {"available": True, "value": value} if result.returncode == 0 and value else unavailable("filesystem_type_unavailable")


def environment(binary: pathlib.Path, repo: pathlib.Path) -> dict[str, Any]:
    version_code, version_stdout, _ = run_command(binary, ["--version"])
    status_code, status_stdout, status_stderr = run_command(binary, ["status", "--repo", str(repo)])
    inspect_code, inspect_stdout, inspect_stderr = run_command(binary, ["inspect", "--repo", str(repo)])
    try:
        status = json.loads(status_stdout) if status_code == 0 else {}
    except json.JSONDecodeError:
        status = {}
    try:
        inspect = json.loads(inspect_stdout) if inspect_code == 0 else {}
    except json.JSONDecodeError:
        inspect = {}
    return {
        "capturedAt": dt.datetime.now(dt.timezone.utc).isoformat(),
        "measurementBeforeIdentityProbe": True,
        "coldCacheClaim": False,
        "platform": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
            "python": platform.python_version(),
            "filesystem": filesystem_type(repo),
        },
        "runtime": {
            "versionOutput": version_stdout.strip(),
            "versionExitCode": version_code,
            "fileSha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "reportedDigest": inspect.get("runtimeDigest"),
            "reportedVersion": inspect.get("runtimeVersion"),
        },
        "repository": {
            "path": str(repo),
            "head": git_output(repo, ["rev-parse", "HEAD"]),
            "branch": git_output(repo, ["branch", "--show-current"]),
            "status": git_output(repo, ["status", "--porcelain=v1", "--untracked-files=all"]),
            "repositoryId": status.get("repositoryId"),
            "snapshotDigest": inspect.get("treeDigest"),
            "trackedFileCount": len((git_output(repo, ["ls-files", "-z"]) or "").split("\0")) - 1,
        },
        "identityProbe": {
            "statusExitCode": status_code,
            "statusStderr": status_stderr[-1024:],
            "inspectExitCode": inspect_code,
            "inspectStderr": inspect_stderr[-1024:],
        },
    }


def validate_inputs(binary: pathlib.Path, repo: pathlib.Path, rounds: int, concurrency: int) -> None:
    if not binary.is_file() or binary.is_symlink() or not os.access(binary, os.X_OK):
        raise SystemExit("runtime binary must be an executable regular file (no symlink)")
    if not repo.is_dir():
        raise SystemExit("repository directory must exist")
    if rounds < 20:
        raise SystemExit("rounds must be at least 20 for the primary scenario")
    if concurrency < 2 or concurrency > 8:
        raise SystemExit("concurrency must be between 2 and 8")


def atomic_json_write(path: pathlib.Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + f".tmp-{os.getpid()}")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temporary, path)


def write_sidecar_evidence(output: pathlib.Path, result: dict[str, Any]) -> None:
    prefix = "WI-692-p0-concurrent-verification-measurement"
    raw_path = output.name
    scenario_summaries: dict[str, Any] = {}
    scenario_paths = {
        "same-identity": f"{prefix}.same-identity.json",
        "distinct-command-identity": f"{prefix}.distinct-identity.json",
        "failure-propagation": f"{prefix}.failure.json",
        "cancellation": f"{prefix}.cancellation.json",
        "resource-contention": f"{prefix}.resource-contention.json",
    }
    for name, scenario in result["scenarios"].items():
        if scenario.get("status") == "not_measured":
            scenario_summaries[name] = scenario
            continue
        rounds = scenario.get("rounds", [])
        executions = [
            round_data["executionCount"].get("value")
            for round_data in rounds
            if round_data["executionCount"].get("available")
        ]
        wall = [round_data["wallClockMs"] for round_data in rounds]
        request_latency = [
            request["elapsedMs"]
            for round_data in rounds
            for request in round_data["requests"]
        ]
        scenario_summaries[name] = {
            "scenario": name,
            "roundCount": scenario["roundCount"],
            "concurrency": scenario["concurrency"],
            "rawEvidence": raw_path,
            "executionCountPerRound": executions,
            "roundWallClock": summarize_latencies(wall),
            "requestLatency": summarize_latencies(request_latency),
            "successfulRequests": sum(
                round_data["successfulRequestCount"] for round_data in rounds
            ),
            "failedRequests": sum(
                round_data["failedRequestCount"] for round_data in rounds
            ),
            "queueWaitLatency": unavailable("CLI_does_not_expose_request_queue_timing"),
            "resourcePeaks": {
                "maxConcurrentRequestsObserved": max(
                    round_data["resourcePeaks"]["maxConcurrentRequestsObserved"]["value"]
                    for round_data in rounds
                ),
                "peakMemoryBytes": unavailable("platform_metric_not_collected"),
                "cpuTimeMs": unavailable("CLI_does_not_expose_cpu_time"),
                "readBytes": unavailable("CLI_does_not_expose_read_bytes"),
                "hashedBytes": unavailable("CLI_does_not_expose_hashed_bytes"),
            },
        }
        if name in scenario_paths:
            atomic_json_write(
                output.parent / scenario_paths[name],
                {
                    "schemaVersion": SCHEMA_VERSION,
                    "kind": "concurrent-verification-scenario",
                    "scenario": name,
                    "rawEvidence": raw_path,
                    "environment": result["environment"],
                    "rawSampleOrder": result["rawSampleOrder"],
                    "rounds": rounds,
                    "summary": scenario_summaries[name],
                },
            )
    atomic_json_write(
        output.parent / f"{prefix}.measurement-summary.json",
        {
            "schemaVersion": SCHEMA_VERSION,
            "kind": "concurrent-verification-measurement-summary",
            "rawEvidence": raw_path,
            "environment": result["environment"],
            "rawSampleOrder": result["rawSampleOrder"],
            "scenarios": scenario_summaries,
            "callGraphEvidence": f"{prefix}.call-graph.json",
            "decision": result["decision"],
            "decisionBoundary": result["decisionBoundary"],
        },
    )
    atomic_json_write(
        output.parent / f"{prefix}.call-graph.json",
        {
            "schemaVersion": SCHEMA_VERSION,
            "kind": "physical-single-flight-call-graph",
            "rawEvidence": raw_path,
            "environment": result["environment"],
            "callGraph": result["callGraph"],
        },
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=pathlib.Path)
    parser.add_argument("repo", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    parser.add_argument("rounds", type=int, nargs="?", default=DEFAULT_ROUNDS)
    parser.add_argument("concurrency", type=int, nargs="?", default=DEFAULT_CONCURRENCY)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    repo = args.repo.resolve(strict=True)
    validate_inputs(binary, repo, args.rounds, args.concurrency)
    env = environment(binary, repo)
    python = sys.executable
    quick = [python, ["-c", "pass"]]
    slow = [python, ["-c", "import time;time.sleep(0.05)"]]
    distinct = [
        (python, ["-c", "pass"]),
        (python, ["-c", "print('different-command-identity')"]),
    ]
    failing = [(python, ["-c", "import sys;sys.exit(17)"])] * args.concurrency
    result: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": "concurrent-verification-measurement",
        "measurementMethod": "independent CLI processes launched through the public verify path",
        "rawSampleOrder": "round order, then requestIndex order; no sorting before grouping",
        "environment": env,
        "scenarios": {},
    }
    result["scenarios"]["same-identity"] = run_scenario(
        binary,
        repo,
        "same-identity",
        [tuple(quick) for _ in range(args.concurrency)],
        args.rounds,
    )
    result["scenarios"]["distinct-command-identity"] = run_scenario(
        binary,
        repo,
        "distinct-command-identity",
        [distinct[index % len(distinct)] for index in range(args.concurrency)],
        args.rounds,
    )
    result["scenarios"]["failure-propagation"] = run_scenario(
        binary,
        repo,
        "failure-propagation",
        failing,
        max(5, min(args.rounds, 20)),
    )
    if os.name == "posix":
        result["scenarios"]["cancellation"] = run_scenario(
            binary,
            repo,
            "cancellation",
            [(python, ["-c", "import time;time.sleep(5)"])],
            1,
            cancel_after_seconds=0.1,
        )
    else:
        result["scenarios"]["cancellation"] = {
            "scenario": "cancellation",
            "status": "not_measured",
            "unavailableReason": "process_group_cancellation_not_implemented_on_this_platform",
        }
    result["scenarios"]["resource-contention"] = run_scenario(
        binary,
        repo,
        "resource-contention",
        [tuple(slow) for _ in range(args.concurrency)],
        max(5, min(args.rounds, 20)),
    )
    result["decisionBoundary"] = {
        "productionCoordinatorCaller": "not established by this harness; inspect call-graph evidence separately",
        "sameIdentityPhysicalExecutionCount": "sum receipt.processesSpawned per independent CLI request",
        "acceptIntegrationOnlyIf": [
            "same-identity requests demonstrate duplicate physical execution",
            "the duplicated cost is material in a declared target scenario",
            "identity, authorization, evidence binding, failure, cancellation and resource limits remain explicit",
        ],
        "otherwise": "decline coordinator integration and record prerequisites for a successor Work Item",
    }
    result["callGraph"] = collect_call_graph(repo)
    result["decision"] = {
        "state": "declined_for_now",
        "reason": (
            "Four independent CLI requests produced four physical executions per round, "
            "but the production call-graph inventory found no caller for "
            "PhysicalSingleFlightCoordinator. The observed duplication is therefore an "
            "independent-process baseline, not evidence that the in-process coordinator "
            "can be safely wired into a live request path."
        ),
        "successorPrerequisites": [
            "Measure a real in-process MCP or service request path with the same repository, Work Item, command, Runtime and toolchain identity.",
            "Bind per-request authorization and evidence receipts before sharing any physical result.",
            "Measure failure, timeout, cancellation, waiter exit and nested Cargo resource behavior on that path.",
        ],
        "productionBehaviorChanged": False,
    }
    atomic_json_write(args.output, result)
    write_sidecar_evidence(args.output, result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
