#!/usr/bin/env bash
set -euo pipefail

if (($# == 0)); then
  printf 'Cargo test runner received no test executable\n' >&2
  exit 2
fi

test_binary=$1
shift

case "${test_binary##*/}" in
  collaboration_admission-*|composition-*) ;;
  *) exec "$test_binary" "$@" ;;
esac

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

export RUST_TEST_THREADS=1
printf 'serializing process-observer test target: %s\n' "${test_binary##*/}" >&2
exec "$test_binary" "${test_args[@]}"
