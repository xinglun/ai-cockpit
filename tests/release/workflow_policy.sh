#!/usr/bin/env bash
set -euo pipefail

workflow=${1:?usage: workflow_policy.sh <workflow>}
repo_root="$(cd "$(dirname "$workflow")/../.." && pwd -P)"
gate_manifest="$repo_root/tests/ci/repository_gate_manifest.json"

if command -v rg >/dev/null 2>&1; then
  search() { rg -n --pcre2 -- "$1" "$2"; }
  extract() { rg -oP "$1" "$2"; }
else
  # GitHub's Ubuntu runner does not guarantee ripgrep. GNU grep's PCRE mode
  # covers the same patterns used by this policy script and keeps the check
  # self-contained instead of making the workflow install another tool.
  search() { grep -nP -- "$1" "$2"; }
  extract() { grep -oP "$1" "$2"; }
fi

fail_if_match() {
  local pattern=$1
  local message=$2
  if search "$pattern" "$workflow" >/dev/null; then
    printf 'policy failure: %s\n' "$message" >&2
    if ! search "$pattern" "$workflow" >&2; then :; fi
    exit 1
  fi
}

require_match() {
  local pattern=$1
  local message=$2
  if ! search "$pattern" "$workflow" >/dev/null; then
    printf 'policy failure: %s\n' "$message" >&2
    exit 1
  fi
}

fail_if_match 'ubuntu-latest|windows-latest' 'moving runner aliases are not allowed'
fail_if_match 'macos-13' 'retired macOS 13 runners are not allowed'
fail_if_match 'macos-14' 'macOS 14 runner is in the deprecation window'
fail_if_match '^\s*uses:\s*[^#]+@(v[0-9]|stable|main|master)(?:\s|$)' 'actions must be pinned to full commit SHAs'
fail_if_match '^permissions:\s*$' 'workflow-wide permissions are not allowed'
fail_if_match 'curl\s+[^|]+\|\s*(sh|bash)' 'shell bootstrap installers are not part of release'
fail_if_match 'homebrew-tap.*(git push|contents: write)' 'release workflow must not mutate the external tap'
fail_if_match 'files:\s*dist/\*\s*$' 'publication must use an explicit asset allowlist'
fail_if_match 'dist/\*\.spdx\.json' 'publication and attestation must use the target-bound SBOM allowlist'

while IFS= read -r action_ref; do
  if [[ ! "$action_ref" =~ ^[0-9a-f]{40}$ ]]; then
    printf 'policy failure: action reference is not a full lowercase commit SHA: %s\n' "$action_ref" >&2
    exit 1
  fi
done < <(extract '^\s*-\s*uses:\s*[^@]+@\K[^[:space:]]+' "$workflow")
awk '
  /^  [A-Za-z0-9_-]+:/ {
    job=$0
    sub(/^  /, "", job)
    sub(/:.*/, "", job)
  }
  job != "publish" && /^\s+contents:\s+write\s*$/ { exit 1 }
' "$workflow" || {
  printf 'policy failure: only publish may receive contents write\n' >&2
  exit 1
}
require_match '^\s*workflow_dispatch:' 'manual verification trigger is required'
require_match '^\s*push:\s*$' 'tag trigger is required'
require_match 'tags:\s*\['"'"'v\*'"'"'\]' 'only semantic v tags trigger publication'
require_match 'cockpit-release' 'canonical release tooling must run in the workflow'
require_match '^  release_tools:' 'shared release acceptance tooling must be built once'
require_match '^    needs: \[release_preflight, release_tools\]$' 'build must wait for the release_tools producer before downloading helpers'
require_match 'cockpit-release-tool-ubuntu-x86_64' 'Linux release jobs must consume the prebuilt release helper'
require_match 'artifact:[[:space:]]*macos-arm64' 'macOS ARM release jobs must consume a prebuilt platform helper'
require_match 'artifact:[[:space:]]*windows-x86_64' 'Windows release jobs must consume a prebuilt platform helper'
fail_if_match 'cargo run --locked --package cockpit-release' 'release jobs must not implicitly rebuild the release helper'
require_match 'COCKPIT_RELEASE_BIN' 'adopter acceptance must use the identity-bound resume helper'
require_match 'ai-cockpit-publish-handoff' 'public acceptance must consume the post-publication handoff artifact'
require_match '\-\-publish-handoff' 'public acceptance must bind the publication handoff'
require_match 'release-manifest\.json' 'canonical manifest must be emitted'
require_match 'SHA256SUMS' 'canonical checksum set must be emitted'
require_match 'all_contracts' 'release routing must inspect all active Contracts before selecting one'
require_match 'selection_revision' 'release routing must bind Contract selection to immutable release identity'
require_match 'no active Contract matches the release identity' 'release routing must fail closed when successor selection is ambiguous'
require_match 'upload-artifact:[[:space:]]*false' 'SBOM action must not upload an orphan default artifact'
require_match 'upload-release-assets:[[:space:]]*false' 'SBOM action must not publish an orphan default SBOM'
require_match '(cockpit-release -- bind-sbom|tools/\$\{\{ matrix\.helper_binary \}\} bind-sbom)' 'each target SBOM must be bound to its packaged archive and executable'
require_match '(cockpit-release -- checksums|COCKPIT_RELEASE_BIN.* checksums)' 'checksums must be generated after all public assets exist'
require_match 'dist/ai-cockpit-v\*-\*\.spdx\.json' 'publication and attestation must select only target-bound SBOM filenames'
require_match 'brew test' 'Homebrew fixture test must be defined'
require_match 'ai-cockpit --version' 'installed binary version smoke must be defined'
require_match '^  publish:' 'publish job must be present'
require_match 'needs:' 'publish must depend on verification jobs'
require_match '^  release_preflight:' 'cheap release preflight must run before build and expensive verification'
require_match '^  source_quality:' 'source-quality job must be present'
require_match '^  release_policy:' 'release-policy job must be present'
require_match '^  attest:' 'final attestation job must be present'
require_match '^  publish_handoff:' 'post-publication handoff job must be present'
require_match '^  post_release_version_consistency:' 'post-publication version consistency job must be present'
require_match '^  adopter_acceptance:' 'post-release adopter acceptance job must be present'
require_match '^  adopter_upgrade_acceptance:' 'post-release N-1 upgrade acceptance job must be present'
require_match '^  release_close:' 'release close barrier job must be present'
require_match 'publish_existing_tag' 'immutable tag recovery must have an explicit workflow input'
require_match 'github\.event\.inputs\.publish_existing_tag == '\''true'\''' 'immutable tag recovery must be explicitly enabled'
require_match '^  staged_adopter_acceptance:' 'pre-publication staged adopter acceptance job must be present'
require_match '^  staged_adopter_upgrade_acceptance:' 'pre-publication staged N-1 acceptance job must be present'
require_match 'tests/ci/run_repository_gates\.py' 'source quality must run the canonical repository gate manifest'
require_match 'target/release/ai-cockpit gate-plan' 'release preflight must derive the typed repository route in Rust'
fail_if_match 'python3 tests/ci/quality_route\.py' 'release must not run the compatibility Python route'
require_match '--stage release' 'source quality must use the release route floor'
require_match '--profile strict' 'source quality must require the strict route'
require_match '--route-receipt' 'source quality must consume the typed route receipt'
fail_if_match '--command' 'release workflow must not substitute an arbitrary verification command for canonical gates'
grep -Fq '"workspace_format"' "$gate_manifest" || {
  printf 'policy failure: canonical repository manifest must retain rustfmt\n' >&2
  exit 1
}
grep -Fq '"workspace_clippy"' "$gate_manifest" || {
  printf 'policy failure: canonical repository manifest must retain Clippy\n' >&2
  exit 1
}
grep -Fq 'tests/ci/run_workspace_package_tests.sh' "$gate_manifest" || {
  printf 'policy failure: canonical repository manifest must derive workspace package tests from cargo metadata\n' >&2
  exit 1
}
require_match 'cargo metadata --locked' 'source quality must gate workspace metadata'
require_match 'cargoLockSha256' 'release identity must bind Cargo.lock'
require_match "jq -er '.before'" 'tag policy must reject mutable tag updates'
require_match 'gh api .*releases/tags' 'release policy must inspect an existing provider Release'
require_match 'verify-provider-release' 'existing provider Releases must be checked by the shared Rust identity verifier'
require_match 'provider-release-state' 'provider Release identity must be persisted for publish recovery'
require_match "if: steps.release_identity.outputs.state == 'new'" 'publish must skip creation when the existing Release identity is reusable'
require_match '--provider-release-id' 'handoff must bind the provider Release identity'
require_match 'actions/attest-build-provenance@' 'final candidate/handoff attestation must be defined'
require_match 'dist/release-manifest\.json' 'published assets must include the canonical manifest'
require_match 'dist/Formula/ai-cockpit\.rb' 'published assets must include the Formula'
require_match 'needs: \[build, aggregate, source_quality, release_policy, verify, smoke_homebrew, smoke_linux, smoke_windows, staged_adopter_acceptance, staged_adopter_upgrade_acceptance, attest\]' 'publish must depend on every final gate, including staged adopter acceptance'
require_match '^  publish_handoff:' 'handoff must be a separate post-publication job'
require_match 'publish_handoff:[[:space:]]*$' 'post-publication handoff job must be addressable'
require_match 'adopter_acceptance:[[:space:]]*$' 'post-release adopter acceptance job must be addressable'
require_match 'tests/release/adopter_acceptance\.sh' 'post-release job must invoke the adopter acceptance harness'
require_match '--candidate-dir' 'staged adopter acceptance must consume the candidate artifact'
require_match '--to-candidate-dir' 'staged N-1 acceptance must consume the candidate artifact'
require_match 'needs: \[publish_handoff, release_tools\]' 'adopter acceptance must run after publication handoff and shared release tooling'
require_match 'needs: \[publish, publish_handoff, release_tools\]' 'N-1 acceptance must run after publication and handoff'
require_match 'tests/release/version_consistency\.sh' 'release workflow must run the version consistency gate'
require_match 'tests/ci/repository_gate_manifest\.json' 'release workflow must bind all repository policy gates'
require_match '--post-release' 'post-publication version consistency must validate public assets'
require_match 'github\.event_name == '\''push'\'' && startsWith\(github\.ref, '\''refs/tags/'\''\)' 'adopter acceptance must retain the tag-triggered post-publication path'
require_match 'if: always\(\)' 'adopter acceptance evidence must upload after success or failure'
require_match 'tests/release/adopter_upgrade_acceptance\.sh' 'N-1 post-release job must invoke the public-artifact upgrade harness'
require_match 'INPUT_FROM_TAG' 'manual N-1 acceptance must receive an explicit from tag'
require_match 'INPUT_TO_TAG' 'manual N-1 acceptance must receive an explicit to tag'
require_match 'HANDOFF_RUN_ID' 'manual public acceptance must consume a handoff from a completed publication run'
require_match 'gh run download' 'manual public acceptance must download the immutable publication handoff'
require_match 'steps\.release_pair\.outputs\.from_tag' 'N-1 execution must use the resolved immutable Release pair'
require_match 'adopterAcceptance:"not_applicable"' 'first-release N-1 boundary must be explicit'
require_match 'name: Upload N-1 upgrade acceptance evidence' 'N-1 evidence must be uploaded independently'
require_match 'release close blocked' 'release close must fail closed when required acceptance is absent'
require_match 'needs: \[publish_handoff, post_release_version_consistency, adopter_acceptance, adopter_upgrade_acceptance\]' 'release close must wait for public acceptance and consistency receipts'
require_match '^    needs: \[publish\]$' 'public version consistency must run in parallel with post-release acceptance'
require_match 'refs/tags/\$\{tag\}\^\{\}' 'publish must compare the peeled tag commit'
require_match 'chmod \+x target/release/ai-cockpit' 'source quality must restore executable permissions after artifact download'
if grep -Fq '\"cockpit-release\"' "$workflow"; then
  printf 'policy failure: release helper checksum extraction must not contain escaped quotes in Bash awk source\n' >&2
  exit 1
fi

if ! bash -n <(
  awk '
    /^      - name: Write release close receipt$/ { in_step=1; next }
    in_step && /^        run: \|$/ { in_run=1; next }
    in_run && /^      - name:/ { exit }
    in_run {
      sub(/^          /, "")
      print
    }
  ' "$workflow"
); then
  printf 'policy failure: release close embedded Bash is not syntactically valid\n' >&2
  exit 1
fi

awk '
  /^  [A-Za-z0-9_-]+:/ {
    job=$0
    sub(/^  /, "", job)
    sub(/:.*/, "", job)
  }
  job == "build" && /id-token:[[:space:]]+write|attestations:[[:space:]]+write/ { exit 1 }
' "$workflow" || {
  printf 'policy failure: build jobs must not receive attestation write permissions\n' >&2
  exit 1
}

handoff_jobs="$(awk '
  /^  [A-Za-z0-9_-]+:/ {
    job=$0
    sub(/^  /, "", job)
    sub(/:.*/, "", job)
  }
  /cargo run .*-- handoff|tools\/cockpit-release handoff/ { print job }
' "$workflow")"
if [[ "$handoff_jobs" != "publish_handoff" ]]; then
  printf 'policy failure: handoff generation must occur only after publication (jobs: %s)\n' "${handoff_jobs:-none}" >&2
  exit 1
fi

require_match 'github\.event_name == '\''push'\'' && startsWith\(github\.ref, '\''refs/tags/'\''\)' 'publish must retain the tag-triggered path'
require_match 'github\.event_name == '\''workflow_dispatch'\'' && github\.event\.inputs\.publish_existing_tag == '\''true'\''' 'publish recovery must require explicit immutable-tag mode'

printf 'workflow policy passed: %s\n' "$workflow"
