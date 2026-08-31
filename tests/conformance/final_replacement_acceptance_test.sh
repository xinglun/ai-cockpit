#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
script="$root/tests/conformance/final_replacement_acceptance.sh"
test -x "$script"
bash -n "$script"
grep -q -- '--repo' "$script"
grep -q -- 'runtimeDigest' "$script"
grep -q -- 'referenceSourceCommit' "$script"
public_reference='spirex-ds-dev/'"ai-cockpit-template"
if grep -q "$public_reference" "$script"; then
  echo 'final acceptance must not access the public reference repository' >&2
  exit 1
fi
grep -q -- 'SHA256SUMS' "$script"
grep -q -- 'cargo test -p cockpit-core --test adversarial_v2' "$script"
grep -q -- 'no-copied-v1-runtime' "$script"
if grep -Eq 'cargo (run|build)' "$script"; then
  echo 'final acceptance must not build or run a source fallback binary' >&2
  exit 1
fi
echo 'final replacement acceptance harness static checks passed'
