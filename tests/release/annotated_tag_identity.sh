#!/usr/bin/env bash
set -euo pipefail

source_repo="$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-tag-source.XXXXXX")"
remote_repo="$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-tag-remote.XXXXXX")"
trap 'rm -rf "$source_repo" "$remote_repo"' EXIT

git -C "$remote_repo" init --bare -q
git -C "$source_repo" init -q
git -C "$source_repo" config user.name 'AI Cockpit Release Test'
git -C "$source_repo" config user.email 'release-test@example.invalid'
git -C "$source_repo" commit --allow-empty -qm 'candidate'
commit_sha="$(git -C "$source_repo" rev-parse HEAD)"
git -C "$source_repo" tag -a v0.1.0 -m 'v0.1.0'
git -C "$source_repo" remote add origin "$remote_repo"
git -C "$source_repo" push -q origin v0.1.0

tag_object_sha="$(git -C "$source_repo" ls-remote origin refs/tags/v0.1.0 | awk '{print $1}')"
peeled_commit_sha="$(git -C "$source_repo" ls-remote origin 'refs/tags/v0.1.0^{}' | awk '{print $1}')"
test -n "$tag_object_sha"
test "$tag_object_sha" != "$commit_sha"
test "$peeled_commit_sha" = "$commit_sha"
printf 'annotated tag object differs from peeled commit; peeled commit is authoritative\n'
