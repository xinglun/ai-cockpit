#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"

for document in \
  "$repo_root/docs/reference/implementation-knowledge.md" \
  "$repo_root/docs/reference/implementation-knowledge.zh-CN.md" \
  "$repo_root/docs/reference/implementation-knowledge.ja.md"; do
  test -f "$document"
  if grep -Fq 'never writes `.ai/`' "$document" \
    || grep -Fq '查询不会写入 `.ai/`' "$document" \
    || grep -Fq '`.ai/` に書き込まず' "$document"; then
    printf 'knowledge projection boundary regression: %s still claims query is write-free\n' "$document" >&2
    exit 1
  fi
  grep -Fq 'repository-local-derived' "$document"
done

for document in \
  "$repo_root/docs/reference/commands.md" \
  "$repo_root/docs/reference/commands.zh-CN.md" \
  "$repo_root/docs/reference/commands.ja.md"; do
  grep -Fq 'knowledge query' "$document"
  grep -Fq 'repository-local' "$document"
  grep -Fq 'projection' "$document"
done

printf 'knowledge projection boundary passed\n'
