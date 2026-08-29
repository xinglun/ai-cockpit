#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cli_source="$repo_root/crates/cockpit-cli/src/main.rs"
repository_source="$repo_root/crates/cockpit-repository/src/lib.rs"

grep -Fq 'implementation_approach_read_only(&repo, &id)' "$cli_source"
inspect_block="$(awk '/WorkItemCommand::Inspect/{capture=1} capture{print} /WorkItemCommand::Declare/{if(capture){exit}}' "$cli_source")"
if grep -Fq 'implementation_approach(&repo, &id)' <<<"$inspect_block"; then
  printf 'inspect projection boundary regression: inspect still calls the persisting approach API\n' >&2
  exit 1
fi
grep -Fq 'fn implementation_approach_internal(' "$repository_source"
grep -Fq 'if persist {' "$repository_source"

grep -Fq '`work-item inspect`' "$repo_root/docs/capabilities.md"
grep -Fq '`work-item inspect`' "$repo_root/docs/capabilities.zh-CN.md"
grep -Fq '`work-item inspect`' "$repo_root/docs/capabilities.ja.md"
grep -Fq 'work-item inspect --repo' "$repo_root/docs/reference/commands.md"
grep -Fq 'work-item inspect --repo' "$repo_root/docs/reference/commands.zh-CN.md"
grep -Fq 'work-item inspect --repo' "$repo_root/docs/reference/commands.ja.md"

printf 'inspect projection boundary passed\n'
