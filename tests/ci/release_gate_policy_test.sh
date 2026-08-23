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
require 'tests/ci/run_workspace_package_tests.sh' 'release tests must derive workspace packages from cargo metadata'
require 'tests/ci/run_repository_gates.py' 'release must run the canonical repository gate manifest'
require 'tests/ci/repository_gate_manifest.json' 'release must bind the canonical repository gate manifest'

if grep -Eq '^[[:space:]]+for package in' "$workflow"; then
  printf 'release gate policy failure: hard-coded package loop is forbidden\n' >&2
  exit 1
fi

# Keep the old one-shot form only as explanatory text. An executable release
# command must not run the workspace as one concurrent Cargo test invocation.
if awk '!/^[[:space:]]*#/ && /cargo test --workspace --all-targets --all-features --quiet/' "$workflow" | grep -q .; then
  printf 'release gate policy failure: one-shot workspace tests are not deterministic\n' >&2
  exit 1
fi

printf 'release gate policy passed\n'
