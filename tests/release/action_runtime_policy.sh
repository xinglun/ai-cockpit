#!/usr/bin/env bash
set -euo pipefail

# Keep the GitHub Actions runtime baseline explicit.  Every action is pinned to
# a full SHA; the actions that previously emitted the Node 20 deprecation
# warning are pinned to their current Node 24-compatible stable release.

usage() {
  printf 'Usage: action_runtime_policy.sh WORKFLOW [WORKFLOW ...]\n'
}

die() {
  printf 'action runtime policy failure: %s\n' "$*" >&2
  exit 1
}

(($# > 0)) || { usage >&2; die 'at least one workflow is required'; }

actions=(
  actions/checkout
  actions/upload-artifact
  actions/download-artifact
  actions/attest-build-provenance
  anchore/sbom-action
  softprops/action-gh-release
  dtolnay/rust-toolchain
)
expected_sha=(
  3d3c42e5aac5ba805825da76410c181273ba90b1
  043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
  3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c
  4d101475d8b20a2381f78447822ac1eab6504dd8
  e22c389904149dbc22b58101806040fa8d37a610
  fe965f7af51af5f2602596916f38a38df2e33de0
  6c977a6ca4077a0ceb28ffbe03f59d46e9ac8772
)
runtime_baseline=(node24 node24 node24 node24 node24 node24 composite)
seen=(0 0 0 0 0 0 0)

for workflow in "$@"; do
  [[ -f "$workflow" ]] || die "workflow is missing: $workflow"
  while IFS= read -r reference; do
    [[ -n "$reference" ]] || continue
    action=${reference%@*}
    sha=${reference#*@}
    [[ "$sha" =~ ^[0-9a-f]{40}$ ]] || die "$workflow has an unpinned action: $reference"
    action_index=-1
    for index in "${!actions[@]}"; do
      if [[ "${actions[$index]}" == "$action" ]]; then
        action_index=$index
        break
      fi
    done
    ((action_index >= 0)) || die "$workflow has an action without an approved runtime baseline: $action"
    [[ "$sha" == "${expected_sha[$action_index]}" ]] || die "$workflow has stale $action ref: $sha (expected ${expected_sha[$action_index]})"
    [[ -n "${runtime_baseline[$action_index]}" ]] || die "runtime baseline is missing for $action"
    seen[$action_index]=1
  done < <(grep -oE 'uses:[[:space:]]*[^@[:space:]]+@[^[:space:]]+' "$workflow" | sed -E 's/^uses:[[:space:]]*//')
done

for index in "${!actions[@]}"; do
  [[ "${seen[$index]}" == 1 ]] || die "required action is absent from supplied workflows: ${actions[$index]}"
done

printf 'action runtime policy passed: %s\n' "$*"
