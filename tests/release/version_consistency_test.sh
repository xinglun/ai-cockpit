#!/usr/bin/env bash
set -euo pipefail

script=tests/release/version_consistency.sh
workflow=.github/workflows/release.yml

test -x "$script"
grep -Fq 'cargo metadata --locked --format-version 1' "$script"
grep -Fq 'docs/release/distribution.ja.md' "$script"
grep -Fq 'docs/release/distribution.zh-CN.md' "$script"
grep -Fq 'docs/architecture/release-distribution.ja.md' "$script"
grep -Fq 'docs/architecture/release-distribution.zh-CN.md' "$script"
grep -Fq -- '--post-release' "$script"
grep -Fq 'cleanup_release_download' "$script"
grep -Fq 'post-release cleanup: failed (release truth unchanged)' "$script"
grep -Fq 'tests/release/version_consistency.sh' "$workflow"
grep -Fq 'post_release_version_consistency:' "$workflow"
grep -Fq 'needs: [publish, publish_handoff]' "$workflow"

version="$(cargo metadata --locked --format-version 1 | jq -er '[.packages[] | select(.name == "cockpit-cli" and .source == null) | .version] | if length == 1 then .[0] else error end')"
tag="v${version}"
real_mktemp="$(command -v mktemp)"
real_rm="$(command -v rm)"

run_post_release_case() {
  local name=$1
  local manifest_kind=$2
  local fail_cleanup=$3
  local root
  root="$("$real_mktemp" -d "${TMPDIR:-/tmp}/version-consistency-test.XXXXXX")"

  set +e
  (
    export WI_TEST_ROOT="$root"
    export WI_TEST_VERSION="$version"
    export WI_TEST_TAG="$tag"
    export WI_MANIFEST_KIND="$manifest_kind"
    export WI_FAIL_CLEANUP="$fail_cleanup"
    export WI_REAL_MKTEMP="$real_mktemp"
    export WI_REAL_RM="$real_rm"

    fake_mktemp() {
      if [[ "${1:-}" == '-d' ]]; then
        "$WI_REAL_MKTEMP" -d "$WI_TEST_ROOT/download.XXXXXX"
      else
        "$WI_REAL_MKTEMP" "$WI_TEST_ROOT/file.XXXXXX"
      fi
    }
    mktemp() { fake_mktemp "$@"; }

    gh() {
      if [[ "${1:-}" == release && "${2:-}" == view ]]; then
        printf '{"tagName":"%s","isDraft":false,"isPrerelease":false}\n' "$WI_TEST_TAG"
        return 0
      fi
      if [[ "${1:-}" == release && "${2:-}" == download ]]; then
        local dir=''
        while [[ $# -gt 0 ]]; do
          if [[ "$1" == '--dir' ]]; then
            dir=$2
            shift 2
          else
            shift
          fi
        done
        if [[ "$WI_MANIFEST_KIND" == valid ]]; then
          jq -n \
            --arg version "$WI_TEST_VERSION" \
            --arg tag "$WI_TEST_TAG" \
            '{version:$version,tag:$tag,artifacts:[
              "aarch64-apple-darwin",
              "aarch64-unknown-linux-gnu",
              "x86_64-apple-darwin",
              "x86_64-pc-windows-msvc",
              "x86_64-unknown-linux-gnu"
            ] | map({
              archive:{filename:("ai-cockpit-v"+$version+"-"+.)},
              sbom:{filename:("ai-cockpit-v"+$version+"-"+.)}
            })}' > "$dir/release-manifest.json"
        else
          printf '%s\n' '{"version":"0.0.0","tag":"v0.0.0","artifacts":[]}' > "$dir/release-manifest.json"
        fi
        return 0
      fi
      return 1
    }

    fake_rm() {
      if [[ "$WI_FAIL_CLEANUP" == true && "${1:-}" == '-rf' ]]; then
        printf 'injected cleanup failure for %s\n' "${2:-unknown}" >&2
        return 1
      fi
      "$WI_REAL_RM" "$@"
    }
    rm() { fake_rm "$@"; }

    export -f fake_mktemp mktemp gh fake_rm rm
    if output="$("$script" --repo "$PWD" --post-release --repository xinglun/ai-cockpit --tag "$WI_TEST_TAG" 2>&1)"; then
      rc=0
    else
      rc=$?
    fi

    case "$name" in
      success)
        [[ "$rc" -eq 0 ]] || { printf '%s\n' "$output" >&2; exit 1; }
        grep -Fq 'post-release public asset check passed' <<<"$output"
        grep -Fq 'post-release cleanup: passed' <<<"$output"
        [[ -z "$(find "$WI_TEST_ROOT" -mindepth 1 -maxdepth 1 -print -quit)" ]]
        ;;
      manifest_failure)
        [[ "$rc" -ne 0 ]] || { printf '%s\n' "$output" >&2; exit 1; }
        grep -Fq 'public manifest version or target matrix drifted' <<<"$output"
        grep -Fq 'post-release cleanup: passed' <<<"$output"
        [[ -z "$(find "$WI_TEST_ROOT" -mindepth 1 -maxdepth 1 -print -quit)" ]]
        ;;
      cleanup_failure)
        [[ "$rc" -ne 0 ]] || { printf '%s\n' "$output" >&2; exit 1; }
        grep -Fq 'post-release cleanup: failed (release truth unchanged)' <<<"$output"
        [[ -n "$(find "$WI_TEST_ROOT" -mindepth 2 -name release-manifest.json -print -quit)" ]]
        ;;
    esac
  )
  rc=$?
  # The test root is an exact, isolated temporary path. Bypass the injected
  # rm function when removing it so a cleanup-failure case cannot leak.
  "$real_rm" -rf -- "$root"
  return "$rc"
}

run_post_release_case success valid false
run_post_release_case manifest_failure invalid false
run_post_release_case cleanup_failure valid true

printf 'version consistency static and cleanup regression checks passed\n'
