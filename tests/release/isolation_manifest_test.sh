#!/usr/bin/env bash
set -euo pipefail

helper="$(cd "$(dirname "$0")" && pwd)/isolation_manifest.sh"
# shellcheck source=/dev/null
source "$helper"

parent="${TMPDIR:-/tmp}"
root="$(mktemp -d "$parent/ai-cockpit-isolation-manifest-regression.XXXXXX")"
root="$(cd "$root" && pwd -P)"
manifest_before="$parent/ai-cockpit-isolation-before.$$.manifest"
manifest_after="$parent/ai-cockpit-isolation-after.$$.manifest"
manifest_output_only="$parent/ai-cockpit-isolation-output-only.$$.manifest"
manifest_untracked="$parent/ai-cockpit-isolation-untracked.$$.manifest"
cleanup() {
  rm -rf -- "$root" "$manifest_before" "$manifest_after" "$manifest_output_only" "$manifest_untracked"
}
trap cleanup EXIT

mkdir -p "$root/empty-directory" "$root/nested"
printf 'alpha\n' > "$root/nested/file.txt"
ln -s nested/file.txt "$root/file-link"
manifest_tree "$root" "$manifest_before"

identity_before="$(path_identity "$root")"
printf 'identity-stability\n' > "$root/nested/identity.txt"
[[ "$(path_identity "$root")" == "$identity_before" ]] || {
  printf 'path identity changed after contents were written below the root\n' >&2
  exit 1
}

jq -e 'select(.path == "empty-directory" and .type == "directory")' "$manifest_before" >/dev/null
jq -e 'select(.path == "nested/file.txt" and .type == "file")' "$manifest_before" >/dev/null
jq -e --arg target 'nested/file.txt' --arg resolved "$root/nested/file.txt" \
  'select(.path == "file-link" and .type == "symlink" and .target == $target and .resolvedTarget == $resolved)' \
  "$manifest_before" >/dev/null
jq -e --arg digest "sha256:$(sha256_file "$root/nested/file.txt")" \
  'select(.path == "nested/file.txt" and .digest == $digest)' "$manifest_before" >/dev/null
validate_manifest_symlink_containment "$root" "$manifest_before"

printf 'beta\n' > "$root/nested/file.txt"
chmod 700 "$root/empty-directory"
rm "$root/file-link"
ln -s nested "$root/file-link"
mkdir "$root/new-directory"
manifest_tree "$root" "$manifest_after"

if cmp -s "$manifest_before" "$manifest_after"; then
  printf 'manifest did not detect content, metadata, directory, and symlink mutations\n' >&2
  exit 1
fi
jq -e --arg digest "sha256:$(sha256_file "$root/nested/file.txt")" \
  'select(.path == "nested/file.txt" and .digest == $digest)' "$manifest_after" >/dev/null
jq -e --arg target nested --arg resolved "$root/nested" \
  'select(.path == "file-link" and .type == "symlink" and .target == $target and .resolvedTarget == $resolved)' \
  "$manifest_after" >/dev/null
jq -e 'select(.path == "new-directory" and .type == "directory")' "$manifest_after" >/dev/null

ln -s missing-target "$root/dangling-link"
manifest_tree "$root" "$manifest_after"
if validate_manifest_symlink_containment "$root" "$manifest_after"; then
  printf 'manifest accepted an unresolved symlink target\n' >&2
  exit 1
fi
rm "$root/dangling-link"

outside="$parent/ai-cockpit-isolation-outside.$$"
mkdir -p "$outside"
ln -s "$outside" "$root/outside-link"
manifest_tree "$root" "$manifest_after"
if validate_manifest_symlink_containment "$root" "$manifest_after"; then
  printf 'manifest accepted a symlink target outside its allowed root\n' >&2
  exit 1
fi

source_repo="$root/source"
output="$source_repo/.ai/declared-output"
mkdir -p "$source_repo/.ai/untracked"
git -C "$source_repo" init -q
git -C "$source_repo" config user.name 'Isolation Test'
git -C "$source_repo" config user.email 'isolation@example.invalid'
printf 'tracked\n' > "$source_repo/tracked.txt"
git -C "$source_repo" add tracked.txt
git -C "$source_repo" commit -qm initial
printf 'before\n' > "$source_repo/.ai/untracked/state.json"
printf 'loose-before\n' > "$source_repo/loose.txt"
manifest_source_checkout "$source_repo" "$output" "$manifest_before"
mkdir -p "$output"
printf 'ignored-before\n' > "$output/receipt.json"
manifest_source_checkout "$source_repo" "$output" "$manifest_output_only"
if ! cmp -s "$manifest_before" "$manifest_output_only"; then
  printf 'declared output creation changed the source manifest through ancestor metadata\n' >&2
  exit 1
fi
printf 'loose-after\n' > "$source_repo/loose.txt"
manifest_source_checkout "$source_repo" "$output" "$manifest_untracked"
if cmp -s "$manifest_output_only" "$manifest_untracked"; then
  printf 'source manifest did not detect a mutation to an untracked non-.ai file\n' >&2
  exit 1
fi
jq -e 'select(.path == "loose.txt" and .type == "file")' "$manifest_untracked" >/dev/null
printf 'loose-before\n' > "$source_repo/loose.txt"
printf 'after\n' > "$source_repo/.ai/untracked/state.json"
printf 'ignored-after\n' > "$output/receipt.json"
manifest_source_checkout "$source_repo" "$output" "$manifest_after"
if cmp -s "$manifest_before" "$manifest_after"; then
  printf 'source manifest did not detect a mutation below an existing untracked .ai directory\n' >&2
  exit 1
fi
jq -e 'select(.path == ".ai/untracked/state.json" and .type == "file")' "$manifest_after" >/dev/null
if jq -e 'select(.path | startswith(".ai/declared-output"))' "$manifest_after" >/dev/null; then
  printf 'source manifest included the declared output directory\n' >&2
  exit 1
fi

rm -rf -- "$root" "$outside"
trap - EXIT
[[ ! -e "$root" ]]
printf 'isolation manifest regression checks passed\n'
