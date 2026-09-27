#!/usr/bin/env bash
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
metadata=""
cargo_bin=cargo
report="$root/target/workspace-package-coverage.json"
hosted_verification_receipt="${AI_COCKPIT_VERIFICATION_RECEIPT:-}"
runtime_bin="${AI_COCKPIT_RUNTIME_BIN:-$root/target/release/ai-cockpit}"
verification_repository="${AI_COCKPIT_VERIFICATION_REPOSITORY:-$root}"
workers="${WORKSPACE_TEST_WORKERS:-2}"
test_threads="${WORKSPACE_TEST_THREADS:-4}"
while (($#)); do
  case "$1" in
    --metadata) metadata=${2:?}; shift 2 ;;
    --cargo) cargo_bin=${2:?}; shift 2 ;;
    --report) report=${2:?}; shift 2 ;;
    *) printf 'unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
done

if [[ ! "$workers" =~ ^[1-9][0-9]*$ ]]; then
  printf 'workspace test worker count must be a positive integer\n' >&2
  exit 2
fi
if [[ ! "$test_threads" =~ ^[1-9][0-9]*$ ]]; then
  printf 'workspace test thread count must be a positive integer\n' >&2
  exit 2
fi

tmp=$(mktemp -d "${TMPDIR:-/tmp}/workspace-packages.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
: >"$tmp/planned"
: >"$tmp/executed"
mkdir -p "$tmp/results"
state=passed
failure_phase=""
failed_package=""
failed_index=""
failed_exit_code=""
failure_diagnostic_tail=""
if [[ -z "$metadata" ]]; then
  metadata="$tmp/metadata.json"
  if ! (cd "$root" && "$cargo_bin" metadata --locked --format-version 1 --no-deps) >"$metadata"; then
    state=failed
    failure_phase=metadata
  fi
fi

if [[ "$state" == passed ]] && ! python3 - "$metadata" >"$tmp/planned" <<'PY'
import json
import sys

metadata = json.load(open(sys.argv[1], encoding="utf-8"))
packages = sorted({item["name"] for item in metadata["packages"] if item.get("source") is None})
if not packages:
    raise SystemExit("cargo metadata contains no workspace packages")
print("\n".join(packages))
PY
then
  state=failed
  failure_phase=metadata
  : >"$tmp/planned"
fi
receipt_mode=false
if [[ "$state" == passed && -n "$hosted_verification_receipt" ]]; then
  receipt_mode=true
  if ! python3 - "$metadata" "$hosted_verification_receipt" "$runtime_bin" "$verification_repository" >"$tmp/executed" 2>"$tmp/receipt-diagnostic" <<'PY'
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

metadata_path, receipt_path, runtime_path, repository_path = map(Path, sys.argv[1:])

def reject(message):
    raise SystemExit(message)

if receipt_path.is_symlink() or runtime_path.is_symlink() or repository_path.is_symlink():
    reject("hosted verification inputs must not be symlinks")
if not receipt_path.is_file() or not runtime_path.is_file() or not repository_path.is_dir():
    reject("hosted verification inputs are missing or not regular files")

metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
expected = sorted({
    package["name"]
    for package in metadata["packages"]
    if package.get("source") is None
})
artifact_bytes = receipt_path.read_bytes()
formal = json.loads(artifact_bytes)
if not isinstance(formal, dict):
    reject("hosted Runtime evidence is not a JSON object")
receipt = formal.get("receipt")
if not isinstance(receipt, dict):
    reject("hosted Runtime evidence is missing its formal receipt envelope")
work_item_id = formal.get("workItemId")
if not isinstance(work_item_id, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}", work_item_id):
    raise SystemExit("hosted Runtime verification receipt has an invalid Work Item identity")
actual_digest = "sha256:" + hashlib.sha256(runtime_path.read_bytes()).hexdigest()
digest_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")
runtime_version = formal.get("runtimeVersion")
repository_id = formal.get("repositoryId")
snapshot_digest = formal.get("repositorySnapshotDigest")
contract_digest = formal.get("contractDigest")
if (
    formal.get("passed") is not True
    or formal.get("runtimeDigest") != actual_digest
    or not isinstance(runtime_version, str)
    or not runtime_version
    or not isinstance(repository_id, str)
    or not digest_pattern.fullmatch(repository_id)
    or not isinstance(snapshot_digest, str)
    or not digest_pattern.fullmatch(snapshot_digest)
    or not isinstance(contract_digest, str)
    or not digest_pattern.fullmatch(contract_digest)
):
    reject("formal hosted Runtime evidence is not passing or has invalid identity bindings")
if (
    receipt.get("workItemId") != work_item_id
    or receipt.get("passed") is not True
    or receipt.get("runtimeDigest") != actual_digest
    or receipt.get("runtimeVersion") != runtime_version
    or receipt.get("repositoryId") != repository_id
):
    reject("nested Runtime receipt identity does not match the formal evidence envelope")

try:
    status_result = subprocess.run(
        [
            str(runtime_path),
            "work-item",
            "status",
            "--repo",
            str(repository_path),
            "--id",
            work_item_id,
            "--json",
        ],
        check=False,
        capture_output=True,
        text=True,
        timeout=30,
    )
except (OSError, subprocess.TimeoutExpired) as error:
    reject(f"candidate Runtime freshness query failed: {error}")
if status_result.returncode != 0:
    reject(
        "candidate Runtime rejected the receipt freshness status query: "
        + status_result.stderr[-2000:]
    )
try:
    current_status = json.loads(status_result.stdout)
except json.JSONDecodeError:
    reject("candidate Runtime returned malformed receipt freshness status")
if (
    current_status.get("workItemId") != work_item_id
    or current_status.get("verification") != "verified"
    or not isinstance(current_status.get("evidenceFreshness"), dict)
    or current_status["evidenceFreshness"].get("state") != "fresh"
):
    reject("candidate Runtime does not accept the formal receipt as fresh")
current_source_digests = current_status.get("sourceDigests")
if not isinstance(current_source_digests, dict):
    reject("candidate Runtime freshness status is missing source identity digests")
if current_source_digests.get("contract") != contract_digest:
    reject("formal receipt Contract digest does not match the current Runtime Contract")
if current_source_digests.get("repositorySnapshot") != snapshot_digest:
    reject("formal receipt source snapshot does not match the current Runtime source snapshot")
if "sha256:" + hashlib.sha256(runtime_path.read_bytes()).hexdigest() != actual_digest:
    reject("candidate Runtime executable changed during freshness validation")

results = receipt.get("results")
nodes_planned = receipt.get("nodesPlanned")
nodes_executed = receipt.get("nodesExecuted")
nodes_reused = receipt.get("nodesReused")
if (
    not isinstance(results, list)
    or not isinstance(nodes_planned, int)
    or isinstance(nodes_planned, bool)
    or not isinstance(nodes_executed, int)
    or isinstance(nodes_executed, bool)
    or not isinstance(nodes_reused, int)
    or isinstance(nodes_reused, bool)
    or nodes_planned != len(results)
    or nodes_executed + nodes_reused != nodes_planned
):
    reject("hosted Runtime receipt has incomplete or inconsistent node accounting")
plan = receipt.get("planReceipt")
if (
    not isinstance(plan, dict)
    or plan.get("workItemId") != work_item_id
    or plan.get("repositoryId") != repository_id
    or plan.get("repositorySnapshotDigest") != snapshot_digest
):
    reject("hosted Runtime plan receipt is not bound to the formal evidence identity")
coverage = plan.get("coverageManifest")
expected_nodes = [f"project-command-0-package-{package}" for package in expected]
if (
    not isinstance(coverage, dict)
    or sorted(coverage.get("workspaceMembers", [])) != expected
    or sorted(coverage.get("nodeIds", [])) != expected_nodes
):
    reject("hosted Runtime plan does not bind the complete Cargo workspace package set")
all_node_ids = []
covered = []
for result in results:
    node_id = result.get("nodeId") if isinstance(result, dict) else None
    if not isinstance(node_id, str) or not node_id:
        reject("hosted Runtime receipt contains an invalid verification node identity")
    if result.get("passed") is not True:
        reject(f"hosted Runtime verification node failed: {node_id}")
    all_node_ids.append(node_id)
    if not node_id.startswith("project-command-0-package-"):
        continue
    package = node_id.removeprefix("project-command-0-package-")
    covered.append(package)
if len(set(all_node_ids)) != len(all_node_ids):
    reject("hosted Runtime receipt contains duplicate verification nodes")
if sorted(covered) != expected or len(set(covered)) != len(expected):
    reject("hosted Runtime receipt package set does not match Cargo workspace metadata")

evidence_directory = repository_path / ".ai" / "evidence"
evidence_path = evidence_directory / f"{work_item_id}.verification.json"
if (
    (repository_path / ".ai").is_symlink()
    or evidence_directory.is_symlink()
    or evidence_path.is_symlink()
    or not evidence_path.is_file()
):
    reject("Runtime-authored formal verification evidence is missing or unsafe")
evidence_bytes = evidence_path.read_bytes()
if artifact_bytes != evidence_bytes:
    reject("coverage evidence is not byte-identical to Runtime-authored formal evidence")
evidence = json.loads(evidence_bytes)
if (
    evidence.get("workItemId") != work_item_id
    or evidence.get("passed") is not True
    or evidence.get("runtimeDigest") != actual_digest
    or evidence.get("repositoryId") != repository_id
    or evidence.get("repositorySnapshotDigest") != snapshot_digest
):
    reject("Runtime-authored formal verification evidence identity is invalid")
print("\n".join(expected))
PY
  then
    state=failed
    failure_phase=hosted_verification_receipt
    failure_diagnostic_tail=$(tail -c 12000 "$tmp/receipt-diagnostic")
    : >"$tmp/executed"
  fi
elif [[ "$state" == passed ]]; then
  packages=()
  while IFS= read -r package; do
    packages+=("$package")
  done <"$tmp/planned"
  pids=()
  active=0
  next=0
  package_count=${#packages[@]}
  while { [[ "$state" == passed ]] && ((next < package_count)) || ((active > 0)); }; do
    while [[ "$state" == passed ]] && ((next < package_count && active < workers)); do
      package=${packages[$next]}
      index=$next
      (
        set +e
        (cd "$root" && "$cargo_bin" test -p "$package" --all-targets -- --test-threads="$test_threads") \
          >"$tmp/results/$index.log" 2>&1
        result=$?
        printf '%s\n' "$result" >"$tmp/results/$index.status"
        exit 0
      ) &
      pids[$index]=$!
      ((next += 1))
      ((active += 1))
    done

    for index in "${!pids[@]}"; do
      if [[ -f "$tmp/results/$index.status" ]]; then
        wait "${pids[$index]}" 2>/dev/null
        result=$(<"$tmp/results/$index.status")
        if [[ "$result" != 0 && "$state" == passed ]]; then
          state=failed
          failure_phase=package_test
          failed_package=${packages[$index]}
          failed_index=$index
          failed_exit_code=$result
        fi
        unset 'pids[index]'
        ((active -= 1))
      fi
    done
    ((active > 0)) && sleep 0.05
  done
fi

if [[ "$state" != passed && -n "$failed_index" ]]; then
  cat "$tmp/results/$failed_index.log" >&2
  failure_diagnostic_tail=$(tail -n 80 "$tmp/results/$failed_index.log" | tail -c 12000)
fi

# Completion order is intentionally independent from the worker schedule so
# the report remains deterministic and can be compared across CI runs.
if [[ "$receipt_mode" != true ]]; then
  : >"$tmp/executed"
  if [[ "$state" == passed || -n "$failed_index" ]]; then
    for index in "${!packages[@]}"; do
      if [[ -f "$tmp/results/$index.status" ]] && [[ "$(<"$tmp/results/$index.status")" == 0 ]]; then
        printf '%s\n' "${packages[$index]}" >>"$tmp/executed"
      fi
    done
  fi
fi

mkdir -p "$(dirname "$report")"
python3 - "$tmp/planned" "$tmp/executed" "$report" "$state" "$failure_phase" "$failed_package" "$failed_exit_code" "$failure_diagnostic_tail" <<'PY'
import json
import sys

def lines(path):
    return [line for line in open(path, encoding="utf-8").read().splitlines() if line]

planned = lines(sys.argv[1])
executed = lines(sys.argv[2])
report = {
    "executed": executed,
    "omitted": sorted(set(planned) - set(executed)),
    "planned": planned,
    "schemaVersion": 1,
    "state": sys.argv[4],
}
if sys.argv[5]:
    report["failurePhase"] = sys.argv[5]
if sys.argv[6]:
    report["failedPackage"] = sys.argv[6]
if sys.argv[7]:
    report["failedExitCode"] = int(sys.argv[7])
if sys.argv[8]:
    report["failureDiagnosticTail"] = sys.argv[8]
open(sys.argv[3], "w", encoding="utf-8").write(json.dumps(report, indent=2, sort_keys=True) + "\n")
PY

if [[ "$state" != passed ]]; then
  printf 'workspace package coverage failed: package=%s exitCode=%s\n' "$failed_package" "$failed_exit_code" >&2
  exit 1
fi
printf 'workspace package coverage passed: %s packages\n' "$(wc -l <"$tmp/executed" | tr -d ' ')"
