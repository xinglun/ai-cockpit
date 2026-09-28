#!/usr/bin/env bash
set -euo pipefail

if (($# == 0)); then
  printf 'Cargo test runner received no test executable\n' >&2
  exit 2
fi

test_binary=$1
shift
test_argument_count=$#

lock_mode=shared
case "${test_binary##*/}" in
  collaboration_admission-*|composition-*)
    lock_mode=exclusive
    ;;
esac

if [[ "$lock_mode" == exclusive ]]; then
  test_args=(--test-threads=1)
  skip_test_threads_value=false
  for argument in "$@"; do
    if [[ "$skip_test_threads_value" == true ]]; then
      skip_test_threads_value=false
      continue
    fi
    case "$argument" in
      --test-threads)
        skip_test_threads_value=true
        ;;
      --test-threads=*) ;;
      *) test_args+=("$argument") ;;
    esac
  done
else
  test_args=("$@")
fi

if [[ "$lock_mode" == exclusive ]]; then
  export RUST_TEST_THREADS=1
  printf 'serializing process-observer test target: %s\n' "${test_binary##*/}" >&2
fi

user_id=${UID:-$(id -u)}
lock_file=${AI_COCKPIT_PROCESS_OBSERVER_LOCK:-${TMPDIR:-/tmp}/ai-cockpit-process-observer-${user_id}.lock}
if command -v flock >/dev/null 2>&1; then
  if [[ "$lock_mode" == exclusive ]]; then
    exec flock -x "$lock_file" "$test_binary" "${test_args[@]}"
  fi
  if ((test_argument_count == 0)); then
    exec flock -s "$lock_file" "$test_binary"
  fi
  exec flock -s "$lock_file" "$test_binary" "${test_args[@]}"
fi

if command -v python3 >/dev/null 2>&1; then
  if ((test_argument_count == 0)); then
    exec python3 -c 'import fcntl, os, sys; fd = os.open(sys.argv[1], os.O_CREAT | os.O_RDWR, 0o600); fcntl.flock(fd, fcntl.LOCK_EX if sys.argv[2] == "exclusive" else fcntl.LOCK_SH); os.set_inheritable(fd, True); os.execvpe(sys.argv[3], sys.argv[3:], os.environ)' \
      "$lock_file" "$lock_mode" "$test_binary"
  fi
  exec python3 -c 'import fcntl, os, sys; fd = os.open(sys.argv[1], os.O_CREAT | os.O_RDWR, 0o600); fcntl.flock(fd, fcntl.LOCK_EX if sys.argv[2] == "exclusive" else fcntl.LOCK_SH); os.set_inheritable(fd, True); os.execvpe(sys.argv[3], sys.argv[3:], os.environ)' \
    "$lock_file" "$lock_mode" "$test_binary" "${test_args[@]}"
fi

printf 'process-observer runner requires flock or Python 3 for cross-process isolation\n' >&2
exit 2
