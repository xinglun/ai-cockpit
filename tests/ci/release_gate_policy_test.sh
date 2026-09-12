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
require 'Fail when the close receipt is not passed' 'close must fail the job when the persisted close receipt is failed'
require '.state == "passed"' 'close must distinguish a passed receipt from a green summary step'
require 'post_release_acceptance' 'release must expose an explicit post-release-only acceptance mode'
require 'reuse_run_id' 'post-release-only acceptance must identify reusable prior-run evidence'
require 'post_release_helper' 'post-release-only acceptance must restore a prebuilt helper without rebuilding'
require 'gh run download "$REUSE_RUN_ID"' 'post-release-only acceptance must download the selected prior-run helper'
require 'workflowName' 'post-release-only acceptance must bind reused helper evidence to the release workflow identity'
require 'headSha' 'post-release-only acceptance must retain the reused workflow execution identity'
require 'releaseToolJob' 'post-release-only acceptance must bind the reused helper to its successful build job'
require 'Preflight helper handoff protocol compatibility' 'reused helper protocol must be checked before public acceptance'
require 'tools/cockpit-release handoff' 'helper compatibility must exercise the actual restored helper'
require 'probe_release_commit' 'helper compatibility must use a distinct synthetic release commit'
require 'post-release-helper-compatibility' 'helper compatibility result must be persisted'
require 'helper_handoff_protocol_incompatible' 'helper protocol failures must have a structured failure code'
require 'remoteWrites:false' 'helper compatibility probe must declare that it performs no remote writes'
require 'cargo build --locked --release --package cockpit-release' 'incompatible helper recovery may rebuild only the release helper'
require 'post-release-helper-repair' 'helper-only repair result must be persisted'
require 'buildCount' 'helper-only repair result must record its build count'
require 'productPackagesRebuilt:false' 'helper-only repair must not rebuild product packages'
require 'verify-provider-release' 'post-release-only acceptance must verify the public Release identity'
require 'releases/download/$TAG/' 'post-release-only acceptance must download immutable public Release assets'
require "github.event.inputs.post_release_acceptance == 'true'" 'post-release-only acceptance must be explicit'
require "github.event.inputs.post_release_acceptance != 'true'" 'publication jobs must be skipped for post-release-only acceptance'
require 'needs: [publish, release_input_preflight]' 'version consistency must run without publish in post-release-only mode'
require 'needs: [publish_handoff, release_tools, post_release_helper]' 'install must use the mode-specific helper dependency'
require 'needs: [publish, publish_handoff, release_tools, post_release_helper]' 'upgrade must use the mode-specific helper dependency'
require 'needs.post_release_helper.result' 'public acceptance must wait for helper restoration'
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
