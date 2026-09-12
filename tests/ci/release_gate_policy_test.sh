#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"
workflow="$repo_root/.github/workflows/release.yml"
manifest="$repo_root/tests/ci/repository_gate_manifest.json"
resolver="$repo_root/tests/ci/resolve_work_item.sh"

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
require 'target/release/ai-cockpit gate-plan' 'release must derive a typed route in the shared Rust application'
if grep -Fq -- 'python3 tests/ci/quality_route.py' "$workflow"; then
  printf 'release gate policy failure: compatibility Python route must not be a production release path\n' >&2
  exit 1
fi
require '--stage release' 'release routing must use the release stage floor'
require '--profile strict' 'release routing must explicitly require the strict profile'
require '--route-receipt' 'release gate execution must consume the typed route receipt'
require 'target/release-quality-route.json' 'release route receipt must be retained as evidence'
require 'release_input_preflight:' 'cheap release input preflight must run before Runtime compilation'
require 'work_item_id:' 'recovery must expose an explicit Work Item identity input'
require 'resolve_work_item.sh' 'release selection must use the shared explicit identity resolver'
require 'if [[ -n "$INPUT_WORK_ITEM_ID" ]]; then' 'optional Work Item input must be appended only when present'
require 'if [[ -n "$INPUT_CONTRACT_PATH" ]]; then' 'optional Contract input must be appended only when present'
require 'if [[ -n "$INPUT_HANDOFF_RUN_ID" ]]; then' 'optional handoff input must be appended only when present'
if grep -Fq -- "--release-source-revision ''" "$workflow"; then
  printf 'release gate policy failure: empty optional resolver arguments are forbidden\n' >&2
  exit 1
fi
grep -Fq 'work_item_id_required' "$resolver" || {
  printf 'release gate policy failure: recovery without an explicit Work Item identity must fail early\n' >&2
  exit 1
}
grep -Fq 'all_contracts=()' "$resolver" || {
  printf 'release gate policy failure: the shared resolver must inspect active Contracts\n' >&2
  exit 1
}
grep -Fq 'work_item_contract_ambiguous' "$resolver" || {
  printf 'release gate policy failure: ambiguous active Contracts must fail closed\n' >&2
  exit 1
}
require 'staged_adopter_acceptance:' 'release must gate publication on staged adopter acceptance'
require 'staged_adopter_upgrade_acceptance:' 'release must gate publication on staged N-1 acceptance'
require '--candidate-dir' 'staged adopter acceptance must consume the candidate artifact'
require '--to-candidate-dir' 'staged N-1 acceptance must consume the candidate artifact'
require 'adopterAcceptance == "not_applicable"' 'release close must validate the first-release N-1 not-applicable receipt'
require 'releasePublished == true' 'release close must bind a not-applicable N-1 result to the published Release'
require 'needs.publish.result == '\''success'\''' 'recovery public acceptance must wait for publication success'
require 'needs.publish_handoff.result == '\''success'\''' 'recovery public acceptance must wait for the publication handoff'
require 'if: always() && needs.publish_handoff.result == '\''success'\''' 'close must download the handoff receipt only after a successful handoff job'
require 'if: always() && needs.post_release_version_consistency.result == '\''success'\''' 'close must download the version receipt only after a successful consistency job'
require 'if: always() && needs.adopter_acceptance.result == '\''success'\''' 'close must download the install receipt only after a successful acceptance job'
require 'if: always() && needs.adopter_upgrade_acceptance.result == '\''success'\''' 'close must download the upgrade receipt only after a successful acceptance job'
require 'if [[ "$GITHUB_EVENT_NAME" == push ]]; then' 'tag-triggered handoff must retain the source/workflow identity equality guard'
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
