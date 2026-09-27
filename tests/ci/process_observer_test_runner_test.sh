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

ordinary="$tmp/bin/ordinary_test-0123456789abcdef"
write_fake_test_binary "$ordinary"
OBSERVED_THREADS="$tmp/ordinary-threads" OBSERVED_ARGS="$tmp/ordinary-args" RUST_TEST_THREADS=8 \
  "$runner" "$ordinary" --test-threads=4 ordinary_filter
test "$(<"$tmp/ordinary-threads")" = 8
diff -u <(printf '%s\n' --test-threads=4 ordinary_filter) "$tmp/ordinary-args"

printf 'process-observer test runner boundaries passed\n'
