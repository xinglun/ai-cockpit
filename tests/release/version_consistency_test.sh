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
grep -Fq 'tests/release/version_consistency.sh' "$workflow"
grep -Fq 'post_release_version_consistency:' "$workflow"
grep -Fq 'needs: [publish, publish_handoff]' "$workflow"
for document in \
  docs/release/distribution.md \
  docs/release/distribution.ja.md \
  docs/release/distribution.zh-CN.md; do
  grep -Fq 'git tag -a' "$document"
  grep -Fq 'gh release create' "$document"
done

printf 'version consistency static checks passed\n'
