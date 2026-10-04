#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
runner="$root/tests/ci/run_process_observer_test_runner.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-process-observer-runner.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/bin"
write_fake_test_binary() {
  local path=$1
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    'printf "%s\\n" "${RUST_TEST_THREADS-<unset>}" > "$OBSERVED_THREADS"' \
    'printf "%s\\n" "$@" > "$OBSERVED_ARGS"' >"$path"
  chmod +x "$path"
}

assert_sensitive_runner() {
  local target_name=$1
  local expected_threads=$2
  local expected_args=$3
  shift 3
  local -a supplied_args=("$@")

  local executable="$tmp/bin/$target_name"
  write_fake_test_binary "$executable"
  OBSERVED_THREADS="$tmp/threads" OBSERVED_ARGS="$tmp/args" RUST_TEST_THREADS=8 \
    "$runner" "$executable" "${supplied_args[@]}"
  test "$(<"$tmp/threads")" = "$expected_threads"
  diff -u <(printf '%s\n' "$expected_args") "$tmp/args"
}

assert_sensitive_runner "composition-0123456789abcdef" 1 \
  $'--test-threads=1\n--nocapture\nscenario_filter' \
  --nocapture --test-threads=4 scenario_filter
assert_sensitive_runner "collaboration_admission-0123456789abcdef" 1 \
  $'--test-threads=1\n--exact\ntarget_test' \
  --test-threads 4 --exact target_test
assert_sensitive_runner "cockpit_verification-0123456789abcdef" 1 \
  $'--test-threads=1\n--nocapture\nobserver_scenario' \
  --nocapture --test-threads=4 observer_scenario

ordinary="$tmp/bin/ordinary_test-0123456789abcdef"
write_fake_test_binary "$ordinary"
OBSERVED_THREADS="$tmp/ordinary-threads" OBSERVED_ARGS="$tmp/ordinary-args" RUST_TEST_THREADS=8 \
  "$runner" "$ordinary" --test-threads=4 ordinary_filter
test "$(<"$tmp/ordinary-threads")" = 8
diff -u <(printf '%s\n' --test-threads=4 ordinary_filter) "$tmp/ordinary-args"

# The canonical package runner must select the host target and propagate the
# process-observer Cargo runner into every package test invocation.
fake_rustc="$tmp/bin/rustc"
fake_cargo="$tmp/bin/cargo"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'case "${1-}" in' \
  '  -vV) printf "host: x86_64-unknown-linux-gnu\\n" ;;' \
  '  --print) printf "unix\\ntarget_arch=\\\"x86_64\\\"\\ntarget_os=\\\"linux\\\"\\n" ;;' \
  '  *) exit 2 ;;' \
  'esac' >"$fake_rustc"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  '[[ "${1-}" == test ]] || exit 2' \
  '[[ "${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER-}" == "$EXPECTED_RUNNER" ]] || { printf "observer runner was not propagated\\n" >&2; exit 3; }' \
  'target_seen=false' \
  'while (($#)); do if [[ "$1" == --target && "${2-}" == x86_64-unknown-linux-gnu ]]; then target_seen=true; break; fi; shift; done' \
  '[[ "$target_seen" == true ]] || { printf "workspace test did not select the runner target\\n" >&2; exit 4; }' \
  >"$fake_cargo"
chmod +x "$fake_rustc" "$fake_cargo"
printf '%s\n' '{"packages":[{"name":"fixture-package","source":null}]}' >"$tmp/workspace-metadata.json"
# The one-package fixture must work even when its parent repository gate has
# a hosted receipt for the real workspace. These foreign values must not be
# consumed by the fixture's package-runner subprocesses.
export AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/foreign-hosted-receipt.json"
export AI_COCKPIT_VERIFICATION_ORCHESTRATION="$tmp/foreign-hosted-orchestration.json"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="$ordinary"
run_fixture_workspace_package_tests() {
  # Hosted verification can inherit a runner from its source checkout; this
  # fixture's package subprocesses must use the isolated checkout's runner.
  env -u AI_COCKPIT_VERIFICATION_RECEIPT \
    -u AI_COCKPIT_VERIFICATION_ORCHESTRATION \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="$runner" \
    "$root/tests/ci/run_workspace_package_tests.sh" "$@"
}
RUSTC="$fake_rustc" \
  EXPECTED_RUNNER="$runner" \
  WORKSPACE_TEST_WORKERS=1 \
  WORKSPACE_TEST_THREADS=4 \
  run_fixture_workspace_package_tests \
    --metadata "$tmp/workspace-metadata.json" \
    --cargo "$fake_cargo" \
    --report "$tmp/workspace-runner-report.json"
jq -e '.state == "passed" and .executed == ["fixture-package"]' \
  "$tmp/workspace-runner-report.json" >/dev/null

# A Cargo runner for another platform must not block the host package gate.
mkdir -p "$tmp/cargo-home-other-target"
printf '%s\n' \
  "[target.'cfg(windows)']" \
  "runner = \"$tmp/bin/custom-cargo-runner\"" \
  >"$tmp/cargo-home-other-target/config.toml"
CARGO_HOME="$tmp/cargo-home-other-target" \
  RUSTC="$fake_rustc" \
  EXPECTED_RUNNER="$runner" \
  WORKSPACE_TEST_WORKERS=1 \
  WORKSPACE_TEST_THREADS=4 \
  run_fixture_workspace_package_tests \
    --metadata "$tmp/workspace-metadata.json" \
    --cargo "$fake_cargo" \
    --report "$tmp/workspace-other-target-runner-report.json"
jq -e '.state == "passed" and .executed == ["fixture-package"]' \
  "$tmp/workspace-other-target-runner-report.json" >/dev/null

# A target runner supplied by Cargo TOML must not be silently shadowed by the
# process-observer runner injected by the package gate.
mkdir -p "$tmp/cargo-home"
printf '%s\n' \
  "[target.'cfg(unix)']" \
  "runner = \"$tmp/bin/custom-cargo-runner\"" \
  >"$tmp/cargo-home/config.toml"
if CARGO_HOME="$tmp/cargo-home" \
  RUSTC="$fake_rustc" \
  EXPECTED_RUNNER="$runner" \
  WORKSPACE_TEST_WORKERS=1 \
  WORKSPACE_TEST_THREADS=4 \
  run_fixture_workspace_package_tests \
    --metadata "$tmp/workspace-metadata.json" \
    --cargo "$fake_cargo" \
    --report "$tmp/workspace-configured-runner-report.json"; then
  printf 'workspace package runner silently replaced a TOML-configured Cargo runner\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .failurePhase == "runner_setup" and (.failureDiagnosticTail | contains("configured Cargo target runner"))' \
  "$tmp/workspace-configured-runner-report.json" >/dev/null

# A conflicting runner in a recursively included Cargo config must not be
# hidden when the package gate injects the process-observer runner.
mkdir -p "$tmp/cargo-home-included/nested"
printf '%s\n' \
  'include = ["nested/first.toml"]' \
  >"$tmp/cargo-home-included/config.toml"
printf '%s\n' \
  'include = ["../runner.toml"]' \
  >"$tmp/cargo-home-included/nested/first.toml"
printf '%s\n' \
  "[target.'cfg(unix)']" \
  "runner = \"$tmp/bin/custom-cargo-runner\"" \
  >"$tmp/cargo-home-included/runner.toml"
if CARGO_HOME="$tmp/cargo-home-included" \
  RUSTC="$fake_rustc" \
  EXPECTED_RUNNER="$runner" \
  WORKSPACE_TEST_WORKERS=1 \
  WORKSPACE_TEST_THREADS=4 \
  run_fixture_workspace_package_tests \
    --metadata "$tmp/workspace-metadata.json" \
    --cargo "$fake_cargo" \
    --report "$tmp/workspace-included-runner-report.json"; then
  printf 'workspace package runner silently replaced a Cargo runner from an included config\n' >&2
  exit 1
fi
jq -e \
  '.state == "failed" and .failurePhase == "runner_setup" and (.failureDiagnosticTail | contains("configured Cargo target runner") and contains("runner.toml"))' \
  "$tmp/workspace-included-runner-report.json" >/dev/null

# An optional include may be absent, but an existing directory is not a
# configuration file and must fail closed instead of being treated as absent.
mkdir -p "$tmp/cargo-home-optional-directory/not-a-config.toml"
printf '%s\n' \
  'include = [{ path = "not-a-config.toml", optional = true }]' \
  >"$tmp/cargo-home-optional-directory/config.toml"
if CARGO_HOME="$tmp/cargo-home-optional-directory" \
  RUSTC="$fake_rustc" \
  EXPECTED_RUNNER="$runner" \
  WORKSPACE_TEST_WORKERS=1 \
  WORKSPACE_TEST_THREADS=4 \
  run_fixture_workspace_package_tests \
    --metadata "$tmp/workspace-metadata.json" \
    --cargo "$fake_cargo" \
    --report "$tmp/workspace-optional-directory-report.json"; then
  printf 'workspace package runner treated an existing directory as an absent optional Cargo include\n' >&2
  exit 1
fi
jq -e \
  '.state == "failed" and .failurePhase == "runner_setup" and (.failureDiagnosticTail | contains("included Cargo configuration is missing or not a regular file"))' \
  "$tmp/workspace-optional-directory-report.json" >/dev/null

# Process-observer targets scan process state outside their own test binary.
# They must wait for package tests holding shared locks, even when Cargo runs
# separate package binaries concurrently.
if command -v flock >/dev/null 2>&1 || command -v python3 >/dev/null 2>&1; then
  lock_file="$tmp/process-observer.lock"
  lock_attempt="$tmp/sensitive-lock-attempt"
  mock_bin="$tmp/lock-bin"
  ordinary_hold="$tmp/bin/ordinary-hold-0123456789abcdef"
  ordinary_parallel="$tmp/bin/ordinary-parallel-0123456789abcdef"
  sensitive_probe="$tmp/bin/cockpit_verification-probe-0123456789abcdef"
  mkdir -p "$mock_bin"
  if real_lock=$(command -v flock 2>/dev/null); then
    lock_tool=flock
  else
    real_lock=$(command -v python3)
    lock_tool=python3
  fi
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    ': > "$LOCK_ATTEMPT"' \
    'exec "$REAL_LOCK_TOOL" "$@"' \
    >"$mock_bin/$lock_tool"
  chmod +x "$mock_bin/$lock_tool"
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    ': > "$ORDINARY_STARTED"' \
    'while [[ ! -e "$ORDINARY_RELEASE" ]]; do sleep 0.01; done' \
    >"$ordinary_hold"
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    ': > "$ORDINARY_PARALLEL_STARTED"' \
    >"$ordinary_parallel"
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    ': > "$SENSITIVE_STARTED"' \
    >"$sensitive_probe"
  chmod +x "$ordinary_hold" "$ordinary_parallel" "$sensitive_probe"

  PATH="$mock_bin:$PATH" \
    REAL_LOCK_TOOL="$real_lock" \
    LOCK_ATTEMPT="$tmp/ordinary-lock-attempt" \
    AI_COCKPIT_PROCESS_OBSERVER_LOCK="$lock_file" \
    ORDINARY_STARTED="$tmp/ordinary-started" \
    ORDINARY_RELEASE="$tmp/ordinary-release" \
    "$runner" "$ordinary_hold" &
  ordinary_pid=$!
  for _ in {1..200}; do
    [[ -e "$tmp/ordinary-started" ]] && break
    sleep 0.01
  done
  test -e "$tmp/ordinary-started"

  PATH="$mock_bin:$PATH" \
    REAL_LOCK_TOOL="$real_lock" \
    LOCK_ATTEMPT="$tmp/parallel-lock-attempt" \
    AI_COCKPIT_PROCESS_OBSERVER_LOCK="$lock_file" \
    ORDINARY_PARALLEL_STARTED="$tmp/ordinary-parallel-started" \
    "$runner" "$ordinary_parallel" &
  parallel_pid=$!
  for _ in {1..200}; do
    [[ -e "$tmp/ordinary-parallel-started" ]] && break
    sleep 0.01
  done
  if [[ ! -e "$tmp/ordinary-parallel-started" ]]; then
    : >"$tmp/ordinary-release"
    wait "$ordinary_pid"
    wait "$parallel_pid"
    printf 'ordinary package tests were serialized instead of sharing the observer lock\n' >&2
    exit 1
  fi
  wait "$parallel_pid"

  PATH="$mock_bin:$PATH" \
    REAL_LOCK_TOOL="$real_lock" \
    LOCK_ATTEMPT="$lock_attempt" \
    AI_COCKPIT_PROCESS_OBSERVER_LOCK="$lock_file" \
    SENSITIVE_STARTED="$tmp/sensitive-started" \
    "$runner" "$sensitive_probe" &
  sensitive_pid=$!
  for _ in {1..200}; do
    [[ -e "$lock_attempt" ]] && break
    sleep 0.01
  done
  if [[ ! -e "$lock_attempt" ]]; then
    : >"$tmp/ordinary-release"
    wait "$ordinary_pid"
    wait "$sensitive_pid"
    printf 'process-observer target did not enter the cross-process lock\n' >&2
    exit 1
  fi
  sleep 0.1
  if [[ -e "$tmp/sensitive-started" ]]; then
    : >"$tmp/ordinary-release"
    wait "$ordinary_pid"
    wait "$sensitive_pid"
    printf 'process-observer target overlapped a package test holding the shared lock\n' >&2
    exit 1
  fi

  : >"$tmp/ordinary-release"
  wait "$ordinary_pid"
  wait "$sensitive_pid"
  test -e "$tmp/sensitive-started"
else
  printf 'process-observer runner requires flock or Python 3 for safe cross-process isolation\n' >&2
  exit 1
fi

printf 'process-observer test runner boundaries passed\n'
