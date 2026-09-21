#!/usr/bin/env bash
set -euo pipefail

# Release version consistency gate. Cargo metadata is the source of truth for
# the current Runtime version; historical release references are intentionally
# allowed, but current installation baselines may not drift silently.

usage() {
  cat <<'USAGE'
Usage: version_consistency.sh --repo DIRECTORY
       version_consistency.sh --repo DIRECTORY --post-release \
         --repository OWNER/REPOSITORY --tag vX.Y.Z
USAGE
}

die() {
  printf 'version consistency failure: %s\n' "$*" >&2
  exit 1
}

repo=''
post_release=false
repository=''
tag=''
while (($# > 0)); do
  case "$1" in
    --repo)
      [[ $# -ge 2 ]] || die '--repo requires a value'
      repo=$2
      shift 2
      ;;
    --post-release)
      post_release=true
      shift
      ;;
    --repository)
      [[ $# -ge 2 ]] || die '--repository requires a value'
      repository=$2
      shift 2
      ;;
    --tag)
      [[ $# -ge 2 ]] || die '--tag requires a value'
      tag=$2
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      usage >&2
      die "unknown argument: $1"
      ;;
  esac
done

[[ -n "$repo" ]] || die '--repo is required'
repo="$(cd "$repo" && pwd)"
cd "$repo"
command -v cargo >/dev/null 2>&1 || die 'cargo is unavailable'
command -v jq >/dev/null 2>&1 || die 'jq is unavailable'

if [[ -n "${COCKPIT_RELEASE_BIN:-}" ]]; then
  [[ -x "$COCKPIT_RELEASE_BIN" ]] || die "COCKPIT_RELEASE_BIN is not executable: $COCKPIT_RELEASE_BIN"
  release_identity="$($COCKPIT_RELEASE_BIN version-consistency --repo "$repo")" || die 'cockpit-release source version consistency failed'
else
  release_identity="$(cargo run --quiet --locked --package cockpit-release -- version-consistency --repo "$repo")" || die 'cockpit-release source version consistency failed'
fi
version="$(printf '%s' "$release_identity" | jq -er '.version')" || die 'cockpit-release did not return a typed source version'
release_tag="v${version}"

require_text() {
  local file=$1
  local text=$2
  [[ -f "$file" ]] || die "required current-version document is missing: $file"
  grep -Fq -- "$text" "$file" || die "$file does not contain current value: $text"
}

require_text .github/workflows/release.yml 'cargo metadata --locked'
require_text .github/workflows/release.yml 'tests/release/version_consistency.sh'

if [[ "$post_release" == true ]]; then
  [[ "$repository" =~ ^[^/]+/[^/]+$ ]] || die '--repository OWNER/REPOSITORY is required for post-release checks'
  [[ "$tag" == "$release_tag" ]] || die "post-release tag $tag does not match workspace $release_tag"
  command -v gh >/dev/null 2>&1 || die 'gh is unavailable for post-release checks'
  release_json="$(mktemp)"
  download_dir="$(mktemp -d)"
  trap 'rm -f "$release_json"; rmdir "$download_dir" 2>/dev/null || true' EXIT
  gh release view "$tag" --repo "$repository" --json tagName,isDraft,isPrerelease > "$release_json" || die 'public Release is unavailable'
  jq -e --arg tag "$tag" '.tagName == $tag and .isDraft == false and .isPrerelease == false' "$release_json" >/dev/null || die 'public Release is not stable'
  gh release download "$tag" --repo "$repository" --pattern release-manifest.json --dir "$download_dir" >/dev/null || die 'public release manifest is unavailable'
  manifest="$download_dir/release-manifest.json"
  jq -e --arg version "$version" --arg tag "$tag" '.version == $version and .tag == $tag and (.artifacts | length) == 5' "$manifest" >/dev/null || die 'public manifest version or target matrix drifted'
  jq -e --arg version "$version" 'all(.artifacts[]; (.archive.filename | startswith("ai-cockpit-v" + $version + "-")) and (.sbom.filename | startswith("ai-cockpit-v" + $version + "-")))' "$manifest" >/dev/null || die 'public asset names are not bound to workspace version'
  printf 'post-release public asset check passed: %s (%s)\n' "$tag" "$repository"
else
  printf 'source version consistency passed: %s\n' "$version"
fi
