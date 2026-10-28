#!/usr/bin/env bash
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
metadata=""
cargo_bin=cargo
report="$root/target/workspace-package-coverage.json"
hosted_verification_receipt="${AI_COCKPIT_VERIFICATION_RECEIPT:-}"
hosted_verification_orchestration="${AI_COCKPIT_VERIFICATION_ORCHESTRATION:-}"
runtime_bin="${AI_COCKPIT_RUNTIME_BIN:-$root/target/release/ai-cockpit}"
verification_repository="${AI_COCKPIT_VERIFICATION_REPOSITORY:-$root}"
workers="${WORKSPACE_TEST_WORKERS:-2}"
test_threads="${WORKSPACE_TEST_THREADS:-1}"
group_timeout_seconds="${WORKSPACE_TEST_TIMEOUT_SECONDS:-900}"
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
if [[ ! "$group_timeout_seconds" =~ ^[1-9][0-9]{0,2}$ ]] || ((group_timeout_seconds > 900)); then
  printf 'workspace group timeout must be an integer from 1 to 900 seconds\n' >&2
  exit 2
fi

tmp=$(mktemp -d "${TMPDIR:-/tmp}/workspace-packages.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
: >"$tmp/planned"
: >"$tmp/executed"
: >"$tmp/coverage-required"
: >"$tmp/hosted-runtime-executed"
: >"$tmp/coverage-run-executed"
mkdir -p "$tmp/results"
printf '{"groups":[],"inventory":[]}\n' >"$tmp/group-plan.json"
: >"$tmp/groups"
# Each group has its own process-level child set. GNU timeout may itself die
# on KILL; this owner survives outside that group and reaps adopted exits.
cat >"$tmp/group-owner.py" <<'PY'
import ctypes
import errno
import os
import signal
import subprocess
import sys
import time

if sys.platform != "linux":
    raise SystemExit("fresh grouped coverage requires Linux subreaper supervision")
libc = ctypes.CDLL(None, use_errno=True)
enabled = ctypes.c_int()
if (libc.prctl(36, 1, 0, 0, 0) != 0 or libc.prctl(37, ctypes.byref(enabled), 0, 0, 0) != 0
        or enabled.value != 1):
    raise SystemExit("cannot enable and verify the target-group subreaper")
child = None
interrupted = 0

def forward(signum, frame):
    global interrupted
    interrupted = signum
    if child is not None and child.poll() is None:
        try:
            child.send_signal(signum)
        except ProcessLookupError:
            pass

def kill_held_group(process):
    # The handler can poll/reap the direct child. Mask it only for this short
    # ownership check + signal, so the held leader PID cannot be recycled.
    previous_mask = signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGTERM, signal.SIGINT})
    try:
        if process.returncode is not None or process.pid <= 1 or process.pid == os.getpgrp():
            return False
        # WNOWAIT proves this is still our child without consuming its status.
        os.waitid(os.P_PID, process.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
        if os.getpgid(process.pid) != process.pid or os.getsid(process.pid) != process.pid:
            return False
        os.killpg(process.pid, signal.SIGKILL)
        return True
    except OSError:
        return False
    finally:
        signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)

signal.signal(signal.SIGTERM, forward)
signal.signal(signal.SIGINT, forward)
if interrupted:
    raise SystemExit(128 + interrupted)
try:
    child = subprocess.Popen(sys.argv[2:], start_new_session=True)
except OSError as error:
    raise SystemExit("cannot start target group: " + str(error))
if interrupted and child.poll() is None:
    child.send_signal(interrupted)
watchdog_failed = False
try:
    code = child.wait(timeout=int(sys.argv[1]) + 31)
except subprocess.TimeoutExpired:
    watchdog_failed = True
    if not kill_held_group(child):
        print("target-group cleanup uncertain: cannot establish held command-group ownership", file=sys.stderr)
    try:
        code = child.wait(timeout=30)
    except subprocess.TimeoutExpired:
        raise SystemExit("target-group cleanup uncertain: direct child did not exit")
# Direct-child status belongs to Popen until this point. Only this dedicated
# group owner uses waitpid(-1), after Popen has consumed that status.
def reap_group_children(deadline):
    while time.monotonic() < deadline:
        try:
            pid, status = os.waitpid(-1, os.WNOHANG)
        except ChildProcessError:
            return
        except OSError as error:
            if error.errno == errno.EINTR:
                continue
            raise SystemExit("target-group cleanup uncertain: waitpid failed")
        if pid > 0:
            print(f"target-group reaped owned pid={pid} status={status}", file=sys.stderr)
        else:
            time.sleep(0.01)
    raise SystemExit("target-group cleanup uncertain: reap deadline exceeded")

reap_group_children(time.monotonic() + 30)
if watchdog_failed:
    raise SystemExit("target-group cleanup uncertain: timeout command exceeded its deadline")
if interrupted:
    raise SystemExit(128 + interrupted)
raise SystemExit(code if code >= 0 else 128 - code)
PY
state=passed
receipt_verification_state=not_provided
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
  if ! python3 - "$metadata" "$hosted_verification_receipt" "$runtime_bin" "$verification_repository" "$hosted_verification_orchestration" "$root/tests/ci" >"$tmp/receipt-plan.json" 2>"$tmp/receipt-diagnostic" <<'PY'
import codecs
import hashlib
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, sys.argv[6])
from run_no_warnings import warning_diagnostics

metadata_path, receipt_path, runtime_path, repository_path = map(Path, sys.argv[1:5])
orchestration_path = Path(sys.argv[5]) if sys.argv[5] else None

def reject(message):
    raise SystemExit(message)

if receipt_path.is_symlink() or runtime_path.is_symlink() or repository_path.is_symlink():
    reject("hosted verification inputs must not be symlinks")
if not receipt_path.is_file() or not runtime_path.is_file() or not repository_path.is_dir():
    reject("hosted verification inputs are missing or not regular files")
if orchestration_path is None or orchestration_path.is_symlink() or not orchestration_path.is_file():
    reject("hosted Runtime orchestration evidence is missing or unsafe")

metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
expected = sorted({
    package["name"]
    for package in metadata["packages"]
    if package.get("source") is None
})
artifact_bytes = receipt_path.read_bytes()
formal = json.loads(artifact_bytes)
orchestration = json.loads(orchestration_path.read_bytes())
if not isinstance(formal, dict):
    reject("hosted Runtime evidence is not a JSON object")
if not isinstance(orchestration, dict):
    reject("hosted Runtime orchestration evidence is not a JSON object")
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
verification_state = orchestration.get("verificationState")
if (
    orchestration.get("schemaVersion") != 1
    or orchestration.get("workItemId") != work_item_id
    or orchestration.get("runtimeDigest") != actual_digest
    or orchestration.get("formalReceiptDigest")
    != "sha256:" + hashlib.sha256(artifact_bytes).hexdigest()
    or orchestration.get("executionRepository") != str(repository_path.resolve())
    or verification_state not in {"passed", "reused"}
):
    reject("hosted Runtime orchestration does not bind this receipt and execution repository")

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
contract_directory = repository_path / ".ai" / "work-items" / "active"
contract_path = contract_directory / f"{work_item_id}.contract.json"
if (
    (repository_path / ".ai").is_symlink()
    or (repository_path / ".ai/work-items").is_symlink()
    or contract_directory.is_symlink()
    or contract_path.is_symlink()
    or not contract_path.is_file()
):
    reject("current Runtime-bound Contract is missing or unsafe")
contract = json.loads(contract_path.read_text(encoding="utf-8"))
observed_contract_digest = "sha256:" + hashlib.sha256(
    json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
).hexdigest()
if observed_contract_digest != contract_digest:
    reject("current Contract bytes do not match the Runtime-bound Contract digest")
declarations = contract.get("verification") if isinstance(contract, dict) else None
if contract.get("workItemId") != work_item_id or not isinstance(declarations, list):
    reject("current Runtime-bound Contract has no verification declaration")
all_node_ids = []
result_by_id = {}
for result in results:
    node_id = result.get("nodeId") if isinstance(result, dict) else None
    if not isinstance(node_id, str) or not node_id:
        reject("hosted Runtime receipt contains an invalid verification node identity")
    if result.get("passed") is not True:
        reject(f"hosted Runtime verification node failed: {node_id}")
    all_node_ids.append(node_id)
    result_by_id[node_id] = result
if len(set(all_node_ids)) != len(all_node_ids):
    reject("hosted Runtime receipt contains duplicate verification nodes")
reused_nodes = plan.get("reusedNodes")
executed_nodes = plan.get("executedNodes")
if (
    not isinstance(reused_nodes, list)
    or any(not isinstance(node, str) for node in reused_nodes)
    or len(set(reused_nodes)) != len(reused_nodes)
    or not isinstance(executed_nodes, list)
    or any(not isinstance(node, str) for node in executed_nodes)
    or len(set(executed_nodes)) != len(executed_nodes)
):
    reject("hosted Runtime plan is missing valid node accounting")
if (
    set(reused_nodes).intersection(executed_nodes)
    or sorted(reused_nodes + executed_nodes) != sorted(all_node_ids)
    or len(executed_nodes) != nodes_executed
    or len(reused_nodes) != nodes_reused
):
    reject("hosted Runtime plan execution accounting does not match its formal receipt")
records = receipt.get("executionRecords")
if not isinstance(records, list):
    reject("hosted Runtime receipt is missing execution records")
record_by_id = {}
for record in records:
    if not isinstance(record, dict) or not isinstance(record.get("nodeId"), str):
        reject("hosted Runtime receipt contains an invalid execution record")
    if record["nodeId"] in record_by_id:
        reject("hosted Runtime receipt contains duplicate execution records")
    record_by_id[record["nodeId"]] = record
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

def unfiltered_serial_test_args(args):
    if not isinstance(args, list) or any(not isinstance(arg, str) or not arg for arg in args):
        return None
    if "--" in args:
        separator = args.index("--")
        if args[separator + 1:] not in (["--test-threads=1"], ["--test-threads", "1"]):
            return None
        return args[:separator]
    return args

def complete_workspace_test(program, args):
    args = unfiltered_serial_test_args(args)
    return (
        program == "cargo"
        and isinstance(args, list)
        and bool(args)
        and args[0] == "test"
        and "--workspace" in args[1:]
        and all(arg in {"--workspace", "--locked", "--all-features", "--all-targets"} for arg in args[1:])
        and len(args[1:]) == len(set(args[1:]))
    )

def additional_targeted_test(program, args):
    args = unfiltered_serial_test_args(args)
    if program != "cargo" or not args or args[0] != "test":
        return False
    package = None
    targets = set()
    locked = False
    index = 1
    while index < len(args):
        arg = args[index]
        if arg == "--locked" and not locked:
            locked = True
            index += 1
        elif arg in {"-p", "--package"} and package is None and index + 1 < len(args):
            package = args[index + 1]
            if package not in expected:
                return False
            index += 2
        elif arg == "--test" and index + 1 < len(args):
            target = args[index + 1]
            if not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]*", target) or target in targets:
                return False
            targets.add(target)
            index += 2
        else:
            return False
    return package is not None and bool(targets)

coverage = plan.get("coverageManifest")
if coverage is None:
    # A passing targeted plan is not a complete-package coverage proof. Only
    # after the common native identity/provenance/accounting checks above may
    # this route request genuine fresh execution of every metadata package.
    def ordinary_targeted_test(words):
        args = unfiltered_serial_test_args(words[1:])
        if words[:2] != ["cargo", "test"] or not args:
            return False
        package = None
        flags = set()
        targets = set()
        index = 1
        while index < len(args):
            arg = args[index]
            if arg in {"--locked", "--all-features", "--all-targets", "--lib"} and arg not in flags:
                flags.add(arg)
                index += 1
            elif arg in {"-p", "--package"} and package is None and index + 1 < len(args):
                package = args[index + 1]
                if package not in expected:
                    return False
                index += 2
            elif arg in {"--test", "--bin"} and index + 1 < len(args):
                target = (arg, args[index + 1])
                if not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]*", target[1]) or target in targets:
                    return False
                targets.add(target)
                index += 2
            else:
                return False
        return package is not None and not ("--all-targets" in flags and (targets or "--lib" in flags))

    targeted_nodes = []
    for index, declaration in enumerate(declarations):
        source = declaration if isinstance(declaration, str) else declaration.get("check") if isinstance(declaration, dict) else None
        if not isinstance(source, str):
            reject("current Contract has an unsupported verification declaration")
        try:
            words = shlex.split(source)
        except ValueError:
            reject("current Contract has malformed verification command quoting")
        # Use the same conservative env grammar as the manifest branch.
        assignment_count = 0
        for word in words:
            if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*=.+", word):
                break
            assignment_count += 1
        command = words[assignment_count:]
        explicit_env = command[:1] == ["env"]
        if explicit_env:
            command = command[1:]
            while command and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*=.+", command[0]):
                command = command[1:]
            if not command or command[0] == "env" or command[0].startswith("-"):
                reject("current Contract has unsupported env command syntax")
        if command[:2] != ["cargo", "test"]:
            continue
        if assignment_count or explicit_env or not ordinary_targeted_test(words):
            reject("unmanifested Runtime plan has a filtered, workspace, or unsupported Cargo test")
        node = f"project-command-{index}"
        targeted_nodes.append(node)
        result = result_by_id.get(node)
        if not isinstance(result, dict) or any(other.startswith(node + "-package-") for other in all_node_ids):
            reject("unmanifested Runtime plan omits an ordinary targeted test result")
        if node in reused_nodes:
            if (result.get("reused") is not True or result.get("action") != "reuse"
                    or result.get("satisfiedBy") != "reused_receipt" or node in record_by_id):
                reject("unmanifested Runtime plan has inconsistent reused targeted evidence")
        else:
            record = record_by_id.get(node)
            if (result.get("reused") is not False or result.get("action") != "execute"
                    or result.get("satisfiedBy") != "execution" or not isinstance(record, dict)
                    or not isinstance(record.get("commandDigest"), str)
                    or not digest_pattern.fullmatch(record["commandDigest"])
                    or record.get("spawned") is not True or record.get("passed") is not True
                    or record.get("exitCode") != 0 or record.get("timedOut") is True):
                reject("unmanifested Runtime plan has inconsistent executed targeted evidence")
    if not targeted_nodes:
        reject("unmanifested Runtime plan has no supported ordinary targeted test")
    print(json.dumps({"coverageRequired": expected, "hostedRuntimeExecuted": []}))
    raise SystemExit(0)
if not isinstance(coverage, dict) or coverage.get("workspaceMembers") != expected:
    reject("hosted Runtime plan does not bind the complete Cargo workspace package set")
node_ids = coverage.get("nodeIds")
command_digests = coverage.get("commandDigests")
if (
    not isinstance(node_ids, list)
    or not isinstance(command_digests, list)
    or len(node_ids) != len(expected)
    or len(command_digests) != len(expected)
    or any(not isinstance(digest, str) or not digest_pattern.fullmatch(digest) for digest in command_digests)
):
    reject("hosted Runtime test manifest has invalid package command bindings")
indices = set()
for package, node in zip(expected, node_ids):
    if not isinstance(node, str):
        reject("hosted Runtime test manifest has a non-string package node")
    match = re.fullmatch(r"project-command-(0|[1-9][0-9]*)-package-(.+)", node)
    if not match or match.group(2) != package:
        reject("hosted Runtime test manifest package node does not match metadata")
    indices.add(int(match.group(1)))
if len(indices) != 1:
    reject("hosted Runtime test manifest mixes command indices")
command_index = indices.pop()
if command_index >= len(declarations):
    reject("hosted Runtime test command index is outside the current Contract")
expected_nodes = [f"project-command-{command_index}-package-{package}" for package in expected]
if node_ids != expected_nodes:
    reject("hosted Runtime test manifest is missing or duplicating packages")

if not complete_workspace_test(coverage.get("sourceProgram"), coverage.get("sourceArgs")):
    reject("hosted Runtime manifest is not a complete unfiltered Cargo workspace test")
test_declarations = []
targeted_declarations = []
assignment_name = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
for index, declaration in enumerate(declarations):
    source = declaration if isinstance(declaration, str) else declaration.get("check") if isinstance(declaration, dict) else None
    if not isinstance(source, str):
        reject("current Contract has an unsupported verification declaration")
    try:
        words = shlex.split(source)
    except ValueError:
        reject("current Contract has malformed verification command quoting")
    assignment_count = 0
    for word in words:
        name, separator, value = word.partition("=")
        if not separator or not value or not assignment_name.fullmatch(name):
            break
        assignment_count += 1
    command_words = words[assignment_count:]
    explicit_env = command_words[:1] == ["env"]
    if explicit_env:
        command_words = command_words[1:]
        while command_words:
            name, separator, value = command_words[0].partition("=")
            if not separator or not value or not assignment_name.fullmatch(name):
                break
            command_words = command_words[1:]
        if not command_words or command_words[0] == "env" or command_words[0].startswith("-"):
            reject("current Contract has unsupported env command syntax")
    if command_words[:2] == ["cargo", "test"]:
        if assignment_count or explicit_env:
            reject("current Contract declares an unsupported environment-wrapped Cargo test")
        if complete_workspace_test(words[0], words[1:]):
            test_declarations.append(index)
            if index == command_index and words[1:] != coverage["sourceArgs"]:
                reject("hosted Runtime test manifest differs from the indexed Contract command")
        elif additional_targeted_test(words[0], words[1:]):
            targeted_declarations.append(index)
        else:
            reject("current Contract declares a filtered or unsupported Cargo test")
if test_declarations != [command_index]:
    reject("hosted Runtime test manifest has no unique matching Contract declaration")
if not set(expected_nodes).issubset(result_by_id):
    reject("hosted Runtime receipt package set does not match Cargo workspace metadata")
if any(f"project-command-{index}" not in result_by_id for index in targeted_declarations):
    reject("hosted Runtime receipt omits an additional targeted test result")
if any(
    re.fullmatch(rf"project-command-{command_index}-package-.+", node)
    and node not in expected_nodes
    for node in all_node_ids
):
    reject("hosted Runtime receipt contains an undeclared package test node")
warning_capture_required = set()
for node, digest in zip(expected_nodes, command_digests):
    result = result_by_id[node]
    if node in reused_nodes:
        if (result.get("reused") is not True or result.get("action") != "reuse"
                or result.get("satisfiedBy") != "reused_receipt" or node in record_by_id):
            reject(f"hosted Runtime reused package has inconsistent evidence: {node}")
    else:
        record = record_by_id.get(node)
        if (result.get("reused") is not False or result.get("action") != "execute"
                or result.get("satisfiedBy") != "execution" or not isinstance(record, dict)
                or record.get("commandDigest") != digest or record.get("spawned") is not True
                or record.get("passed") is not True or record.get("exitCode") != 0):
            reject(f"hosted Runtime executed package digest or result is inconsistent: {node}")
        # Executed package coverage and complete warning evidence are
        # independent. Preserve native records; incomplete diagnostics require
        # one separate full-stream quality execution for this package only.
        package = node.removeprefix(f"project-command-{command_index}-package-")
        for stream in ["stdout", "stderr"]:
            if stream not in record:
                warning_capture_required.add(package)
                continue
            raw = record[stream]
            if not isinstance(raw, str) or re.fullmatch(r"(?:[0-9a-f]{2})*", raw) is None:
                reject(f"hosted Runtime package diagnostic hex is invalid: {node}")
            flag = record.get(stream + "Truncated")
            if stream + "Truncated" in record and type(flag) is not bool:
                reject(f"hosted Runtime package capture flag is invalid: {node}")
            try:
                text = codecs.getincrementaldecoder("utf-8")("strict").decode(bytes.fromhex(raw), final=flag is not True)
            except UnicodeError:
                reject(f"hosted Runtime package diagnostic encoding is invalid: {node}")
            if warning_diagnostics(text):
                reject(f"hosted Runtime package emitted a warning: {node}")
            if flag is not False:
                warning_capture_required.add(package)
if verification_state == "reused":
    hosted_runtime_executed = []
    coverage_required = expected
else:
    package_node_ids = set(expected_nodes)
    reused_packages = {
        node.removeprefix(f"project-command-{command_index}-package-")
        for node in reused_nodes
        if node in package_node_ids
    }
    if not package_node_ids.issubset(set(all_node_ids)):
        reject("hosted Runtime receipt omits a required package result")
    hosted_runtime_executed = [package for package in expected if package not in reused_packages]
    coverage_required = [package for package in expected if package in reused_packages or package in warning_capture_required]

print(json.dumps({"coverageRequired": coverage_required, "hostedRuntimeExecuted": hosted_runtime_executed}))
PY
  then
    state=failed
    failure_phase=hosted_verification_receipt
    failure_diagnostic_tail=$(tail -c 12000 "$tmp/receipt-diagnostic")
    : >"$tmp/coverage-required"
    : >"$tmp/hosted-runtime-executed"
  else
    receipt_verification_state=$(jq -er '.verificationState' "$hosted_verification_orchestration") || {
      state=failed
      failure_phase=hosted_verification_receipt
      failure_diagnostic_tail="could not read hosted Runtime verification state"
    }
    jq -r '.coverageRequired[]' "$tmp/receipt-plan.json" >"$tmp/coverage-required"
    jq -r '.hostedRuntimeExecuted[]' "$tmp/receipt-plan.json" >"$tmp/hosted-runtime-executed"
  fi
elif [[ "$state" == passed ]]; then
  cp "$tmp/planned" "$tmp/coverage-required"
fi

if [[ "$state" == passed ]]; then
  packages=()
  while IFS= read -r package; do
    packages+=("$package")
  done <"$tmp/coverage-required"
  package_count=${#packages[@]}
  if ((package_count > 0)); then
    if ! python3 - "$metadata" "$tmp/coverage-required" "$tmp/group-plan.json" \
      >"$tmp/groups" 2>"$tmp/group-diagnostic" <<'PY'
import hashlib
import json
import re
import sys
import tomllib
from pathlib import Path

metadata_path, required_path, plan_path = map(Path, sys.argv[1:])
raw = metadata_path.read_bytes()
metadata = json.loads(raw)
required = required_path.read_text().splitlines()
packages = {}
for package in metadata["packages"]:
    if package.get("source") is not None:
        continue
    name = package.get("name")
    if not isinstance(name, str) or not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_-]*", name) or name in packages:
        raise SystemExit("metadata has invalid or duplicate workspace package names")
    packages[name] = package
groups = []
inventory = []
lib_kinds = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"}
rank = {"lib": 0, "bin": 1, "test": 2, "example": 3, "bench": 4}

def bench_enabled(package, kind, name, tested):
    # Metadata exposes `test`, but not `bench`. For test=false integration /
    # bench targets, retain Cargo's union of tested and benched targets.
    path = Path(package.get("manifest_path", ""))
    if path.name != "Cargo.toml" or path.is_symlink() or not path.is_file():
        raise SystemExit("test=false target requires a regular Cargo.toml to resolve bench selection")
    try:
        manifest = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        raise SystemExit("cannot inspect target bench selection in Cargo.toml")
    entries = manifest.get(kind, [])
    if not isinstance(entries, list) or any(not isinstance(e, dict) or "name" not in e for e in entries):
        raise SystemExit("target bench selection has unsupported unnamed or malformed declarations")
    matches = [e for e in entries if e["name"] == name]
    if len(matches) > 1:
        raise SystemExit("target bench selection has duplicate declarations")
    entry = matches[0] if matches else {}
    manifest_test = entry.get("test", kind == "test")
    bench = entry.get("bench", kind == "bench")
    if type(manifest_test) is not bool or manifest_test != tested or type(bench) is not bool:
        raise SystemExit("target test/bench flags disagree with metadata or are malformed")
    return bench

for package_name in required:
    package = packages.get(package_name)
    if package is None or not isinstance(package.get("targets"), list) or not package["targets"]:
        raise SystemExit("fresh coverage requires the complete metadata target inventory")
    package_groups = []
    seen = set()
    for target in package["targets"]:
        if not isinstance(target, dict):
            raise SystemExit("metadata target is malformed")
        kinds = target.get("kind")
        name = target.get("name")
        tested = target.get("test")
        if (not isinstance(kinds, list) or not kinds or any(not isinstance(k, str) for k in kinds)
                or len(kinds) != len(set(kinds)) or not isinstance(name, str)
                or not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]*", name) or type(tested) is not bool):
            raise SystemExit("metadata target kind/name/test fields are invalid")
        if set(kinds).issubset(lib_kinds):
            kind = "lib"
        elif len(kinds) == 1 and kinds[0] in {"bin", "test", "example", "bench", "custom-build"}:
            kind = kinds[0]
        else:
            raise SystemExit("metadata has an unsupported target kind")
        key = (kind, name)
        if key in seen or (kind == "lib" and any(k == "lib" for k, _ in seen)):
            raise SystemExit("metadata target inventory has duplicate targets")
        seen.add(key)
        features = target.get("required-features", [])
        if not isinstance(features, list) or features:
            raise SystemExit("required-feature target selection cannot be proven from no-deps metadata")
        selected = kind != "custom-build"
        reason = "explicit_all_targets_selection" if selected else "cargo_build_dependency"
        if kind in {"test", "bench"} and not tested:
            selected = bench_enabled(package, kind, name, tested)
            if not selected:
                reason = "test_and_bench_disabled"
        item = {"package": package_name, "kind": kind, "name": name,
                "test": tested, "selected": selected, "reason": reason}
        inventory.append(item)
        if selected:
            args = ["--lib"] if kind == "lib" else ["--" + kind, name]
            package_groups.append({"id": f"{package_name}/{kind}/{name}",
                                   "package": package_name, "kind": kind, "name": name, "args": args})
    if not package_groups:
        raise SystemExit("package has no selected test groups; empty coverage cannot pass")
    groups.extend(sorted(package_groups, key=lambda g: (rank[g["kind"]], g["name"])))
plan = {"metadataDigest": "sha256:" + hashlib.sha256(raw).hexdigest(),
        "groups": groups, "inventory": sorted(inventory, key=lambda i: (i["package"], i["kind"], i["name"]))}
plan_path.write_text(json.dumps(plan, sort_keys=True) + "\n")
for group in groups:
    print(json.dumps(group, sort_keys=True))
PY
    then
      state=failed
      failure_phase=target_inventory
      failure_diagnostic_tail=$(tail -c 12000 "$tmp/group-diagnostic")
      : >"$tmp/groups"
    fi
  fi
  if ((package_count > 0)) && ! command -v timeout >/dev/null 2>&1; then
    state=failed
    failure_phase=runner_setup
    failure_diagnostic_tail="GNU timeout is required for bounded target-group execution"
  fi
  if ((package_count > 0)) && [[ "$state" == passed ]]; then
    rustc_bin="${RUSTC:-rustc}"
    if ! rustc_info=$("$rustc_bin" -vV 2>&1); then
      state=failed
      failure_phase=runner_setup
      failure_diagnostic_tail=$(tail -c 12000 <<<"$rustc_info")
    else
      host_target=$(sed -n 's/^host: //p' <<<"$rustc_info")
      if [[ -z "$host_target" ]]; then
        state=failed
        failure_phase=runner_setup
        failure_diagnostic_tail="rustc -vV did not report a host target"
      else
        runner_variable="CARGO_TARGET_${host_target//-/_}_RUNNER"
        runner_variable=$(tr '[:lower:]' '[:upper:]' <<<"$runner_variable")
        process_observer_runner="$root/tests/ci/run_process_observer_test_runner.sh"
        runner_config_diagnostic=$(
          python3 - "$root" "$host_target" "$process_observer_runner" 2>&1 <<'PY'
import ast
import os
import re
import subprocess
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:
    print("Python 3.11 or newer is required to inspect Cargo runner configuration", file=sys.stderr)
    raise SystemExit(1)

repository = Path(sys.argv[1])
host_target = sys.argv[2]
required_runner = sys.argv[3]
rustc = os.environ.get("RUSTC", "rustc")

cfg_result = subprocess.run(
    [rustc, "--print", "cfg", "--target", host_target],
    check=False,
    capture_output=True,
    text=True,
)
if cfg_result.returncode != 0:
    print(cfg_result.stderr.strip() or "rustc could not report host cfg values", file=sys.stderr)
    raise SystemExit(1)

cfg_flags = set()
cfg_pairs = set()
for line in cfg_result.stdout.splitlines():
    if "=" not in line:
        cfg_flags.add(line)
        continue
    key, encoded_value = line.split("=", 1)
    try:
        value = ast.literal_eval(encoded_value)
    except (SyntaxError, ValueError):
        print(f"rustc reported an invalid cfg value: {line}", file=sys.stderr)
        raise SystemExit(1)
    if not isinstance(value, str):
        print(f"rustc reported a non-string cfg value: {line}", file=sys.stderr)
        raise SystemExit(1)
    cfg_pairs.add((key, value))

token_pattern = re.compile(r'\s*([A-Za-z_][A-Za-z0-9_]*|"(?:\\.|[^"\\])*"|=|[(),])')

def parse_cfg(expression):
    tokens = []
    position = 0
    while position < len(expression):
        if not expression[position:].strip():
            break
        match = token_pattern.match(expression, position)
        if match is None:
            raise ValueError(f"unsupported cfg selector syntax: {expression}")
        tokens.append(match.group(1))
        position = match.end()

    def parse_node(index):
        if index >= len(tokens):
            raise ValueError(f"incomplete cfg selector: {expression}")
        name = tokens[index]
        if name in {"all", "any", "not"} and index + 1 < len(tokens) and tokens[index + 1] == "(":
            children = []
            index += 2
            while index < len(tokens) and tokens[index] != ")":
                child, index = parse_node(index)
                children.append(child)
                if index < len(tokens) and tokens[index] == ",":
                    index += 1
                elif index < len(tokens) and tokens[index] != ")":
                    raise ValueError(f"invalid cfg selector: {expression}")
            if index >= len(tokens):
                raise ValueError(f"unterminated cfg selector: {expression}")
            if name == "not" and len(children) != 1:
                raise ValueError(f"cfg not() must have one expression: {expression}")
            return (name, children), index + 1
        if index + 1 < len(tokens) and tokens[index + 1] == "=":
            if index + 2 >= len(tokens) or not tokens[index + 2].startswith('"'):
                raise ValueError(f"invalid cfg key/value selector: {expression}")
            value = ast.literal_eval(tokens[index + 2])
            return ("pair", name, value), index + 3
        return ("flag", name), index + 1

    tree, final_position = parse_node(0)
    if final_position != len(tokens):
        raise ValueError(f"trailing cfg selector input: {expression}")
    return tree

def matches_cfg(tree):
    kind = tree[0]
    if kind == "all":
        return all(matches_cfg(child) for child in tree[1])
    if kind == "any":
        return any(matches_cfg(child) for child in tree[1])
    if kind == "not":
        return not matches_cfg(tree[1][0])
    if kind == "pair":
        return (tree[1], tree[2]) in cfg_pairs
    return tree[1] in cfg_flags

config_files = []
for directory in (repository, *repository.parents):
    config_files.extend((directory / ".cargo/config.toml", directory / ".cargo/config"))
cargo_home = Path(os.environ.get("CARGO_HOME") or (Path.home() / ".cargo"))
config_files.extend((cargo_home / "config.toml", cargo_home / "config"))

seen = set()

def inspect_config(config_path, *, required=False):
    resolved_path = config_path.resolve()
    if resolved_path in seen:
        return
    if not config_path.is_file():
        if required:
            print(f"included Cargo configuration is missing or not a regular file: {config_path}", file=sys.stderr)
            raise SystemExit(1)
        return
    seen.add(resolved_path)
    try:
        configuration = tomllib.loads(config_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        print(f"cannot inspect Cargo configuration {config_path}: {error}", file=sys.stderr)
        raise SystemExit(1)

    includes = configuration.get("include", [])
    if not isinstance(includes, list):
        print(f"Cargo include configuration is not an array in {config_path}", file=sys.stderr)
        raise SystemExit(1)
    for include in includes:
        optional = False
        if isinstance(include, str):
            include_path = include
        elif isinstance(include, dict):
            if set(include) - {"path", "optional"}:
                print(f"Cargo include entry has unsupported fields in {config_path}", file=sys.stderr)
                raise SystemExit(1)
            include_path = include.get("path")
            optional = include.get("optional", False)
            if not isinstance(optional, bool):
                print(f"Cargo include optional flag is not a boolean in {config_path}", file=sys.stderr)
                raise SystemExit(1)
        else:
            print(f"Cargo include entry is not a path or table in {config_path}", file=sys.stderr)
            raise SystemExit(1)
        if not isinstance(include_path, str) or not include_path.endswith(".toml"):
            print(f"Cargo include path is invalid in {config_path}", file=sys.stderr)
            raise SystemExit(1)
        included_config = Path(include_path)
        if not included_config.is_absolute():
            included_config = config_path.parent / included_config
        if optional:
            try:
                included_config.stat()
            except FileNotFoundError:
                continue
            except OSError as error:
                print(f"cannot inspect optional Cargo configuration {included_config}: {error}", file=sys.stderr)
                raise SystemExit(1)
        inspect_config(included_config, required=True)

    targets = configuration.get("target", {})
    if not isinstance(targets, dict):
        print(f"Cargo target configuration is not a table in {config_path}", file=sys.stderr)
        raise SystemExit(1)
    for selector, settings in targets.items():
        if not isinstance(settings, dict) or "runner" not in settings:
            continue
        applies = selector == host_target
        if selector.startswith("cfg(") and selector.endswith(")"):
            try:
                applies = matches_cfg(parse_cfg(selector[4:-1]))
            except (ValueError, SyntaxError) as error:
                print(f"cannot resolve Cargo runner selector {selector!r} in {config_path}: {error}", file=sys.stderr)
                raise SystemExit(1)
        if not applies:
            continue
        configured = settings["runner"]
        if configured not in (required_runner, [required_runner]):
            print(
                f"configured Cargo target runner in {config_path} for {selector!r} conflicts with the required process observer runner",
                file=sys.stderr,
            )
            raise SystemExit(1)

for config_path in config_files:
    inspect_config(config_path)
PY
        )
        runner_config_status=$?
        configured_runner="${!runner_variable:-}"
        if ((runner_config_status != 0)); then
          state=failed
          failure_phase=runner_setup
          failure_diagnostic_tail=$(tail -c 12000 <<<"$runner_config_diagnostic")
        elif [[ -n "$configured_runner" && "$configured_runner" != "$process_observer_runner" ]]; then
          state=failed
          failure_phase=runner_setup
          failure_diagnostic_tail="${runner_variable} must use ${process_observer_runner} for safe process-observer isolation"
        else
          export "${runner_variable}=${process_observer_runner}"
        fi
      fi
    fi
  fi
  pids=()
  groups=()
  while IFS= read -r group; do groups+=("$group"); done <"$tmp/groups"
  group_count=${#groups[@]}
  active=0
  next=0
  while { [[ "$state" == passed ]] && ((next < group_count)) || ((active > 0)); }; do
    while [[ "$state" == passed ]] && ((next < group_count && active < workers)); do
      package=$(jq -er '.package' <<<"${groups[$next]}")
      selector=()
      while IFS= read -r argument; do selector+=("$argument"); done < <(jq -r '.args[]' <<<"${groups[$next]}")
      index=$next
      (
        set +e
        (cd "$root" && python3 "$tmp/group-owner.py" "$group_timeout_seconds" \
          timeout --signal=TERM --kill-after=30s "${group_timeout_seconds}s" \
          "$cargo_bin" test -p "$package" --target "$host_target" "${selector[@]}" -- --test-threads="$test_threads") \
          >"$tmp/results/$index.log" 2>&1
        result=$?
        if [[ "$result" == 0 ]]; then
          python3 "$root/tests/ci/run_no_warnings.py" --check-log "$tmp/results/$index.log"             >>"$tmp/results/$index.log" 2>&1
          result=$?
        fi
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
        cat "$tmp/results/$index.log"
        if [[ "$result" != 0 && "$state" == passed ]]; then
          state=failed
          failure_phase=package_test
          failed_package=$(jq -er '.package' <<<"${groups[$index]}")
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
: >"$tmp/coverage-run-executed"
python3 - "$tmp/group-plan.json" "$tmp/results" "$tmp/coverage-run-executed" "$tmp/group-results.json" <<'PY'
import json
import sys
from pathlib import Path

plan_path, results_path, completed_path, groups_path = map(Path, sys.argv[1:])
plan = json.loads(plan_path.read_text())
results = []
by_package = {}
for index, group in enumerate(plan["groups"]):
    status_path = results_path / f"{index}.status"
    code = int(status_path.read_text()) if status_path.is_file() else None
    result = dict(group, exitCode=code, state="not_run" if code is None else "passed" if code == 0 else "failed")
    results.append(result)
    by_package.setdefault(group["package"], []).append(code)
complete = [name for name, statuses in sorted(by_package.items()) if statuses and all(code == 0 for code in statuses)]
completed_path.write_text("".join(name + "\n" for name in complete))
groups_path.write_text(json.dumps(dict(plan, groups=results), sort_keys=True) + "\n")
PY
python3 - "$tmp/planned" "$tmp/hosted-runtime-executed" "$tmp/coverage-run-executed" "$tmp/executed" <<'PY'
import sys

planned_path, hosted_path, coverage_path, executed_path = sys.argv[1:]
def lines(path):
    return [line for line in open(path, encoding="utf-8").read().splitlines() if line]

completed = set(lines(hosted_path)) | set(lines(coverage_path))
with open(executed_path, "w", encoding="utf-8") as stream:
    stream.write("".join(f"{package}\n" for package in lines(planned_path) if package in completed))
PY

mkdir -p "$(dirname "$report")"
python3 - "$tmp/planned" "$tmp/executed" "$tmp/hosted-runtime-executed" "$tmp/coverage-run-executed" "$report" "$state" "$failure_phase" "$failed_package" "$failed_exit_code" "$failure_diagnostic_tail" "$receipt_verification_state" "$group_timeout_seconds" "$workers" "$test_threads" "$tmp/group-results.json" <<'PY'
import json
import sys

def lines(path):
    return [line for line in open(path, encoding="utf-8").read().splitlines() if line]

planned = lines(sys.argv[1])
executed = lines(sys.argv[2])
hosted_runtime_executed = lines(sys.argv[3])
coverage_run_executed = lines(sys.argv[4])
report = {
    "executed": executed,
    "executedByCoverageRunner": coverage_run_executed,
    "executedByHostedRuntime": hosted_runtime_executed,
    "omitted": sorted(set(planned) - set(executed)),
    "planned": planned,
    "schemaVersion": 1,
    "state": sys.argv[6],
    "verificationReceiptState": sys.argv[11],
    "groupTimeoutSeconds": int(sys.argv[12]),
    "groupCleanupGraceSeconds": 30,
    "groupReapGraceSeconds": 30,
    "groupOwnerDeadlineSlackSeconds": 1,
    "groupOwnerForceKillGraceSeconds": 30,
    "workers": int(sys.argv[13]),
    "testThreads": int(sys.argv[14]),
}
group_results = json.loads(open(sys.argv[15], encoding="utf-8").read())
report["groups"] = group_results["groups"]
report["targetInventory"] = group_results["inventory"]
if "metadataDigest" in group_results:
    report["groupMetadataDigest"] = group_results["metadataDigest"]
if sys.argv[7]:
    report["failurePhase"] = sys.argv[7]
if sys.argv[8]:
    report["failedPackage"] = sys.argv[8]
if sys.argv[9]:
    report["failedExitCode"] = int(sys.argv[9])
if sys.argv[10]:
    report["failureDiagnosticTail"] = sys.argv[10]
open(sys.argv[5], "w", encoding="utf-8").write(json.dumps(report, indent=2, sort_keys=True) + "\n")
PY

if [[ "$state" != passed ]]; then
  printf 'workspace package coverage failed: phase=%s package=%s exitCode=%s\n' \
    "$failure_phase" "$failed_package" "$failed_exit_code" >&2
  if [[ -n "$failure_diagnostic_tail" ]]; then
    printf '%s\n' "$failure_diagnostic_tail" >&2
  fi
  exit 1
fi
printf 'workspace package coverage passed: %s packages\n' "$(wc -l <"$tmp/executed" | tr -d ' ')"
