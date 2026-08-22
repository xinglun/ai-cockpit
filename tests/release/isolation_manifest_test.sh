#!/usr/bin/env bash
set -euo pipefail

helper="$(cd "$(dirname "$0")" && pwd)/isolation_manifest.sh"
# shellcheck source=/dev/null
source "$helper"

parent="${TMPDIR:-/tmp}"
root="$(mktemp -d "$parent/ai-cockpit-isolation-manifest-regression.XXXXXX")"
manifest_before="$parent/ai-cockpit-isolation-before.$$.manifest"
manifest_after="$parent/ai-cockpit-isolation-after.$$.manifest"
cleanup() {
  rm -rf -- "$root" "$manifest_before" "$manifest_after"
}
trap cleanup EXIT

mkdir -p "$root/empty-directory" "$root/nested"
printf 'alpha\n' > "$root/nested/file.txt"
ln -s nested/file.txt "$root/file-link"
manifest_tree "$root" "$manifest_before"

grep -q $'empty-directory\tdirectory\t' "$manifest_before"
grep -q $'nested/file.txt\tfile\t' "$manifest_before"
grep -q $'file-link\tsymlink\t' "$manifest_before"
grep -q "sha256:$(sha256_file "$root/nested/file.txt")" "$manifest_before"
grep -q "sha256:$(sha256_text 'nested/file.txt')" "$manifest_before"

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
grep -q "sha256:$(sha256_file "$root/nested/file.txt")" "$manifest_after"
grep -q "sha256:$(sha256_text 'nested')" "$manifest_after"
grep -q $'new-directory\tdirectory\t' "$manifest_after"

rm -rf -- "$root"
trap - EXIT
[[ ! -e "$root" ]]
printf 'isolation manifest regression checks passed\n'
