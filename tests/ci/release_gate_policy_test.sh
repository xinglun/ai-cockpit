#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"
workflow="$repo_root/.github/workflows/release.yml"
manifest="$repo_root/tests/ci/repository_gate_manifest.json"

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
require 'tests/ci/run_repository_gates.py' 'release must run the canonical repository gate manifest'
require 'tests/ci/repository_gate_manifest.json' 'release must bind the canonical repository gate manifest'
require 'tests/ci/quality_route.py' 'release must derive a typed route from repository facts'
require '--stage release' 'release routing must use the release stage floor'
require '--profile strict' 'release routing must explicitly require the strict profile'
require '--route-receipt' 'release gate execution must consume the typed route receipt'
require 'target/release-quality-route.json' 'release route receipt must be retained as evidence'
require 'staged_adopter_acceptance:' 'release must gate publication on staged adopter acceptance'
require 'staged_adopter_upgrade_acceptance:' 'release must gate publication on staged N-1 acceptance'
require '--candidate-dir' 'staged adopter acceptance must consume the candidate artifact'
require '--to-candidate-dir' 'staged N-1 acceptance must consume the candidate artifact'
grep -Fq 'tests/ci/run_workspace_package_tests.sh' "$manifest" || {
  printf 'release gate policy failure: canonical manifest must derive workspace packages from cargo metadata\n' >&2
  exit 1
}
grep -Fq '"workspace_clippy"' "$manifest" || {
  printf 'release gate policy failure: canonical manifest must retain Clippy\n' >&2
  exit 1
}
grep -Fq '"workspace_format"' "$manifest" || {
  printf 'release gate policy failure: canonical manifest must retain rustfmt\n' >&2
  exit 1
}

if grep -Fq -- '--command' "$workflow"; then
  printf 'release gate policy failure: arbitrary command substitution is forbidden\n' >&2
  exit 1
fi

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
