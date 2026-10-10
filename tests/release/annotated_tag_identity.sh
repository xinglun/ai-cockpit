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
git -C "$source_repo" commit --allow-empty -qm 'workflow execution head'
workflow_head_sha="$(git -C "$source_repo" rev-parse HEAD)"
test "$workflow_head_sha" != "$commit_sha"
git -C "$source_repo" remote add origin "$remote_repo"
git -C "$source_repo" push -q origin v0.1.0

tag_object_sha="$(git -C "$source_repo" ls-remote origin refs/tags/v0.1.0 | awk '{print $1}')"
peeled_commit_sha="$(git -C "$source_repo" ls-remote origin 'refs/tags/v0.1.0^{}' | awk '{print $1}')"
test -n "$tag_object_sha"
test "$tag_object_sha" != "$commit_sha"
test "$peeled_commit_sha" = "$commit_sha"

# The typed source revision is the immutable product source.  Workflow
# orchestration can run at a later head; that head must not replace the
# manifest's source commit.  A plan carrying the execution head as its source
# is rejected because it does not match the annotated tag's peeled commit.
bind_normal_release_source() {
  local expected_source=$1
  local actual_tag_source
  actual_tag_source="$(git -C "$source_repo" ls-remote origin 'refs/tags/v0.1.0^{}' | awk '{print $1}')"
  test -n "$actual_tag_source"
  test "$actual_tag_source" = "$expected_source"
}
bind_normal_release_source "$commit_sha"
if bind_normal_release_source "$workflow_head_sha"; then
  printf 'release source binding accepted the workflow head instead of the annotated tag source\n' >&2
  exit 1
fi

# A lightweight tag has no peeled-tag advertisement in ls-remote.  The
# release workflow intentionally rejects that shape instead of guessing that
# the tag was reviewed and immutable.
git -C "$source_repo" tag v0.1.1 "$commit_sha"
git -C "$source_repo" push -q origin v0.1.1
lightweight_object_sha="$(git -C "$source_repo" ls-remote origin refs/tags/v0.1.1 | awk '{print $1}')"
lightweight_peeled_sha="$(git -C "$source_repo" ls-remote origin 'refs/tags/v0.1.1^{}' | awk '{print $1}')"
test "$lightweight_object_sha" = "$commit_sha"
test -z "$lightweight_peeled_sha"
printf 'annotated tag object differs from peeled commit; lightweight tags have no peeled identity and are rejected\n'
