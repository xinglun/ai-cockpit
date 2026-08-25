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

metadata="$(cargo metadata --locked --format-version 1)"
version="$(printf '%s' "$metadata" | jq -er '[.packages[] | select(.name == "cockpit-cli" and .source == null) | .version] | if length == 1 then .[0] else error("cockpit-cli workspace package is ambiguous") end')"
release_tag="v${version}"
printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || die "invalid workspace version: $version"

if ! printf '%s' "$metadata" | jq -e --arg version "$version" '[.packages[] | select(.source == null) | .version == $version] | all' >/dev/null; then
  die 'workspace package versions are not aligned with cockpit-cli'
fi

require_text() {
  local file=$1
  local text=$2
  [[ -f "$file" ]] || die "required current-version document is missing: $file"
  grep -Fq -- "$text" "$file" || die "$file does not contain current value: $text"
}

for file in \
  docs/release/distribution.md \
  docs/release/distribution.ja.md \
  docs/release/distribution.zh-CN.md; do
  require_text "$file" "$release_tag"
  require_text "$file" "ai-cockpit-${release_tag}-"
done

for file in \
  docs/architecture/release-distribution.md \
  docs/architecture/release-distribution.ja.md \
  docs/architecture/release-distribution.zh-CN.md; do
  require_text "$file" "$release_tag"
  if ! grep -Eiq 'baseline|基线|ベースライン' "$file"; then
    die "$file does not declare a release baseline"
  fi
done

for file in \
  docs/architecture/versioning.md \
  docs/architecture/versioning.ja.md \
  docs/architecture/versioning.zh-CN.md; do
  require_text "$file" "$version"
done

# Operations pages describe the current baseline target without pinning a
# release number. The version is resolved from Cargo metadata above so a
# Runtime release cannot silently leave an old version in the operator route.
for file in \
  docs/operations/README.md \
  docs/operations/README.ja.md \
  docs/operations/README.zh-CN.md; do
  require_text "$file" 'x86_64-unknown-linux-gnu'
  if grep -Eq 'v[0-9]+\.[0-9]+\.[0-9]+' "$file"; then
    die "$file hard-codes a release version in the current operations baseline"
  fi
done

# A current-baseline line must never name a different semantic version. This
# deliberately ignores historical N-1/migration prose elsewhere in the docs.
while IFS= read -r line; do
  [[ "$line" == *"$release_tag"* ]] || die "current baseline is stale: $line"
done < <(grep -HinE 'current installation baseline|current immutable public baseline|現在の installation baseline|現在の immutable public baseline|当前安装基线|当前不可变公开基线' docs/release/distribution.* docs/architecture/release-distribution.* || true)

require_text .github/workflows/release.yml 'cargo metadata --locked'
require_text .github/workflows/release.yml 'tests/release/version_consistency.sh'

if [[ "$post_release" == true ]]; then
  [[ "$repository" =~ ^[^/]+/[^/]+$ ]] || die '--repository OWNER/REPOSITORY is required for post-release checks'
  [[ "$tag" == "$release_tag" ]] || die "post-release tag $tag does not match workspace $release_tag"
  command -v gh >/dev/null 2>&1 || die 'gh is unavailable for post-release checks'
  release_json="$(mktemp)"
  download_dir="$(mktemp -d)"
  cleanup_release_download() {
    local prior_exit=$1
    local cleanup_failed=false
    local cleanup_reason=''

    if ! rm -f -- "$release_json"; then
      cleanup_failed=true
      cleanup_reason='release metadata temporary file could not be removed'
    fi
    if ! rm -rf -- "$download_dir" || [[ -e "$download_dir" ]]; then
      cleanup_failed=true
      if [[ -n "$cleanup_reason" ]]; then
        cleanup_reason+="; "
      fi
      cleanup_reason+='download directory could not be removed'
    fi

    if [[ "$cleanup_failed" == true ]]; then
      # Cleanup is an independent postcondition. Report it without changing
      # the already observed public Release truth or rewriting any receipt.
      printf 'post-release cleanup: failed (release truth unchanged): %s\n' \
        "$cleanup_reason" >&2
      if (( prior_exit == 0 )); then
        prior_exit=1
      fi
    else
      printf 'post-release cleanup: passed\n' >&2
    fi
    exit "$prior_exit"
  }
  trap 'cleanup_release_download "$?"' EXIT
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
