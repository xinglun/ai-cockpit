#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  printf 'usage: %s <source-repository> <cleanup-record>\n' "$0" >&2
  exit 2
fi

source_repository=$(cd "$1" && pwd -P) || exit 2
record_input=$2
[[ -L "$record_input" ]] && {
  printf 'cleanup record must not be a symlink: %s\n' "$record_input" >&2
  exit 2
}
record_directory=$(cd "$(dirname "$record_input")" && pwd -P) || exit 2
cleanup_record="$record_directory/$(basename "$record_input")"
if [[ ! -f "$cleanup_record" || -L "$cleanup_record" ]]; then
  printf 'cleanup record must be a regular file: %s\n' "$cleanup_record" >&2
  exit 2
fi

if ! jq -e --arg repository "$source_repository" \
  'type == "object" and .schemaVersion == 1 and .sourceRepository == $repository
    and (.state | type == "string") and (.cleanupExitCode | type == "number")' \
  "$cleanup_record" >/dev/null; then
  printf 'cleanup record is malformed or belongs to another source repository\n' >&2
  exit 2
fi

state=$(jq -er '.state' "$cleanup_record")
if [[ "$state" != deferred_for_consumer ]]; then
  exit 0
fi
if [[ "$(jq -er '.cleanupExitCode' "$cleanup_record")" != 0 ]]; then
  printf 'deferred cleanup record has a non-zero producer cleanup status\n' >&2
  exit 2
fi

isolated_repository=$(jq -er '.isolatedRepository | strings | select(length > 0)' "$cleanup_record")
if [[ "$isolated_repository" != /* || -L "$isolated_repository" || ! -d "$isolated_repository" ]]; then
  printf 'deferred worktree path is missing, relative, or unsafe\n' >&2
  exit 2
fi

runner_temp_input=${RUNNER_TEMP:-${TMPDIR:-/tmp}}
runner_temp=$(cd "$runner_temp_input" && pwd -P) || exit 2
worktree_parent=${isolated_repository%/source}
if [[ "$worktree_parent" == "$isolated_repository" \
  || "$(dirname "$worktree_parent")" != "$runner_temp" \
  || "$(basename "$worktree_parent")" != ai-cockpit-hosted-runtime.* \
  || -L "$worktree_parent" ]]; then
  printf 'deferred worktree is outside its dedicated hosted Runtime temporary parent\n' >&2
  exit 2
fi
canonical_repository=$(cd "$isolated_repository" && pwd -P) || exit 2
if [[ "$canonical_repository" != "$isolated_repository" ]]; then
  printf 'deferred worktree path is not canonical\n' >&2
  exit 2
fi

common_directory() {
  local common
  common=$(git -C "$1" rev-parse --git-common-dir) || return 1
  if [[ "$common" != /* ]]; then common="$1/$common"; fi
  (cd "$common" && pwd -P)
}

source_common=$(common_directory "$source_repository") || exit 2
worktree_common=$(common_directory "$isolated_repository") || exit 2
source_head=$(git -C "$source_repository" rev-parse --verify 'HEAD^{commit}') || exit 2
worktree_head=$(git -C "$isolated_repository" rev-parse --verify 'HEAD^{commit}') || exit 2
if [[ "$source_common" != "$worktree_common" || "$source_head" != "$worktree_head" ]]; then
  printf 'deferred worktree is not the source checkout snapshot owned by this job\n' >&2
  exit 2
fi
if ! git -C "$source_repository" worktree list --porcelain | \
  awk -v path="$isolated_repository" '$1 == "worktree" && substr($0, 10) == path { found = 1 } END { exit !found }'; then
  printf 'deferred path is not registered as a linked worktree of the source repository\n' >&2
  exit 2
fi

cleanup_status=0
git -C "$source_repository" worktree remove --force "$isolated_repository" >/dev/null || cleanup_status=$?
if [[ "$cleanup_status" == 0 ]]; then
  if [[ -d "$isolated_repository" ]]; then
    rmdir "$isolated_repository" || cleanup_status=$?
  fi
fi
if [[ "$cleanup_status" == 0 ]]; then
  rmdir "$worktree_parent" || {
    cleanup_status=$?
    printf 'hosted Runtime temporary parent contains unexpected leftovers:\n' >&2
    find "$worktree_parent" -mindepth 1 -maxdepth 2 -print >&2 || true
  }
fi
new_state=removed
[[ "$cleanup_status" == 0 ]] || new_state=failed
temporary_record=$(mktemp "$record_directory/.hosted-runtime-cleanup.XXXXXX") || exit 2
if ! jq --arg state "$new_state" --argjson exitCode "$cleanup_status" \
  '.state = $state | .cleanupExitCode = $exitCode' \
  "$cleanup_record" >"$temporary_record"; then
  rm -f -- "$temporary_record"
  exit 2
fi
mv -f -- "$temporary_record" "$cleanup_record" || exit 2
if [[ "$cleanup_status" != 0 ]]; then
  printf 'failed to remove deferred hosted Runtime worktree (exit %s): %s\n' \
    "$cleanup_status" "$isolated_repository" >&2
  exit 1
fi
