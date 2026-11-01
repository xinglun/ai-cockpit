#!/usr/bin/env bash
set -uo pipefail

# The optional publisher/binder lanes are independent of the legacy helper.
# Only the live Runtime child's bounded stdout can supply publisher metadata.
if [[ ${1:-} == --native-source || ${1:-} == --bind-native-source ]]; then
  python3 - "$@" <<'PY_NATIVE_SOURCE_MODE'
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import selectors
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import time

MAX_OBJECT = 64 * 1024 * 1024
MAX_RUNTIME = 512 * 1024 * 1024
MAX_STREAM = 64 * 1024
TOTAL_SECONDS = 900
CLEANUP_SECONDS = 10
MAX_PROC_ENTRIES = 32768
MAX_PROC_BYTES = 16 * 1024 * 1024
READ_FLAGS = os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW
DIRECTORY_FLAGS = READ_FLAGS | os.O_DIRECTORY
NATIVE_KEYS = {"schemaVersion", "publisherNonce", "inputDigest", "nativeRoot", "runtimeDigest"}
REPLY_KEYS = NATIVE_KEYS | {"portablePath", "executionId"}

class Refusal(Exception):
    pass

class CleanupUnknown(Refusal):
    pass

def require(condition):
    if not condition:
        raise Refusal()

def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result)
        result[key] = value
    return result

def parse_json(raw):
    return json.loads(raw, object_pairs_hook=unique,
                      parse_constant=lambda _: (_ for _ in ()).throw(Refusal()))

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("ascii")

def digest(value):
    return type(value) is str and re.fullmatch(r"sha256:[0-9a-f]{64}", value) is not None

def native_record(value):
    require(type(value) is dict and set(value) == NATIVE_KEYS)
    require(type(value["schemaVersion"]) is int and value["schemaVersion"] == 1)
    require(type(value["publisherNonce"]) is str and re.fullmatch(r"[0-9a-f]{64}", value["publisherNonce"]))
    require(all(digest(value[key]) for key in ("inputDigest", "nativeRoot", "runtimeDigest")))
    require(value["nativeRoot"] == value["inputDigest"])

def file_token(info):
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)

def open_regular(path, maximum, directory=None):
    fd = os.open(path, READ_FLAGS, dir_fd=directory)
    try:
        info = os.fstat(fd)
        require(stat.S_ISREG(info.st_mode) and 0 <= info.st_size <= maximum)
        return os.fdopen(fd, "rb"), info
    except BaseException:
        os.close(fd)
        raise

def hash_stream(stream, maximum, destination=None):
    hashed = hashlib.sha256()
    total = 0
    while True:
        block = stream.read(min(64 * 1024, maximum - total + 1))
        if not block:
            break
        total += len(block)
        require(total <= maximum)
        hashed.update(block)
        if destination is not None:
            destination.write(block)
    return "sha256:" + hashed.hexdigest()

def bounded_json_file(path, maximum, directory=None):
    stream, info = open_regular(path, maximum, directory)
    with stream:
        raw = stream.read(maximum + 1)
        require(len(raw) <= maximum and file_token(info) == file_token(os.fstat(stream.fileno())))
    return parse_json(raw)

def owned_child_status(process):
    # WNOWAIT keeps this exact child PID (and its session/group ID) reserved.
    # Popen.poll()/wait() must not run until all group signals are finished.
    if process.returncode is not None:
        raise CleanupUnknown()
    try:
        result = os.waitid(os.P_PID, process.pid,
                           os.WEXITED | os.WNOHANG | os.WNOWAIT)
    except ChildProcessError:
        raise CleanupUnknown() from None
    if result is not None and result.si_pid != process.pid:
        raise CleanupUnknown()
    return result

def owned_group_has_members(process, deadline):
    owned_child_status(process)
    entries = 0
    read_bytes = 0
    leader_seen = False
    others = False
    try:
        with os.scandir("/proc") as listing:
            for entry in listing:
                if not entry.name.isascii() or not entry.name.isdecimal():
                    continue
                entries += 1
                if entries > MAX_PROC_ENTRIES or time.monotonic() >= deadline:
                    raise CleanupUnknown()
                try:
                    fd = os.open("/proc/" + entry.name + "/stat", READ_FLAGS)
                except (FileNotFoundError, ProcessLookupError):
                    continue
                try:
                    with os.fdopen(fd, "rb") as source:
                        raw = source.read(4097)
                except (FileNotFoundError, ProcessLookupError):
                    continue
                read_bytes += len(raw)
                if len(raw) > 4096 or read_bytes > MAX_PROC_BYTES:
                    raise CleanupUnknown()
                boundary = raw.rfind(b") ")
                fields = raw[boundary + 2:].split() if boundary >= 0 else []
                if len(fields) < 4:
                    raise CleanupUnknown()
                pid = int(entry.name)
                group, session = int(fields[2]), int(fields[3])
                if pid == process.pid:
                    if group != process.pid or session != process.pid:
                        raise CleanupUnknown()
                    leader_seen = True
                elif group == process.pid:
                    if session != process.pid:
                        raise CleanupUnknown()
                    others = True
    except (OSError, ValueError):
        raise CleanupUnknown() from None
    if not leader_seen:
        raise CleanupUnknown()
    owned_child_status(process)
    return others

def signal_owned_group(process, number):
    owned_child_status(process)
    try:
        os.killpg(process.pid, number)
    except ProcessLookupError:
        # A group containing only the held zombie may have no signal target.
        # Membership is still checked before the final reap.
        pass

def stop_owned_child(process, deadline):
    clean = False
    # This covers only the fixed Runtime process group. Runtime supervision
    # remains responsible for the project tree, including separate sessions.
    end = min(deadline, time.monotonic() + 8)
    try:
        if owned_child_status(process) is None:
            signal_owned_group(process, signal.SIGTERM)
            grace = min(end, time.monotonic() + 2)
            while time.monotonic() < grace and owned_child_status(process) is None:
                time.sleep(0.01)
        # Close the anchored group before checking membership. This also stops
        # a same-group member racing the observation by forking new members.
        signal_owned_group(process, signal.SIGKILL)
        while time.monotonic() < end:
            if owned_child_status(process) is not None and not owned_group_has_members(process, end):
                clean = True
                break
            time.sleep(0.01)
    except (OSError, CleanupUnknown):
        clean = False
    # No negative-PGID signal or group observation is permitted after this
    # final reap, even when cleanup could not be proved within the deadline.
    try:
        process.wait(timeout=max(0.0, min(1.0, deadline - time.monotonic())))
    except (subprocess.TimeoutExpired, ChildProcessError):
        return False
    return clean

def actual_publisher(runtime, repository, request, deadline):
    require(time.monotonic() < deadline - CLEANUP_SECONDS)
    require(all(hasattr(os, name) for name in ("waitid", "WNOWAIT", "P_PID", "WEXITED", "WNOHANG", "CLD_EXITED")))
    require(Path("/proc/self/stat").is_file())
    process = subprocess.Popen([str(runtime), "__portable-native-publish", request],
        cwd=repository, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, start_new_session=True, close_fds=True)
    streams = {"stdout":bytearray(), "stderr":bytearray()}
    selector = None
    exited_at = None
    try:
        selector = selectors.DefaultSelector()
        for name, stream in (("stdout", process.stdout), ("stderr", process.stderr)):
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ, name)
        while True:
            require(time.monotonic() < deadline - CLEANUP_SECONDS)
            status = owned_child_status(process)
            if status is not None:
                if exited_at is None:
                    exited_at = time.monotonic()
                if not selector.get_map():
                    break
                if time.monotonic() >= exited_at + 1:
                    # A pipe held beyond the leader's exit may belong to an
                    # unobservable separate session. Preserve the stage.
                    raise CleanupUnknown()
            for key, _ in selector.select(0.05):
                chunk = os.read(key.fd, 8192)
                if not chunk:
                    selector.unregister(key.fileobj)
                else:
                    streams[key.data].extend(chunk)
                    require(len(streams[key.data]) <= MAX_STREAM)
        require(status.si_code == os.CLD_EXITED and status.si_status == 0)
        # Natural EOF and the real exit status are required. Residual original
        # group members are refused, not credited as a successful native run.
        require(not owned_group_has_members(process, deadline - CLEANUP_SECONDS))
        return bytes(streams["stdout"]), bytes(streams["stderr"])
    finally:
        if selector is not None:
            selector.close()
        clean = stop_owned_child(process, deadline)
        process.stdout.close()
        process.stderr.close()
        if not clean:
            raise CleanupUnknown()

def safe_text_path(value):
    require(type(value) is str and value and not any(ord(char) < 32 for char in value))
    return Path(value)

def bind(arguments):
    require(len(arguments) == 4)
    encoded, artifact_id, archive_digest = arguments[1:]
    require(len(encoded) <= 8192 and len(encoded) % 2 == 0 and re.fullmatch(r"[0-9a-f]+", encoded))
    value = parse_json(bytes.fromhex(encoded))
    native_record(value)
    require(canonical(value).hex() == encoded)
    require(re.fullmatch(r"[1-9][0-9]{0,19}", artifact_id) and int(artifact_id) <= 2**64 - 1)
    if re.fullmatch(r"[0-9a-f]{64}", archive_digest):
        archive_digest = "sha256:" + archive_digest
    require(digest(archive_digest))
    value.update(artifactId=int(artifact_id), archiveDigest=archive_digest)
    print("AI_COCKPIT_ARTIFACT_BINDING_V1 " + canonical(value).hex(), flush=True)

def publish(arguments):
    deadline = time.monotonic() + TOTAL_SECONDS
    require(len(arguments) in (7, 9) and arguments[5] == "--node-id")
    require(len(arguments) != 9 or arguments[7] == "--task-caller")
    repository = safe_text_path(arguments[2]).resolve(strict=True)
    require(repository.is_dir())
    root_fd = os.open(repository, DIRECTORY_FLAGS)
    try:
        publish_open_root(arguments, repository, root_fd, deadline)
    finally:
        os.close(root_fd)

def publish_open_root(arguments, repository, root_fd, deadline):
    runtime_arg, _, contract_arg, artifact_arg = arguments[1:5]
    node = arguments[6]
    require(re.fullmatch(r"project-command-[0-9]+", node))
    runtime = safe_text_path(runtime_arg)
    if not runtime.is_absolute():
        runtime = repository / runtime
    contract = safe_text_path(contract_arg)
    if not contract.is_absolute():
        contract = repository / contract
    relative_contract = contract.relative_to(repository)
    require(all(part not in ("", ".", "..") for part in relative_contract.parts))
    directory = os.dup(root_fd)
    try:
        for part in relative_contract.parts[:-1]:
            following = os.open(part, DIRECTORY_FLAGS, dir_fd=directory)
            os.close(directory)
            directory = following
        contract_value = bounded_json_file(relative_contract.parts[-1], 1024 * 1024, directory)
    finally:
        os.close(directory)
    require(type(contract_value) is dict and type(contract_value.get("workItemId")) is str)
    wi = contract_value["workItemId"]
    require(re.fullmatch(r"[A-Za-z0-9_-]+", wi))
    task_caller = None
    if len(arguments) == 9:
        caller_path = safe_text_path(arguments[8])
        if not caller_path.is_absolute():
            caller_path = repository / caller_path
        task_caller = bounded_json_file(caller_path, 64 * 1024)
        require(type(task_caller) is dict and set(task_caller) == {"schemaVersion","actor","authoritySource","assurance"})
        require(type(task_caller["schemaVersion"]) is int and task_caller["schemaVersion"] == 1)
        require(all(type(task_caller[key]) is str and task_caller[key] for key in ("actor","authoritySource")))
        require(task_caller["assurance"] == "self_declared")
    runner = safe_text_path(os.environ.get("RUNNER_TEMP", artifact_arg))
    if not runner.is_absolute():
        runner = repository / runner
    require(stat.S_ISDIR(runner.lstat().st_mode) and not runner.is_symlink())
    stage = Path(tempfile.mkdtemp(prefix="ai-cockpit-native-source-", dir=runner))
    os.chmod(stage, 0o700)
    stage_fd = os.open(stage, DIRECTORY_FLAGS)
    stage_identity = (os.fstat(stage_fd).st_dev, os.fstat(stage_fd).st_ino)
    succeeded = False
    preserve_stage = False
    try:
        source, before = open_regular(runtime, MAX_RUNTIME)
        require(before.st_size > 0 and before.st_mode & 0o111)
        with source, os.fdopen(os.open("native-runtime", os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                                      0o500, dir_fd=stage_fd), "wb") as destination:
            runtime_digest = hash_stream(source, MAX_RUNTIME, destination)
            require(file_token(before) == file_token(os.fstat(source.fileno())))
            destination.flush()
            os.fsync(destination.fileno())
        pinned_runtime = stage / "native-runtime"
        nonce = secrets.token_hex(32)  # Fresh non-secret session identity, before spawn.
        wire = canonical({"schemaVersion":1,"repositoryRoot":str(repository),"workItemId":wi,
                          "nodeId":node,"taskCaller":task_caller,"publisherNonce":nonce}).decode("ascii")
        require(len(wire) <= 16 * 1024)
        raw_stdout, raw_stderr = actual_publisher(pinned_runtime, repository, wire, deadline)
        for name, raw in (("publisher.stdout", raw_stdout), ("publisher.stderr", raw_stderr)):
            with os.fdopen(os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                                  0o600, dir_fd=stage_fd), "wb") as stream:
                stream.write(raw)
        reply = parse_json(raw_stdout)
        require(type(reply) is dict and set(reply) == REPLY_KEYS)
        value = {key:reply[key] for key in NATIVE_KEYS}
        native_record(value)
        require(reply["publisherNonce"] == nonce and reply["runtimeDigest"] == runtime_digest)
        require(digest(reply["executionId"]))
        expected_relative = Path(".ai/evidence") / (wi + ".verification-admissions/portable/objects") / (reply["inputDigest"][7:] + ".bin")
        require(type(reply["portablePath"]) is str and reply["portablePath"] == str(repository / expected_relative))
        directory = os.dup(root_fd)
        try:
            for part in expected_relative.parts[:-1]:
                following = os.open(part, DIRECTORY_FLAGS, dir_fd=directory)
                os.close(directory)
                directory = following
            original, object_before = open_regular(expected_relative.parts[-1], MAX_OBJECT, directory)
            with original, os.fdopen(os.open("portable-native-evidence.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                                            0o600, dir_fd=stage_fd), "wb") as target:
                observed_digest = hash_stream(original, MAX_OBJECT, target)
                require(file_token(object_before) == file_token(os.fstat(original.fileno())))
                target.flush()
                os.fsync(target.fileno())
            require(observed_digest == reply["inputDigest"])
        finally:
            os.close(directory)
        copied, _ = open_regular("portable-native-evidence.json", MAX_OBJECT, stage_fd)
        with copied:
            require(hash_stream(copied, MAX_OBJECT) == reply["inputDigest"])
        current_runtime, current_info = open_regular(runtime, MAX_RUNTIME)
        with current_runtime:
            require(file_token(current_info) == file_token(before) and hash_stream(current_runtime, MAX_RUNTIME) == runtime_digest)
        executed, _ = open_regular("native-runtime", MAX_RUNTIME, stage_fd)
        with executed:
            require(hash_stream(executed, MAX_RUNTIME) == runtime_digest)
        current_stage = stage.lstat()
        require(stat.S_ISDIR(current_stage.st_mode) and (current_stage.st_dev, current_stage.st_ino) == stage_identity)
        require(time.monotonic() < deadline)
        record_hex = canonical(value).hex()
        output_path = os.environ.get("GITHUB_OUTPUT")
        if output_path:
            fd = os.open(safe_text_path(output_path), os.O_WRONLY | os.O_APPEND | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK, 0o600)
            with os.fdopen(fd, "ab") as output:
                require(stat.S_ISREG(os.fstat(output.fileno()).st_mode))
                output.write(("native_record_hex=" + record_hex + "\nnative_portable_path=" + str(stage / "portable-native-evidence.json") + "\n").encode("utf-8"))
        print("AI_COCKPIT_NATIVE_SOURCE_V1 " + record_hex, flush=True)
        succeeded = True
    except CleanupUnknown:
        preserve_stage = True
        raise
    finally:
        os.close(stage_fd)
        if not succeeded and not preserve_stage:
            try:
                current = stage.lstat()
                if stat.S_ISDIR(current.st_mode) and (current.st_dev, current.st_ino) == stage_identity:
                    shutil.rmtree(stage)
            except OSError:
                pass  # Preserve uncertain owned partial data; never follow a replacement.

try:
    os.umask(0o077)
    if sys.argv[1] == "--bind-native-source":
        bind(sys.argv[1:])
    else:
        publish(sys.argv[1:])
except Exception:
    # Child stdout/stderr and input text are never reflected into job logs.
    print("native source publication/binding refused", file=sys.stderr)
    raise SystemExit(1)
PY_NATIVE_SOURCE_MODE
  exit "$?"
fi


if [[ $# -ne 4 && $# -ne 6 ]] || { [[ $# -eq 6 ]] && [[ $5 != --task-caller ]]; }; then
  printf 'usage: %s <runtime-bin> <repository> <contract> <artifact-dir> [--task-caller <file>]\n' "$0" >&2
  exit 2
fi

runtime_bin=$1
repository=$2
contract=$3
artifact_dir=$4
task_caller_file=""
if [[ $# -eq 6 ]]; then task_caller_file=$6; fi
repository=$(cd "$repository" && pwd -P) || exit 2
source_repository=$repository
execution_repository=$source_repository
if [[ "$runtime_bin" != /* ]]; then runtime_bin="$source_repository/$runtime_bin"; fi
if [[ "$contract" != /* ]]; then contract="$source_repository/$contract"; fi
if [[ "$artifact_dir" != /* ]]; then artifact_dir="$source_repository/$artifact_dir"; fi
if [[ -n "$task_caller_file" && "$task_caller_file" != /* ]]; then
  task_caller_file="$source_repository/$task_caller_file"
fi
contract_directory=$(cd "$(dirname "$contract")" && pwd -P) || exit 2
contract="$contract_directory/$(basename "$contract")"
if [[ ! -x "$runtime_bin" ]]; then
  printf 'candidate Runtime is not executable: %s\n' "$runtime_bin" >&2
  exit 2
fi
if [[ ! -f "$contract" || -L "$contract" ]]; then
  printf 'Work Item Contract is missing or not a regular file: %s\n' "$contract" >&2
  exit 2
fi
case "$contract" in
  "$source_repository"/*) contract_relative=${contract#"$source_repository"/} ;;
  *)
    printf 'Work Item Contract must be inside the source repository: %s\n' "$contract" >&2
    exit 2
    ;;
esac
mkdir -p "$artifact_dir" || exit 2

status_source="$artifact_dir/hosted-runtime-status-source.json"
status_before="$artifact_dir/hosted-runtime-status-before.json"
status_after="$artifact_dir/hosted-runtime-status-after.json"
status_verified="$artifact_dir/hosted-runtime-status-verified.json"
status_workspace="$artifact_dir/hosted-runtime-status-workspace.json"
validation_before="$artifact_dir/hosted-runtime-validation-before.json"
validation_after="$artifact_dir/hosted-runtime-validation-after.json"
preflight_output="$artifact_dir/hosted-runtime-preflight.json"
verification_output="$artifact_dir/hosted-runtime-execution.json"
formal_receipt_output="$artifact_dir/hosted-runtime-verification.json"
orchestration_output="$artifact_dir/hosted-runtime-orchestration.json"
cleanup_output="$artifact_dir/hosted-runtime-worktree-cleanup.json"

worktree_parent=""
worktree_path=""
execution_isolation_state=not_needed_fresh_receipt
execution_source_head=""
execution_cargo_context=null

work_item_id=$(jq -er '.workItemId | strings | select(length > 0)' "$contract") || {
  printf 'Contract does not declare a Work Item ID\n' >&2
  exit 2
}
runtime_digest=$(python3 - "$runtime_bin" <<'PY'
import hashlib
import sys
from pathlib import Path

print("sha256:" + hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
) || exit 2
: >"$formal_receipt_output" || exit 2

preflight_state=not_started
verification_state=not_started
failure_reason=""
initial_actions='[]'
refreshed_actions='[]'
execution_output_seal=null

capture_cargo_reuse_context() {
  python3 - "$execution_repository" "$source_repository/tests/ci/run_process_observer_test_runner.sh" <<'PY_CARGO_CONTEXT'
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
from pathlib import Path

def cargo_reuse_context(repository, required_runner, host_target=None):
    # Prove only the default/no-config case. Cargo's merging, includes,
    # relative paths and custom targets are deliberately not reimplemented.
    try:
        repository = Path(repository).resolve(strict=True)
        required_runner = Path(required_runner).absolute()
        if host_target is None:
            info = subprocess.run([os.environ.get("RUSTC", "rustc"), "-vV"],
                                  cwd=repository, capture_output=True, text=True, timeout=30)
            if info.returncode != 0:
                return None
            hosts = [line[6:] for line in info.stdout.splitlines() if line.startswith("host: ")]
            if len(hosts) != 1:
                return None
            host_target = hosts[0]
        if not re.fullmatch(r"[A-Za-z0-9_]+(?:-[A-Za-z0-9_]+)+", host_target):
            return None
        if os.environ.get("CARGO_BUILD_TARGET", "") not in ("", host_target):
            return None
        home = Path(os.environ.get("CARGO_HOME") or (Path.home() / ".cargo"))
        if not home.is_absolute():
            home = repository / home
        candidates = [directory / ".cargo" / name
                      for directory in (repository, *repository.parents)
                      for name in ("config", "config.toml")]
        candidates.extend(home / name for name in ("config", "config.toml"))
        for path in candidates:
            try:
                path.lstat()
            except FileNotFoundError:
                continue
            return None  # Any config, including dangling links, needs fresh execution.
        runner_key = "CARGO_TARGET_" + host_target.upper().replace("-", "_") + "_RUNNER"
        if os.environ.get(runner_key) != str(required_runner):
            return None
        if required_runner.resolve(strict=True) != required_runner:
            return None
        descriptor = os.open(required_runner, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
        with os.fdopen(descriptor, "rb") as stream:
            mode = os.fstat(stream.fileno()).st_mode
            if not stat.S_ISREG(mode) or not mode & 0o111:
                return None
            raw = stream.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            return None
        return {"hostTarget": host_target, "configuration": "absent",
                "runnerPath": str(required_runner),
                "runnerDigest": "sha256:" + hashlib.sha256(raw).hexdigest()}
    except (OSError, ValueError, subprocess.TimeoutExpired):
        return None

print(json.dumps(cargo_reuse_context(Path(sys.argv[1]), sys.argv[2]), sort_keys=True))
PY_CARGO_CONTEXT
}

write_orchestration_report() {
  local formal_receipt_digest=""
  if [[ -s "$formal_receipt_output" ]]; then
    formal_receipt_digest=$(python3 - "$formal_receipt_output" <<'PY'
import hashlib
import sys
from pathlib import Path

print("sha256:" + hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
) || return 1
  fi
  jq -n \
    --arg workItemId "$work_item_id" \
    --arg runtimeDigest "$runtime_digest" \
    --arg executionRepository "$execution_repository" \
    --arg formalReceiptDigest "$formal_receipt_digest" \
    --arg preflightState "$preflight_state" \
    --arg verificationState "$verification_state" \
    --arg failureReason "$failure_reason" \
    --argjson initialActions "$initial_actions" \
    --argjson refreshedActions "$refreshed_actions" \
    --argjson executionOutput "$execution_output_seal" \
    '{schemaVersion:1,workItemId:$workItemId,runtimeDigest:$runtimeDigest,executionRepository:$executionRepository,formalReceiptDigest:(if $formalReceiptDigest == "" then null else $formalReceiptDigest end),initialActions:$initialActions,refreshedActions:$refreshedActions,preflightState:$preflightState,verificationState:$verificationState,failureReason:(if $failureReason == "" then null else $failureReason end),executionOutput:$executionOutput}' \
    >"$orchestration_output"
}

if [[ ! "$work_item_id" =~ ^[A-Za-z0-9_-]+$ ]]; then
  verification_state=invalidated
  failure_reason=work_item_id_path_invalid
  write_orchestration_report
  printf 'Work Item ID is unsafe for the canonical evidence path\n' >&2
  exit 1
fi

task_caller_args=()
if [[ -n "$task_caller_file" ]]; then
  if [[ ! -f "$task_caller_file" || -L "$task_caller_file" || ! -r "$task_caller_file" ]] \
    || ! jq -e 'type == "object"
      and (keys == ["actor", "assurance", "authoritySource", "schemaVersion"])
      and .schemaVersion == 1
      and (.actor | type == "string" and length > 0)
      and (.authoritySource | type == "string" and length > 0)
      and .assurance == "self_declared"' "$task_caller_file" >/dev/null; then
    verification_state=invalidated
    failure_reason=task_caller_invalid
    write_orchestration_report
    printf 'explicit task caller must be a readable regular JSON file with the exact schemaVersion/actor/authoritySource/assurance fields\n' >&2
    exit 1
  fi
  task_caller_args=(--task-caller "$task_caller_file")
fi

cleanup_execution_worktree() {
  local command_status=$?
  local cleanup_status=0
  local cleanup_state=removed
  local defer_cleanup=false
  trap - EXIT

  if [[ "${AI_COCKPIT_DEFER_WORKTREE_CLEANUP:-false}" == true \
    && "$command_status" == 0 \
    && "$verification_state" == passed \
    && -n "$worktree_path" ]]; then
    defer_cleanup=true
    cleanup_state=deferred_for_consumer
  elif [[ -n "$worktree_path" ]]; then
    if git -C "$source_repository" worktree list --porcelain | awk -v path="$worktree_path" '$1 == "worktree" && substr($0, 10) == path { found = 1 } END { exit !found }'; then
      git -C "$source_repository" worktree remove --force "$worktree_path" >/dev/null || cleanup_status=$?
    elif [[ -d "$worktree_path" ]]; then
      rmdir "$worktree_path" || cleanup_status=$?
    fi
  fi
  if [[ "$defer_cleanup" != true && -n "$worktree_parent" && -d "$worktree_parent" ]]; then
    rmdir "$worktree_parent" || cleanup_status=$?
  fi

  [[ -z "$worktree_path" ]] && cleanup_state=$execution_isolation_state
  [[ "$cleanup_status" == 0 ]] || cleanup_state=failed
  jq -n \
    --arg state "$cleanup_state" \
    --arg sourceRepository "$source_repository" \
    --arg isolatedRepository "$worktree_path" \
    --argjson cleanupExitCode "$cleanup_status" \
    '{schemaVersion:1,state:$state,sourceRepository:$sourceRepository,isolatedRepository:(if $isolatedRepository == "" then null else $isolatedRepository end),cleanupExitCode:$cleanupExitCode}' \
    >"$cleanup_output" || cleanup_status=$?

  if [[ "$defer_cleanup" == true ]]; then
    exit "$command_status"
  fi

  if [[ "$cleanup_status" != 0 ]]; then
    printf 'failed to clean hosted Runtime worktree (exit %s): %s\n' "$cleanup_status" "$worktree_path" >&2
    exit 1
  fi
  exit "$command_status"
}
trap cleanup_execution_worktree EXIT

prepare_isolated_execution() {
  local source_head
  local untracked
  execution_isolation_state=preparing
  if ! git -C "$source_repository" diff --quiet HEAD --; then
    execution_isolation_state=failed_precondition
    printf 'source checkout has tracked changes; refusing to verify a different committed snapshot\n' >&2
    return 1
  fi
  untracked=$(git -C "$source_repository" ls-files --others --exclude-standard) || return 1
  if [[ -n "$untracked" ]]; then
    execution_isolation_state=failed_precondition
    printf 'source checkout has untracked files; refusing to verify a different committed snapshot:\n%s\n' "$untracked" >&2
    return 1
  fi

  source_head=$(git -C "$source_repository" rev-parse --verify 'HEAD^{commit}') || {
    execution_isolation_state=failed_to_resolve_source_head
    return 1
  }
  execution_source_head=$source_head
  worktree_parent=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/ai-cockpit-hosted-runtime.XXXXXX") || {
    execution_isolation_state=failed_to_create_parent
    return 1
  }
  worktree_parent=$(cd "$worktree_parent" && pwd -P) || {
    execution_isolation_state=failed_to_resolve_parent
    return 1
  }
  worktree_path="$worktree_parent/source"
  if ! git -C "$source_repository" worktree add --detach "$worktree_path" "$source_head" >"$artifact_dir/hosted-runtime-worktree-create.log" 2>&1; then
    execution_isolation_state=failed_to_create
    cat "$artifact_dir/hosted-runtime-worktree-create.log" >&2
    return 1
  fi

  execution_repository=$worktree_path
  contract="$execution_repository/$contract_relative"
  execution_isolation_state=created
  if ! query_status "$status_before" "$artifact_dir/hosted-runtime-status-before.stderr"; then
    failure_reason=isolated_initial_status_query_failed
    printf 'candidate Runtime status query failed in the isolated worktree\n' >&2
    return 1
  fi
  if ! jq -e --arg id "$work_item_id" '.workItemId == $id and (.safeActions | type == "array")' "$status_before" >/dev/null; then
    failure_reason=isolated_initial_status_identity_or_shape_invalid
    printf 'candidate Runtime returned an invalid isolated Work Item status projection\n' >&2
    return 1
  fi
  initial_actions=$(jq -c '.safeActions' "$status_before") || return 1
}

query_status() {
  local output=$1
  local error_output=$2
  "$runtime_bin" work-item status \
    --repo "$execution_repository" \
    --id "$work_item_id" \
    --json >"$output" 2>"$error_output"
}

query_validate() {
  local output=$1
  local error_output=$2
  "$runtime_bin" work-item validate \
    --repo "$execution_repository" \
    --id "$work_item_id" \
    --json >"$output" 2>"$error_output"
}

validation_report_is_valid() {
  local report_path=$1
  local contract_path=$2
  jq -e --slurpfile contract "$contract_path" '
    def nonblank: type == "string" and (gsub("^\\s+|\\s+$"; "") | length) > 0;
    if type != "object"
      or (.state | type != "string")
      or (.unknowns | type != "array")
      or (.findings | type != "array")
      or any(.unknowns[]; type != "string")
      or any(.findings[]; type != "object" or (.code | type != "string") or (.severity | type != "string"))
    then false
    else
      . as $report
      | $contract[0] as $document
      | [$report.findings[] | select(.severity == "error")] as $errors
      | [$report.unknowns[] | select(startswith("required_scenario_unverified:")) | ltrimstr("required_scenario_unverified:")] as $unknown_scenarios
      | [$document.scenarioCoverage[]? | select(type == "object") | select(.required == true and .status == "unverified")] as $planned
      | all($report.findings[]; .severity != "error")
        or (
          $report.state == "blocked"
          and $report.scenarioCoverage == "blocked"
          and ($document.scenarioCoverage | type == "array")
          and all($document.scenarioCoverage[]; type == "object")
          and ($errors | length) > 0
          and all($errors[]; .code == "required_scenario_unverified")
          and all($errors[]; (.message | nonblank))
          and all($report.unknowns[]; startswith("required_scenario_unverified:"))
          and all($planned[]; (.scenario | nonblank) and ((.expected // .expectedResult) | nonblank) and (.verificationPlan | nonblank))
          and ($planned | length) > 0
          and ($planned | length) == ($errors | length)
          and ($planned | length) == ($unknown_scenarios | length)
          and ($unknown_scenarios | unique | length) == ($unknown_scenarios | length)
          and (($planned | map(.scenario) | sort) == ($unknown_scenarios | sort))
          and (($errors | map(.message) | sort) == ($planned | map("required scenario \(.scenario) is unverified") | sort))
        )
    end' "$report_path" >/dev/null
}

status_allows() {
  local status_path=$1
  local action=$2
  jq -e --arg action "$action" '(.safeActions // []) | index($action) != null' "$status_path" >/dev/null
}

sync_workspace_formal_receipt() {
  local formal_evidence=$1
  local verified_status=$2
  if ! python3 - \
    "$source_repository" \
    "$execution_repository" \
    "$formal_evidence" \
    "$verified_status" \
    "$status_source" \
    "$work_item_id" \
    "$runtime_digest" <<'PY'
import hashlib
import json
import os
import re
import secrets
import stat
import sys
from pathlib import Path

def _concise_exception(error_type, error, traceback):
    print(f"workspace receipt sync rejected: {error}", file=sys.stderr)

sys.excepthook = _concise_exception

workspace, execution_root, formal_path, verified_path, source_status_path = map(Path, sys.argv[1:6])
work_item_id, runtime_digest = sys.argv[6:8]

def fail(message):
    raise ValueError(message)

def is_digest(value):
    return isinstance(value, str) and re.fullmatch(r"sha256:[0-9a-f]{64}", value) is not None

def read_regular(path):
    metadata = path.lstat()
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        fail(f"{path} must be a regular non-symlink file")
    return path.read_bytes()

if not re.fullmatch(r"[A-Za-z0-9_-]+", work_item_id):
    fail("unsafe Work Item ID")
if not is_digest(runtime_digest):
    fail("candidate Runtime digest is invalid")
if workspace.is_symlink() or not workspace.is_dir():
    fail("workspace root must be a real directory")
if execution_root.is_symlink() or not execution_root.is_dir():
    fail("isolated execution root must be a real directory")

filename = f"{work_item_id}.verification.json"
expected_formal_path = execution_root / ".ai" / "evidence" / filename
if formal_path != expected_formal_path:
    fail("formal receipt path does not match the exact Work Item target")

formal_bytes = read_regular(formal_path)
try:
    formal = json.loads(formal_bytes)
    verified = json.loads(read_regular(verified_path))
    source_status = json.loads(read_regular(source_status_path))
except (UnicodeDecodeError, json.JSONDecodeError) as error:
    fail(f"receipt or status JSON is invalid: {error}")

if not isinstance(formal, dict) or not isinstance(verified, dict) or not isinstance(source_status, dict):
    fail("receipt and status projections must be JSON objects")
if verified.get("workItemId") != work_item_id or source_status.get("workItemId") != work_item_id:
    fail("Work Item identity differs between source and verified status")
if verified.get("verification") != "verified" or verified.get("evidenceFreshness", {}).get("state") != "fresh":
    fail("isolated Runtime status does not report fresh verification")

verified_digests = verified.get("sourceDigests")
source_digests = source_status.get("sourceDigests")
if not isinstance(verified_digests, dict) or not isinstance(source_digests, dict):
    fail("source and isolated Runtime statuses lack digest bindings")
repository_id = verified.get("repositoryId")
contract_digest = verified_digests.get("contract")
snapshot_digest = verified_digests.get("repositorySnapshot")
if not is_digest(repository_id) or not is_digest(contract_digest) or not is_digest(snapshot_digest):
    fail("verified Runtime status contains an invalid repository, Contract, or snapshot digest")
if source_status.get("repositoryId") != repository_id:
    fail("source and isolated Runtime repository identities differ")
if source_digests.get("contract") != contract_digest or source_digests.get("repositorySnapshot") != snapshot_digest:
    fail("source and isolated Runtime Contract or snapshot bindings differ")

receipt = formal.get("receipt")
if not isinstance(receipt, dict):
    fail("formal verification receipt is missing its nested receipt")
if formal.get("workItemId") != work_item_id or formal.get("passed") is not True:
    fail("formal verification receipt has the wrong Work Item identity or is not passed")
if formal.get("runtimeDigest") != runtime_digest or receipt.get("runtimeDigest") != runtime_digest:
    fail("formal verification receipt Runtime identity differs from the executed Runtime")
if not isinstance(formal.get("runtimeVersion"), str) or not formal["runtimeVersion"].strip():
    fail("formal verification receipt has no Runtime version")
if receipt.get("runtimeVersion") != formal.get("runtimeVersion") or receipt.get("workItemId") != work_item_id or receipt.get("passed") is not True:
    fail("nested verification receipt identity is inconsistent")
if formal.get("repositoryId") != repository_id or receipt.get("repositoryId") != repository_id:
    fail("formal verification receipt repository identity differs from Runtime status")
if formal.get("contractDigest") != contract_digest:
    fail("formal verification receipt Contract digest differs from Runtime status")
if formal.get("repositorySnapshotDigest") != snapshot_digest:
    fail("formal verification receipt snapshot digest differs from Runtime status")
plan = receipt.get("planReceipt")
if not isinstance(plan, dict) or plan.get("workItemId") != work_item_id or plan.get("repositoryId") != repository_id:
    fail("formal verification plan receipt identity is inconsistent")
if plan.get("repositorySnapshotDigest") != snapshot_digest:
    fail("formal verification plan receipt snapshot differs from Runtime status")

directory_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
file_flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
def open_directory(parent_fd, name):
    descriptor = os.open(name, directory_flags, dir_fd=parent_fd)
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        fail(f"{name} is not a directory")
    return descriptor

root_fd = os.open(workspace, directory_flags)
ai_fd = evidence_fd = None
temporary_name = None
try:
    ai_fd = open_directory(root_fd, ".ai")
    evidence_fd = open_directory(ai_fd, "evidence")
    try:
        existing = os.stat(filename, dir_fd=evidence_fd, follow_symlinks=False)
    except FileNotFoundError:
        existing = None
    if existing is not None and (stat.S_ISLNK(existing.st_mode) or not stat.S_ISREG(existing.st_mode)):
        fail("canonical workspace receipt target is not a regular non-symlink file")

    temporary_name = f".{filename}.{os.getpid()}.{secrets.token_hex(8)}.tmp"
    create_flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
    temporary_fd = os.open(temporary_name, create_flags, 0o600, dir_fd=evidence_fd)
    try:
        with os.fdopen(temporary_fd, "wb", closefd=True) as stream:
            stream.write(formal_bytes)
            stream.flush()
            os.fsync(stream.fileno())
    except BaseException:
        try:
            os.unlink(temporary_name, dir_fd=evidence_fd)
        except FileNotFoundError:
            pass
        raise

    os.replace(temporary_name, filename, src_dir_fd=evidence_fd, dst_dir_fd=evidence_fd)
    temporary_name = None
    os.fsync(evidence_fd)
    installed_fd = os.open(filename, file_flags, dir_fd=evidence_fd)
    try:
        installed = bytearray()
        while True:
            chunk = os.read(installed_fd, 1024 * 1024)
            if not chunk:
                break
            installed.extend(chunk)
    finally:
        os.close(installed_fd)
    if bytes(installed) != formal_bytes:
        fail("atomic workspace receipt bytes differ from the verified source")
    source_hash = hashlib.sha256(formal_bytes).hexdigest()
    installed_hash = hashlib.sha256(installed).hexdigest()
    if source_hash != installed_hash:
        fail("atomic workspace receipt hash differs from the verified source")
finally:
    if temporary_name is not None and evidence_fd is not None:
        try:
            os.unlink(temporary_name, dir_fd=evidence_fd)
        except FileNotFoundError:
            pass
    if evidence_fd is not None:
        os.close(evidence_fd)
    if ai_fd is not None:
        os.close(ai_fd)
    os.close(root_fd)

PY
  then
    return 1
  fi

  if ! "$runtime_bin" work-item status \
    --repo "$source_repository" \
    --id "$work_item_id" \
    --json >"$status_workspace" 2>"$artifact_dir/hosted-runtime-status-workspace.stderr"; then
    printf 'candidate Runtime could not recheck the synchronized workspace receipt\n' >&2
    return 1
  fi
  if ! jq -e \
    --arg id "$work_item_id" \
    --slurpfile receipt "$formal_evidence" \
    '.workItemId == $id
      and .repositoryId == $receipt[0].repositoryId
      and .sourceDigests.contract == $receipt[0].contractDigest
      and .sourceDigests.repositorySnapshot == $receipt[0].repositorySnapshotDigest
      and .verification == "verified"
      and .evidenceFreshness.state == "fresh"' \
    "$status_workspace" >/dev/null; then
    printf 'candidate Runtime did not accept the synchronized workspace receipt as fresh\n' >&2
    return 1
  fi
  if ! cmp -s "$formal_evidence" "$repository/.ai/evidence/$work_item_id.verification.json"; then
    printf 'canonical workspace receipt bytes differ from the verified receipt\n' >&2
    return 1
  fi
}

if ! query_status "$status_source" "$artifact_dir/hosted-runtime-status-source.stderr"; then
  failure_reason=initial_status_query_failed
  write_orchestration_report
  printf 'candidate Runtime status query failed; see %s\n' "$artifact_dir/hosted-runtime-status-before.stderr" >&2
  exit 1
fi
if ! jq -e --arg id "$work_item_id" '.workItemId == $id and (.safeActions | type == "array")' "$status_source" >/dev/null; then
  failure_reason=initial_status_identity_or_shape_invalid
  write_orchestration_report
  printf 'candidate Runtime returned an invalid Work Item status projection\n' >&2
  exit 1
fi
initial_actions=$(jq -c '.safeActions' "$status_source") || exit 1
if ! query_validate "$validation_before" "$artifact_dir/hosted-runtime-validation-before.stderr"; then
  verification_state=invalidated
  failure_reason=initial_validation_query_failed
  write_orchestration_report
  printf 'candidate Runtime Work Item validation query failed; see %s\n' \
    "$artifact_dir/hosted-runtime-validation-before.stderr" >&2
  exit 1
fi
if ! validation_report_is_valid "$validation_before" "$contract"; then
  verification_state=invalidated
  failure_reason=validation_report_invalid
  write_orchestration_report
  printf 'candidate Runtime returned an invalid Work Item validation report\n' >&2
  exit 1
fi

verification_exit=0
if jq -e '.verification == "verified" and .evidenceFreshness.state == "fresh"' "$status_source" >/dev/null; then
  # A complete fresh Runtime receipt is already the required formal result.
  # Reuse its exact bytes; do not rerun preflight or verification merely to
  # recreate evidence for the same Work Item snapshot.
  preflight_state=not_required_existing_fresh_receipt
  refreshed_actions=$initial_actions
  formal_evidence="$repository/.ai/evidence/$work_item_id.verification.json"
  if [[ -L "$repository/.ai" || -L "$repository/.ai/evidence" || ! -f "$formal_evidence" || -L "$formal_evidence" ]]; then
    verification_state=invalidated
    failure_reason=existing_fresh_receipt_missing_or_unsafe
    verification_exit=1
  elif ! jq -e \
    --arg id "$work_item_id" \
    --arg digest "$runtime_digest" \
    --argjson status "$(<"$status_source")" \
    '.workItemId == $id
      and .passed == true
      and .runtimeDigest == $digest
      and (.runtimeVersion | type == "string" and length > 0)
      and (.contractDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
      and (.repositoryId | type == "string" and test("^sha256:[0-9a-f]{64}$"))
      and (.repositorySnapshotDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
      and .workItemId == $status.workItemId
      and .repositoryId == $status.repositoryId
      and .contractDigest == $status.sourceDigests.contract
      and .repositorySnapshotDigest == $status.sourceDigests.repositorySnapshot
      and (.receipt | type == "object")
      and .receipt.workItemId == $id
      and .receipt.passed == true
      and .receipt.runtimeDigest == $digest
      and .receipt.runtimeVersion == .runtimeVersion
      and .receipt.repositoryId == .repositoryId
      and (.receipt.results | type == "array" and length > 0)
      and .receipt.nodesPlanned == (.receipt.results | length)
      and .receipt.nodesExecuted + .receipt.nodesReused == .receipt.nodesPlanned
      and all(.receipt.results[]; .passed == true and (.nodeId | type == "string" and length > 0))
      and ([.receipt.results[].nodeId] | unique | length) == (.receipt.results | length)
      and (.receipt.planReceipt | type == "object")
      and .receipt.planReceipt.workItemId == $id
      and .receipt.planReceipt.repositoryId == .repositoryId
      and .receipt.planReceipt.baseRevision == $status.baseCommit
      and .receipt.planReceipt.stage == "task"
      and .receipt.planReceipt.repositorySnapshotDigest == .repositorySnapshotDigest
      and (.receipt.planReceipt.executedNodes | type == "array")
      and (.receipt.planReceipt.reusedNodes | type == "array")
      and (([.receipt.planReceipt.executedNodes[], .receipt.planReceipt.reusedNodes[]] | sort) == ([.receipt.results[].nodeId] | sort))
      and (.receipt.planReceipt.coverageManifest.nodeIds | type == "array" and length > 0)
      and ([.receipt.results[].nodeId] as $result_ids | all(.receipt.planReceipt.coverageManifest.nodeIds[]; . as $node | ($result_ids | index($node)) != null))' \
    "$formal_evidence" >/dev/null; then
    verification_state=invalidated
    failure_reason=existing_fresh_receipt_identity_mismatch
    verification_exit=1
  elif ! cp "$formal_evidence" "$formal_receipt_output" || ! cmp -s "$formal_evidence" "$formal_receipt_output"; then
    verification_state=invalidated
    failure_reason=formal_verification_evidence_copy_failed
    verification_exit=1
  else
    verification_state=reused
  fi
else
  if [[ -z "$task_caller_file" ]] \
    && jq -e '.governanceProfile.taskAuthorization? != null' "$contract" >/dev/null; then
    verification_state=invalidated
    failure_reason=task_caller_required
    write_orchestration_report
    printf 'task-authorized Work Item verification requires an explicit --task-caller file\n' >&2
    exit 1
  fi
  if ! prepare_isolated_execution; then
    [[ -n "$failure_reason" ]] || failure_reason=isolated_worktree_preparation_failed
    verification_state=failed
    write_orchestration_report
    exit 1
  fi
  if status_allows "$status_before" run_preflight; then
    preflight_state=admitted
    preflight_exit=0
    "$runtime_bin" preflight \
      --repo "$execution_repository" \
      --contract "$contract" \
      "${task_caller_args[@]}" >"$preflight_output" 2>"$artifact_dir/hosted-runtime-preflight.stderr" || preflight_exit=$?
    if [[ "$preflight_exit" != 0 ]]; then
      preflight_state=failed
      failure_reason=preflight_failed
      if query_status "$status_after" "$artifact_dir/hosted-runtime-status-after.stderr"; then
        refreshed_actions=$(jq -c '.safeActions // []' "$status_after" 2>/dev/null) || refreshed_actions='[]'
      else
        refreshed_actions='[]'
      fi
      write_orchestration_report
      printf 'candidate Runtime preflight failed with exit %s\n' "$preflight_exit" >&2
      exit "$preflight_exit"
    fi
    preflight_state=passed
  elif status_allows "$status_before" run_verification; then
    preflight_state=not_admitted_and_not_required
  else
    failure_reason=no_admitted_preflight_or_verification_action
    write_orchestration_report
    printf 'candidate Runtime admits neither run_preflight nor run_verification\n' >&2
    exit 1
  fi

  # Re-read admission after the optional preflight and immediately before
  # verification. A preflight receipt or an earlier status snapshot is not
  # authority for the following action.
  if ! query_status "$status_after" "$artifact_dir/hosted-runtime-status-after.stderr"; then
    failure_reason=refreshed_status_query_failed
    write_orchestration_report
    printf 'candidate Runtime status refresh failed; see %s\n' "$artifact_dir/hosted-runtime-status-after.stderr" >&2
    exit 1
  fi
  if ! jq -e --arg id "$work_item_id" '.workItemId == $id and (.safeActions | type == "array")' "$status_after" >/dev/null; then
    failure_reason=refreshed_status_identity_or_shape_invalid
    write_orchestration_report
    printf 'candidate Runtime returned an invalid refreshed Work Item status projection\n' >&2
    exit 1
  fi
  refreshed_actions=$(jq -c '.safeActions' "$status_after") || exit 1
  if ! status_allows "$status_after" run_verification; then
    failure_reason=run_verification_not_admitted_after_refresh
    write_orchestration_report
    printf 'candidate Runtime no longer admits run_verification after status refresh\n' >&2
    exit 1
  fi
  if ! query_validate "$validation_after" "$artifact_dir/hosted-runtime-validation-after.stderr"; then
    verification_state=invalidated
    failure_reason=validation_query_failed
    write_orchestration_report
    printf 'candidate Runtime Work Item validation refresh failed; see %s\n' \
      "$artifact_dir/hosted-runtime-validation-after.stderr" >&2
    exit 1
  fi
  if ! validation_report_is_valid "$validation_after" "$contract"; then
    verification_state=invalidated
    failure_reason=validation_report_invalid
    write_orchestration_report
    printf 'candidate Runtime returned an invalid refreshed Work Item validation report\n' >&2
    exit 1
  fi

  # The source-local cheap consumers are prerequisites, not a permission or
  # a reusable receipt. Run them against this exact isolated source before
  # any declared project command. Unknown usage does not manufacture errors.
  if ! bash "$(dirname "$0")/run_repository_prechecks.sh" --repo "$execution_repository" \
    >"$artifact_dir/hosted-runtime-prechecks.log" 2>&1; then
    verification_state=invalidated
    failure_reason=repository_prechecks_failed
    write_orchestration_report
    cat "$artifact_dir/hosted-runtime-prechecks.log" >&2
    exit 1
  fi

  # Request only the current operation's explicit Work Item ceiling. This is
  # not authorization: Runtime still checks the effective policy, stage and
  # unconditional limit before spawning any verification command.
  verification_timeout_args=()
  if ! verification_timeout=$(python3 - "$contract" <<'PY'
import json
import sys
from pathlib import Path

contract = json.loads(Path(sys.argv[1]).read_text())
policy = contract.get("governancePolicy")
if policy is None:
    raise SystemExit(0)
operation = contract.get("operation")
if operation is None:
    operation = "production_destructive" if "destructive" in contract.get("risk", "").lower() else "modify_source"
rule = next((rule for rule in policy["rules"] if rule["operation"] == operation), None)
requirement = rule.get("verificationRequirement") if rule else None
timeout = requirement.get("maxTimeoutSeconds") if requirement else None
if timeout is not None:
    if type(timeout) is not int or timeout <= 0:
        raise SystemExit("declared verification timeout must be a positive finite integer")
    print(timeout)
PY
  ); then
    verification_state=invalidated
    failure_reason=verification_timeout_request_invalid
    write_orchestration_report
    exit 1
  fi
  if [[ -n "$verification_timeout" ]]; then
    verification_timeout_args=(--timeout-seconds "$verification_timeout")
  fi

  execution_cargo_context=$(capture_cargo_reuse_context) || execution_cargo_context=null

  "$runtime_bin" verify \
    --repo "$execution_repository" \
    --work-item "$work_item_id" \
    --workers 1 "${verification_timeout_args[@]}" "${task_caller_args[@]}" \
    >"$verification_output" 2>"$artifact_dir/hosted-runtime-verification.stderr" || verification_exit=$?
  if [[ "$verification_exit" != 0 ]]; then
    verification_state=failed
    failure_reason=verification_failed
  else
    formal_evidence="$execution_repository/.ai/evidence/$work_item_id.verification.json"
    if [[ -L "$repository/.ai" || -L "$repository/.ai/evidence" || ! -f "$formal_evidence" || -L "$formal_evidence" ]]; then
      verification_state=invalidated
      failure_reason=formal_verification_evidence_missing_or_unsafe
      verification_exit=1
    elif ! jq -e \
      --arg id "$work_item_id" \
      --arg digest "$runtime_digest" \
      '.workItemId == $id
        and .passed == true
        and .runtimeDigest == $digest
        and (.runtimeVersion | type == "string" and length > 0)
        and (.contractDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
        and (.repositoryId | type == "string" and test("^sha256:[0-9a-f]{64}$"))
        and (.repositorySnapshotDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
        and (.receipt | type == "object")
        and .receipt.workItemId == $id
        and .receipt.passed == true
        and .receipt.runtimeDigest == $digest
        and .receipt.runtimeVersion == .runtimeVersion
        and .receipt.repositoryId == .repositoryId
        and .receipt.planReceipt.repositorySnapshotDigest == .repositorySnapshotDigest' \
      "$formal_evidence" >/dev/null; then
      verification_state=invalidated
      failure_reason=formal_verification_evidence_identity_mismatch
      verification_exit=1
    elif ! query_status "$status_verified" "$artifact_dir/hosted-runtime-status-verified.stderr"; then
      verification_state=invalidated
      failure_reason=post_verification_status_query_failed
      verification_exit=1
    elif ! jq -e \
      --arg id "$work_item_id" \
      --arg contract_digest "$(jq -r '.contractDigest' "$formal_evidence")" \
      --arg snapshot_digest "$(jq -r '.repositorySnapshotDigest' "$formal_evidence")" \
      '.workItemId == $id
        and .verification == "verified"
        and .evidenceFreshness.state == "fresh"
        and .sourceDigests.contract == $contract_digest
        and .sourceDigests.repositorySnapshot == $snapshot_digest' \
      "$status_verified" >/dev/null; then
      verification_state=invalidated
      failure_reason=post_verification_status_mismatch
      verification_exit=1
    elif ! sync_workspace_formal_receipt "$formal_evidence" "$status_verified"; then
      verification_state=invalidated
      failure_reason=workspace_formal_verification_sync_failed
      verification_exit=1
    elif ! cp "$formal_evidence" "$formal_receipt_output" || ! cmp -s "$formal_evidence" "$formal_receipt_output"; then
      verification_state=invalidated
      failure_reason=formal_verification_evidence_copy_failed
      verification_exit=1
    else
      verification_state=passed
    fi
  fi
fi

final_runtime_digest=$(python3 - "$runtime_bin" <<'PY'
import hashlib
import sys
from pathlib import Path

print("sha256:" + hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
) || exit 2
if [[ "$final_runtime_digest" != "$runtime_digest" ]]; then
  verification_state=invalidated
  failure_reason=candidate_runtime_changed_during_verification
  verification_exit=1
fi

if [[ "$verification_state" == passed ]]; then
  final_cargo_context=$(capture_cargo_reuse_context) || final_cargo_context=null
  if [[ "$execution_cargo_context" != "$final_cargo_context" ]]; then
    final_cargo_context=null
  fi
  # Capture only the stdout just returned by this Runtime invocation. This is
  # same-job orchestration provenance, not a native signature or protection
  # against an actor able to rewrite all of the hosted input artifacts.
  if ! execution_output_seal=$(python3 - "$verification_output" "$formal_receipt_output" "$execution_source_head" "$final_cargo_context" <<'PY'
import hashlib
import json
import os
import re
import stat
import sys
from pathlib import Path

def coverage_environment_digest():
    ignored = {"_", "SHLVL", "OLDPWD", "GITHUB_ACTION", "GITHUB_ENV", "GITHUB_OUTPUT",
               "GITHUB_PATH", "GITHUB_STATE", "GITHUB_STEP_SUMMARY",
               "AI_COCKPIT_DEFER_WORKTREE_CLEANUP", "ROUTE_CONTRACT_PATH", "ROUTE_LANE", "ROUTE_PROFILE",
               "AI_COCKPIT_VERIFICATION_RECEIPT", "AI_COCKPIT_VERIFICATION_ORCHESTRATION",
               "AI_COCKPIT_RUNTIME_BIN", "AI_COCKPIT_VERIFICATION_REPOSITORY",
               "AI_COCKPIT_GATE_ID", "AI_COCKPIT_GATE_MANIFEST_DIGEST",
               "AI_COCKPIT_GATE_ROUTE_RECEIPT_DIGEST", "AI_COCKPIT_GATE_STAGE"}
    values = sorted((key, value) for key, value in os.environ.items() if key not in ignored)
    return "sha256:" + hashlib.sha256(json.dumps(values, separators=(",", ":"), ensure_ascii=True).encode()).hexdigest()

def unique(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError("duplicate stdout/evidence JSON field")
        value[key] = item
    return value

def read_regular(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
    with os.fdopen(descriptor, "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise ValueError("hosted output is not a regular file")
        raw = stream.read(64 * 1024 * 1024 + 1)
        if len(raw) > 64 * 1024 * 1024:
            raise ValueError("hosted output exceeds capture budget")
        return raw

try:
    path = Path(sys.argv[1])
    raw = read_regular(path)
    output = json.loads(raw, object_pairs_hook=unique)
    formal = json.loads(read_regular(sys.argv[2]), object_pairs_hook=unique)
    receipt = formal["receipt"]
    if not isinstance(output, dict) or output.get("workItemId", formal["workItemId"]) != formal["workItemId"]:
        raise ValueError("Runtime stdout has an invalid Work Item identity")
    projection = {key: output.get(key) for key in receipt}
    projection["workItemId"] = output["planReceipt"]["workItemId"]
    if (json.dumps(projection, sort_keys=True, allow_nan=False) != json.dumps(receipt, sort_keys=True, allow_nan=False)
            or not isinstance(output.get("plannedNodes"), list)):
        raise ValueError("Runtime stdout does not project to the formal receipt")
    context = None
    names = {"repository": "GITHUB_REPOSITORY", "runId": "GITHUB_RUN_ID",
             "runAttempt": "GITHUB_RUN_ATTEMPT", "job": "GITHUB_JOB",
             "headSha": "GITHUB_SHA", "runnerOs": "RUNNER_OS"}
    candidate = {key: os.environ.get(name, "") for key, name in names.items()}
    if (os.environ.get("GITHUB_ACTIONS") == "true"
            and all(candidate.values()) and candidate["headSha"] == sys.argv[3]):
        context = candidate
    print(json.dumps({"schemaVersion": 1, "source": "runtime_stdout",
                      "file": "hosted-runtime-execution.json",
                      "sha256": "sha256:" + hashlib.sha256(raw).hexdigest(),
                      "providerContext": context, "environmentDigest": coverage_environment_digest(),
                      "cargoContext": json.loads(sys.argv[4])}))
except (OSError, ValueError, KeyError, TypeError) as error:
    print("hosted execution output binding rejected: " + str(error), file=sys.stderr)
    raise SystemExit(1)
PY
  ); then
    execution_output_seal=null
    verification_state=invalidated
    failure_reason=execution_output_binding_invalid
    verification_exit=1
  fi
fi

write_orchestration_report
if [[ "$failure_reason" == post_verification_status_mismatch ]]; then
  printf 'candidate Runtime does not accept the formal receipt as fresh for the current Contract and source snapshot\n' >&2
fi
if [[ "$failure_reason" == existing_fresh_receipt_* || "$failure_reason" == formal_verification_evidence_copy_failed ]]; then
  printf 'candidate Runtime reports fresh verification, but its formal receipt is not reusable (%s)\n' "$failure_reason" >&2
fi
exit "$verification_exit"
