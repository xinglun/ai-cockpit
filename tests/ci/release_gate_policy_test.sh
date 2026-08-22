#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"
workflow="$repo_root/.github/workflows/release.yml"

[[ -f "$workflow" ]] || { printf 'release workflow is missing\n' >&2; exit 1; }

require() {
  local pattern=$1
  local message=$2
  if ! grep -Fq -- "$pattern" "$workflow"; then
    printf 'release gate policy failure: %s\n' "$message" >&2
    exit 1
  fi
}

require 'name: Run source quality gates' 'source quality step is required'
require '# serial strategy used by CI' 'release strategy must document CI alignment'
require 'for package in \' 'release tests must enumerate packages'
require 'cargo test -p "$package" --all-targets -- --test-threads=1' \
  'release tests must run each package with one test thread'
require 'tests/ci/runtime_verify_shadow_test.sh' 'shadow boundary policy must run'
require 'tests/ci/release_gate_policy_test.sh' 'release gate policy test must run'

# Keep the old one-shot form only as explanatory text. An executable release
# command must not run the workspace as one concurrent Cargo test invocation.
if awk '!/^[[:space:]]*#/ && /cargo test --workspace --all-targets --all-features --quiet/' "$workflow" | grep -q .; then
  printf 'release gate policy failure: one-shot workspace tests are not deterministic\n' >&2
  exit 1
fi

printf 'release gate policy passed\n'
