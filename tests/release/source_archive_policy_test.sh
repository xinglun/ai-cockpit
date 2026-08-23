#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"
attributes="$repo_root/.gitattributes"
ignore="$repo_root/.gitignore"

[[ -f "$attributes" && ! -L "$attributes" ]] || {
  printf 'source archive policy failure: .gitattributes must be a regular file\n' >&2
  exit 1
}
grep -Fxq '/.ai export-ignore' "$attributes"
grep -Fxq '/.worktrees export-ignore' "$attributes"
grep -Fxq '/dist export-ignore' "$attributes"
grep -Fxq '/target export-ignore' "$attributes"
grep -Fxq '/dist/' "$ignore"
grep -Fxq '/.worktrees/' "$ignore"
grep -Fxq '__pycache__/' "$ignore"
grep -Fxq '*.pyc' "$ignore"

mkdir -p "$repo_root/target"
archive="$(mktemp "$repo_root/target/source-archive-policy.XXXXXX.tar")"
members="$(mktemp "$repo_root/target/source-archive-members.XXXXXX.txt")"
cleanup() {
  find "$archive" "$members" -type f -delete 2>/dev/null || true
}
trap cleanup EXIT

git -C "$repo_root" archive --worktree-attributes --format=tar HEAD > "$archive"
tar -tf "$archive" > "$members"

grep -Fxq 'Cargo.toml' "$members"
grep -Fxq 'Cargo.lock' "$members"
grep -Fxq 'LICENSE' "$members"
grep -Fxq 'crates/cockpit-cli/Cargo.toml' "$members"
if grep -Eq '^(\.ai|\.worktrees|dist|target)(/|$)' "$members"; then
  printf 'source archive policy failure: governance or generated content leaked into archive\n' >&2
  exit 1
fi

printf 'source archive policy passed\n'
