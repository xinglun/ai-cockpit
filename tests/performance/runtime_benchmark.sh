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

exec python3 - "$binary" "$repo" "$output" "$iterations" "$work_item" "$budgets" "$script_dir" <<'PY'
import datetime as dt
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import tempfile
import time

binary = pathlib.Path(sys.argv[1])
repo = pathlib.Path(sys.argv[2])
output = pathlib.Path(sys.argv[3])
iterations = int(sys.argv[4])
work_item = sys.argv[5] or None
budgets_path = pathlib.Path(sys.argv[6]) if sys.argv[6] else None
script_dir = sys.argv[7]

sys.path.insert(0, script_dir)
from runtime_benchmark_stats import summarize

probe_log = []

def call(args, parse_json=False):
    try:
        result = subprocess.run(
            [str(binary), *args, "--repo", str(repo)],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=300,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise SystemExit(f"benchmark command failed: {args[0]} ({type(error).__name__})")
    if result.returncode != 0:
        raise SystemExit(f"benchmark command failed: {args[0]} (exit {result.returncode})")
    probe_log.append(args[0])
    if parse_json:
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError as error:
            raise SystemExit(f"benchmark command returned invalid JSON: {args[0]}") from error
    return None

version = subprocess.run(
    [str(binary), "--version"],
    stdin=subprocess.DEVNULL,
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    check=False,
    timeout=30,
)
if version.returncode != 0 or not version.stdout.decode("utf-8", "replace").strip().startswith("ai-cockpit "):
    raise SystemExit("runtime binary did not report an ai-cockpit version")
probe_log.append("--version")

inspect = call(["inspect"], parse_json=True)
runtime_version = inspect.get("runtimeVersion")
runtime_digest = inspect.get("runtimeDigest")
status = call(["status"], parse_json=True)
repository_id = status.get("repositoryId")
if not all(isinstance(value, str) and value.startswith("sha256:") for value in (runtime_digest, repository_id)):
    raise SystemExit("inspect did not provide runtime/repository identity")

def measure(name, args):
    prior_probe_calls = probe_log.count(args[0])
    raw = []
    for _ in range(iterations + 1):
        started = time.perf_counter_ns()
        call(args)
        raw.append(round((time.perf_counter_ns() - started) / 1_000_000, 3))
    return summarize(name, raw, prior_probe_calls=prior_probe_calls)

samples = []
for name, args in (("inspect", ["inspect"]), ("status", ["status"]), ("doctor", ["doctor"]), ("observe", ["observe"])):
    samples.extend(measure(name, args))
if work_item:
    samples.extend(measure("work-item-status", ["work-item", "status", "--id", work_item]))
    samples.extend(measure("diagnose", ["diagnose", "--work-item", work_item]))

if budgets_path:
    try:
        budgets = json.loads(budgets_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"budgets unreadable: {budgets_path}") from error
    if not isinstance(budgets, list) or any(not isinstance(item, dict) for item in budgets):
        raise SystemExit("budgets must be a JSON array of objects")
else:
    budgets = []

def git_capture(args):
    try:
        result = subprocess.run(
            ["git", *args],
            cwd=repo,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=30,
        )
    except OSError:
        return None
    if result.returncode != 0:
        return None
    return result.stdout.decode("utf-8", "replace").strip()

def tracked_file_count():
    tracked = git_capture(["ls-files"])
    if tracked is None:
        return "unavailable"
    return len([line for line in tracked.splitlines() if line])

def filesystem_kind(path):
    if shutil.which("df") is None:
        return "unavailable"
    args = ["df", "-T", str(path)] if platform.system() == "Linux" else ["df", str(path)]
    try:
        result = subprocess.run(
            args, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, check=False, timeout=10
        )
    except OSError:
        return "unavailable"
    if result.returncode != 0:
        return "unavailable"
    return result.stdout.decode("utf-8", "replace").strip()

dirty_status = git_capture(["status", "--porcelain"])
environment = {
    "hardware": {
        "machine": platform.machine(),
        "processorCount": os.cpu_count() or "unavailable",
    },
    "os": {
        "system": platform.system(),
        "release": platform.release(),
        "version": platform.version(),
    },
    "filesystem": filesystem_kind(repo),
    "repositoryState": {
        "repositoryId": repository_id,
        "head": git_capture(["rev-parse", "HEAD"]) or "unavailable",
        "branch": git_capture(["rev-parse", "--abbrev-ref", "HEAD"]) or "unavailable",
        "dirty": "unavailable" if dirty_status is None else bool(dirty_status),
    },
    "dataScale": {"trackedFileCount": tracked_file_count()},
    "preMeasurementProcessInvocations": probe_log,
}

document = {
    "schemaVersion": 1,
    "runtimeVersion": runtime_version,
    "runtimeDigest": runtime_digest,
    "repositoryId": repository_id,
    "binaryDigest": "sha256:" + hashlib.sha256(binary.read_bytes()).hexdigest(),
    "platform": {"system": platform.system(), "machine": platform.machine()},
    "capturedAt": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "source": "explicit-external-binary",
    "measurementModel": (
        "independent-cli-process-per-call; this harness does not measure "
        "persistent MCP session latency"
    ),
    "iterations": iterations,
    "samples": samples,
    "environment": environment,
    "budgets": budgets,
}

output.parent.mkdir(parents=True, exist_ok=True)
with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=output.parent, prefix=f".{output.name}.", delete=False) as handle:
    temporary = pathlib.Path(handle.name)
    json.dump(document, handle, indent=2, sort_keys=True)
    handle.write("\n")
os.replace(temporary, output)
PY
