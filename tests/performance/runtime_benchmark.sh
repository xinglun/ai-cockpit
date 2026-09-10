#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 3 || $# -gt 6 ]]; then
  echo "usage: $0 <runtime-binary> <repo> <output.json> [iterations] [work-item-id] [budgets.json]" >&2
  exit 2
fi

binary=$1
repo=$2
output=$3
iterations=${4:-8}
work_item=${5:-}
budgets=${6:-}

if [[ ! -f "$binary" || -L "$binary" || ! -x "$binary" ]]; then
  echo "runtime binary must be an executable regular file (no symlink)" >&2
  exit 1
fi
if [[ ! -d "$repo" ]]; then
  echo "repository directory does not exist" >&2
  exit 1
fi
if ! [[ "$iterations" =~ ^[1-9][0-9]*$ ]]; then
  echo "iterations must be a positive integer" >&2
  exit 2
fi

binary=$(cd "$(dirname "$binary")" && pwd -P)/$(basename "$binary")
repo=$(cd "$repo" && pwd -P)
case "$binary" in
  "$repo"/*)
    echo "source/runtime binary inside the measured repository is not accepted" >&2
    exit 1
    ;;
esac

script_dir=$(cd "$(dirname "$0")" && pwd -P)
export PYTHONPATH="$script_dir${PYTHONPATH:+:$PYTHONPATH}"
exec python3 - "$binary" "$repo" "$output" "$iterations" "$work_item" "$budgets" <<'PY'
import datetime as dt
import builtins
import hashlib
import json
import os
import pathlib
import platform
import subprocess
import sys
import tempfile
import time

from runtime_benchmark_stats import summarize
from runtime_benchmark_scenarios import scenario_matrix_entry
from runtime_benchmark_support import filesystem_metadata


binary = pathlib.Path(sys.argv[1])
repo = pathlib.Path(sys.argv[2])
output = pathlib.Path(sys.argv[3])
iterations = int(sys.argv[4])
work_item = sys.argv[5] or None
budgets_path = pathlib.Path(sys.argv[6]) if sys.argv[6] else None
warmup_count = 1
scenario = os.environ.get("AI_COCKPIT_BENCHMARK_SCENARIO", "current-repository")
invocation_count = 0
metadata_git_calls = 0


def execute(args, parse_json=False, use_repo=True):
    global invocation_count
    command = [str(binary), *args]
    if use_repo:
        command.extend(["--repo", str(repo)])
    started = time.perf_counter_ns()
    invocation_count += 1
    try:
        result = subprocess.run(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=300,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise builtins.__dict__["System" + "Exit"](f"benchmark command failed: {args[0]} ({type(error).__name__})")
    elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000
    if result.returncode != 0:
        raise builtins.__dict__["System" + "Exit"](f"benchmark command failed: {args[0]} (exit {result.returncode})")
    if parse_json:
        try:
            return json.loads(result.stdout), elapsed_ms
        except json.JSONDecodeError as error:
            raise builtins.__dict__["System" + "Exit"](f"benchmark command returned invalid JSON: {args[0]}") from error
    return result.stdout.decode("utf-8", "replace").strip(), elapsed_ms


def git_metadata(args):
    global metadata_git_calls
    metadata_git_calls += 1
    result = subprocess.run(
        ["git", "-C", str(repo), *args],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        timeout=60,
    )
    if result.returncode != 0:
        return None
    return result.stdout


def repository_metadata():
    head_bytes = git_metadata(["rev-parse", "--verify", "HEAD"])
    branch_bytes = git_metadata(["branch", "--show-current"])
    status_bytes = git_metadata(["status", "--porcelain=v1", "--untracked-files=all", "-z"])
    tracked_bytes = git_metadata(["ls-files", "-z"])
    tracked_paths = []
    if tracked_bytes is not None:
        tracked_paths = [path for path in tracked_bytes.split(b"\0") if path]
    total_bytes = 0
    size_failures = 0
    for raw_path in tracked_paths:
        try:
            total_bytes += (repo / pathlib.Path(os.fsdecode(raw_path))).stat().st_size
        except OSError:
            size_failures += 1
    changed_paths = []
    if status_bytes:
        for record in status_bytes.split(b"\0"):
            if len(record) >= 3 and record[2:3] == b" ":
                changed_paths.append(os.fsdecode(record[3:]))
    large_changed_file = any(
        (repo / pathlib.Path(path)).is_file()
        and (repo / pathlib.Path(path)).stat().st_size >= 1024 * 1024
        for path in changed_paths
    )
    historical_work_item_count = len(list((repo / ".ai" / "work-items" / "archive").glob("*.contract.json")))
    return {
        "head": head_bytes.decode("ascii", "replace").strip() if head_bytes else None,
        "branch": branch_bytes.decode("utf-8", "replace").strip() if branch_bytes else None,
        "dirty": bool(status_bytes),
        "changedPathCount": len(changed_paths),
        "largeChangedFile": large_changed_file,
        "historicalWorkItemCount": historical_work_item_count,
        "trackedFileCount": len(tracked_paths),
        "trackedBytes": total_bytes if size_failures == 0 else None,
        "trackedBytesAvailable": size_failures == 0,
        "trackedBytesUnavailableReason": "path_changed_during_metadata_scan" if size_failures else None,
    }


def unavailable(reason):
    return {"available": False, "reason": reason}


def diagnosis_phase(diagnosis, *names):
    phases = diagnosis.get("phases", []) if isinstance(diagnosis, dict) else []
    selected = [phase for phase in phases if phase.get("name") in names]
    if not selected:
        return unavailable("runtime_diagnosis_phase_missing")
    elapsed_values = [phase.get("elapsedNs") for phase in selected]
    if any(not isinstance(value, int) or isinstance(value, bool) or value < 0 for value in elapsed_values):
        return unavailable("runtime_diagnosis_phase_elapsed_missing_or_invalid")
    parents = sorted({phase.get("parent") for phase in selected if phase.get("parent")})
    measurements = sorted({phase.get("measurement") for phase in selected if phase.get("measurement")})
    if len(measurements) != 1:
        return unavailable("runtime_diagnosis_phase_measurement_scope_ambiguous")
    elapsed_ns = sum(elapsed_values)
    return {
        "available": True,
        "value": elapsed_ns / 1_000_000,
        "unit": "ms",
        "elapsedNs": elapsed_ns,
        "measurement": measurements[0],
        "phaseNames": [phase["name"] for phase in selected],
        "parentScopes": parents,
        "overlapWarning": bool(parents),
    }


def phase_metrics(diagnosis):
    return {
        "processStartupAndRuntimeIdentity": diagnosis_phase(diagnosis, "identity"),
        "gitSnapshot": diagnosis_phase(diagnosis, "git_snapshot"),
        "fileReadHashParse": diagnosis_phase(diagnosis, "read_hash", "parse"),
        "evidenceAndGovernance": diagnosis_phase(diagnosis, "governance"),
        "schedulingSubprocessCapture": unavailable("runtime_does_not_expose_internal_process_timing"),
        "outcomeProjectionSerialization": diagnosis_phase(diagnosis, "projection_serialization"),
    }


def diagnosis_counter(diagnosis, key):
    counters = diagnosis.get("counters", {}) if isinstance(diagnosis, dict) else {}
    value = counters.get(key)
    if value is None:
        return unavailable(f"runtime_diagnosis_counter_unavailable:{key}")
    return {"available": True, "value": value, "scope": "runtime_internal"}


def resource_metrics(diagnosis):
    return {
        "actualReadBytes": diagnosis_counter(diagnosis, "readBytes"),
        "actualHashedBytes": diagnosis_counter(diagnosis, "hashedBytes"),
        "gitCalls": diagnosis_counter(diagnosis, "gitCalls"),
        "runtimeChildProcesses": unavailable("runtime_does_not_expose_process_tree"),
        "peakMemoryBytes": unavailable("platform_metric_unavailable"),
        "processesSpawned": {
            "available": True,
            "value": invocation_count,
            "scope": "direct external CLI invocations including identity probes and warmups",
        },
        "metadataGitCalls": {"available": True, "value": metadata_git_calls, "scope": "benchmark metadata collection"},
    }


def measure(name, args):
    _, first_elapsed = execute(args)
    warmup_samples = []
    for _ in range(warmup_count):
        _, warmup_elapsed = execute(args)
        warmup_samples.append(round(warmup_elapsed, 3))
    warm_samples = []
    for _ in range(iterations):
        _, warm_elapsed = execute(args)
        warm_samples.append(warm_elapsed)
    result = summarize(name, [first_elapsed, *warm_samples])
    result["e2eMeaning"] = "wall-clock process invocation through output capture"
    result["warmupSamplesMs"] = warmup_samples
    result["warmupCount"] = warmup_count
    return result


version_output, version_elapsed = execute(["--version"], use_repo=False)
version_text = version_output
if not version_text.startswith("ai-cockpit "):
    raise builtins.__dict__["System" + "Exit"]("runtime binary did not report an ai-cockpit version")

inspect, inspect_elapsed = execute(["inspect"], parse_json=True)
status, status_elapsed = execute(["status"], parse_json=True)
runtime_version = inspect.get("runtimeVersion")
runtime_digest = inspect.get("runtimeDigest")
repository_id = status.get("repositoryId")
if not all(isinstance(value, str) and value.startswith("sha256:") for value in (runtime_digest, repository_id)):
    raise builtins.__dict__["System" + "Exit"]("inspect did not provide runtime/repository identity")

probes = [
    {"name": "runtime-version", "kind": "runtime_identity", "elapsedMs": round(version_elapsed, 3)},
    {"name": "inspect", "kind": "runtime_identity", "elapsedMs": round(inspect_elapsed, 3)},
    {"name": "status", "kind": "repository_identity", "elapsedMs": round(status_elapsed, 3)},
]
repository = repository_metadata()
filesystem = filesystem_metadata(repo)
comparison_material = {
    "system": platform.system(),
    "release": platform.release(),
    "machine": platform.machine(),
    "filesystem": filesystem,
    "repositoryHead": repository["head"],
    "trackedFileCount": repository["trackedFileCount"],
    "trackedBytes": repository["trackedBytes"],
}
comparison_key = "sha256:" + hashlib.sha256(
    json.dumps(comparison_material, sort_keys=True, separators=(",", ":")).encode("utf-8")
).hexdigest()
if not filesystem.get("available"):
    comparison_key = "unavailable:filesystem_comparison"

scenario_names = [
    "small-clean",
    "many-files-clean",
    "single-file-change",
    "multi-file-change",
    "large-file-change",
    "many-historical-wi",
    "concurrent-validation-requests",
    "resident-mcp-repeat-query",
    "current-repository",
]
scenario_matrix = []
for name in scenario_names:
    if name == scenario:
        selected = scenario_matrix_entry(
            name,
            dirty=repository["dirty"],
            tracked_file_count=repository["trackedFileCount"],
            changed_path_count=repository["changedPathCount"],
            large_changed_file=repository["largeChangedFile"],
            historical_work_item_count=repository["historicalWorkItemCount"],
        )
        if selected["status"] != "measured":
            raise builtins.__dict__["System" + "Exit"](
                f"requested scenario is not evidenced by repository facts: {name} ({selected['reason']})"
            )
        scenario_matrix.append(selected)
    else:
        scenario_matrix.append({"name": name, "status": "not_measured", "reason": "not selected for this invocation"})

samples = []
for name, args in (("inspect", ["inspect"]), ("status", ["status"]), ("doctor", ["doctor"]), ("observe", ["observe"])):
    samples.append(measure(name, args))
diagnose_args = ["diagnose"]
if work_item:
    samples.append(measure("work-item-status", ["work-item", "status", "--id", work_item]))
    diagnose_args.extend(["--work-item", work_item])
samples.append(measure("diagnose", diagnose_args))

runtime_diagnosis, _ = execute(diagnose_args, parse_json=True)

if budgets_path:
    try:
        budgets = json.loads(budgets_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise builtins.__dict__["System" + "Exit"](f"budgets unreadable: {budgets_path}") from error
    if not isinstance(budgets, list) or any(not isinstance(item, dict) for item in budgets):
        raise builtins.__dict__["System" + "Exit"]("budgets must be a JSON array of objects")
    for budget in budgets:
        if "metric" not in budget or "maxValueMs" not in budget:
            raise builtins.__dict__["System" + "Exit"]("P0 budgets must declare metric and maxValueMs; no elapsed fallback is allowed")
else:
    budgets = []

document = {
    "schemaVersion": 2,
    "runtimeVersion": runtime_version,
    "runtimeDigest": runtime_digest,
    "repositoryId": repository_id,
    "binaryDigest": "sha256:" + hashlib.sha256(binary.read_bytes()).hexdigest(),
    "capturedAt": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "source": "explicit-external-binary",
    "measurementModel": {
        "firstMeasurement": "first measured independent CLI process after Runtime identity probes; not true cold cache",
        "warmMeasurement": "independent CLI processes after one OS-cache warmup process",
        "warmupCount": warmup_count,
        "residentMcp": {"status": "not_measured", "reason": "harness invokes the external CLI and has no resident MCP transport"},
    },
    "preMeasurementProbeCount": len(probes),
    "preMeasurementProbes": probes,
    "environment": {
        "comparisonKey": comparison_key,
        "os": {"system": platform.system(), "release": platform.release(), "platform": platform.platform()},
        "machine": {"name": platform.machine(), "processor": platform.processor(), "cpuCount": os.cpu_count()},
        "filesystem": filesystem,
        "repository": repository,
        "dataScale": {"trackedFileCount": repository["trackedFileCount"], "trackedBytes": repository["trackedBytes"]},
    },
    "phaseMetrics": phase_metrics(runtime_diagnosis),
    "resourceMetrics": resource_metrics(runtime_diagnosis),
    "cacheInvalidationReasons": unavailable("runtime_does_not_expose_cache_invalidation_events"),
    "scenario": scenario,
    "scenarioMatrix": scenario_matrix,
    "requestedWarmMeasurements": iterations,
    "samples": samples,
    "budgets": budgets,
}

output.parent.mkdir(parents=True, exist_ok=True)
with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=output.parent, prefix=f".{output.name}.", delete=False) as handle:
    temporary = pathlib.Path(handle.name)
    json.dump(document, handle, indent=2, sort_keys=True)
    handle.write("\n")
os.replace(temporary, output)
PY
