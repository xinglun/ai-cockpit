#!/usr/bin/env bash
set -euo pipefail

workflow=${1:?usage: workflow_policy.sh <workflow>}

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
require_match 'release-manifest\.json' 'canonical manifest must be emitted'
require_match 'SHA256SUMS' 'canonical checksum set must be emitted'
require_match 'brew test' 'Homebrew fixture test must be defined'
require_match 'ai-cockpit --version' 'installed binary version smoke must be defined'
require_match '^  publish:' 'publish job must be present'
require_match 'needs:' 'publish must depend on verification jobs'
require_match '^  source_quality:' 'source-quality job must be present'
require_match '^  release_policy:' 'release-policy job must be present'
require_match '^  attest:' 'final attestation job must be present'
require_match '^  publish_handoff:' 'post-publication handoff job must be present'
require_match '^  post_release_version_consistency:' 'post-publication version consistency job must be present'
require_match '^  adopter_acceptance:' 'post-release adopter acceptance job must be present'
require_match '^  adopter_upgrade_acceptance:' 'post-release N-1 upgrade acceptance job must be present'
require_match 'cargo fmt --all -- --check' 'source quality must run rustfmt'
require_match 'cargo clippy --workspace --all-targets --all-features -- -D warnings' 'source quality must run Clippy'
require_match 'tests/ci/run_workspace_package_tests\.sh' 'source quality must derive workspace package tests from cargo metadata'
require_match 'tests/ci/run_repository_gates\.py' 'source quality must run the canonical repository gate manifest'
require_match 'cargo metadata --locked' 'source quality must gate workspace metadata'
require_match 'cargoLockSha256' 'release identity must bind Cargo.lock'
require_match "jq -er '.before'" 'tag policy must reject mutable tag updates'
require_match 'gh api .*releases/tags' 'release policy must reject an existing provider Release'
require_match '--provider-release-id' 'handoff must bind the provider Release identity'
require_match 'actions/attest-build-provenance@' 'final candidate/handoff attestation must be defined'
require_match 'dist/release-manifest\.json' 'published assets must include the canonical manifest'
require_match 'dist/Formula/ai-cockpit\.rb' 'published assets must include the Formula'
require_match 'needs: \[build, aggregate, source_quality, release_policy, verify, smoke_homebrew, smoke_linux, smoke_windows, attest\]' 'publish must depend on every final gate'
require_match '^  publish_handoff:' 'handoff must be a separate post-publication job'
require_match 'publish_handoff:[[:space:]]*$' 'post-publication handoff job must be addressable'
require_match 'adopter_acceptance:[[:space:]]*$' 'post-release adopter acceptance job must be addressable'
require_match 'tests/release/adopter_acceptance\.sh' 'post-release job must invoke the adopter acceptance harness'
require_match 'needs: \[publish, publish_handoff\]' 'adopter acceptance must run after publication and handoff'
require_match 'tests/release/version_consistency\.sh' 'release workflow must run the version consistency gate'
require_match 'tests/ci/repository_gate_manifest\.json' 'release workflow must bind all repository policy gates'
require_match '--post-release' 'post-publication version consistency must validate public assets'
require_match 'if: github\.event_name == '\''push'\'' && startsWith\(github\.ref, '\''refs/tags/'\''\)' 'adopter acceptance must be tag-only and post-publication'
require_match 'if: always\(\)' 'adopter acceptance evidence must upload after success or failure'
require_match 'tests/release/adopter_upgrade_acceptance\.sh' 'N-1 post-release job must invoke the public-artifact upgrade harness'
require_match 'INPUT_FROM_TAG' 'manual N-1 acceptance must receive an explicit from tag'
require_match 'INPUT_TO_TAG' 'manual N-1 acceptance must receive an explicit to tag'
require_match 'steps\.release_pair\.outputs\.from_tag' 'N-1 execution must use the resolved immutable Release pair'
require_match 'adopterAcceptance:"not_applicable"' 'first-release N-1 boundary must be explicit'
require_match 'name: Upload N-1 upgrade acceptance evidence' 'N-1 evidence must be uploaded independently'
require_match 'refs/tags/\$\{tag\}\^\{\}' 'publish must compare the peeled tag commit'

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
  /cargo run .*-- handoff/ { print job }
' "$workflow")"
if [[ "$handoff_jobs" != "publish_handoff" ]]; then
  printf 'policy failure: handoff generation must occur only after publication (jobs: %s)\n' "${handoff_jobs:-none}" >&2
  exit 1
fi

awk '
  /^jobs:/ { in_jobs=1 }
  in_jobs && /^  publish:/ { in_publish=1 }
  in_publish && /^  [A-Za-z0-9_-]+:/ && $0 !~ /^  publish:/ { in_publish=0 }
  in_publish && /if:.*github\.event_name == '\''push'\'' && startsWith\(github\.ref, '\''refs\/tags\/'\''\)/ { found=1 }
  END { exit(found ? 0 : 1) }
' "$workflow" || {
  printf 'policy failure: publish must be tag-gated\n' >&2
  exit 1
}

printf 'workflow policy passed: %s\n' "$workflow"
