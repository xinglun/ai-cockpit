#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"
script="$repo_root/tests/ci/runtime_verify_shadow.sh"

[[ -x "$script" ]] || { printf 'Runtime shadow script must be executable\n' >&2; exit 1; }
bash -n "$script"
if grep -Eq 'cargo[[:space:]]+(build|run)|target/debug/ai-cockpit|workspace binary' "$script"; then
  printf 'Runtime shadow must not use a source or workspace binary fallback\n' >&2
  exit 1
fi
grep -q -- 'archive_sha256=' "$script"
grep -q -- 'binary_sha256=' "$script"
grep -q -- 'cargoShadowRequired:true' "$script"
grep -q -- '--proto' "$script"
grep -q -- 'https://github.com/xinglun/ai-cockpit/releases/download' "$script"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-ci-shadow-test.XXXXXX")"
cleanup() { find "$tmp_root" -depth -mindepth 0 -delete 2>/dev/null || true; }
trap cleanup EXIT
if AI_COCKPIT_RUNTIME_TAG=v0.2.14 "$script" "$repo_root" "$tmp_root/invalid.json" >/dev/null 2>&1; then
  printf 'unsupported Runtime baseline must fail closed\n' >&2
  exit 1
fi
printf 'Runtime shadow harness policy passed\n'
