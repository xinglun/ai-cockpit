#!/usr/bin/env bash
set -euo pipefail

repo="${1:-$(pwd)}"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-parity-check.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/docs/reference"
mkdir -p "$tmp/.ai/evidence" "$tmp/.ai/decisions"
cp "$repo/docs/reference/reference-parity.md" "$tmp/docs/reference/reference-parity.md"
cp "$repo/docs/reference/reference-parity.zh-CN.md" "$tmp/docs/reference/reference-parity.zh-CN.md"
cp "$repo/docs/reference/reference-parity.ja.md" "$tmp/docs/reference/reference-parity.ja.md"
cp "$repo/tests/docs/parity_status_check.sh" "$tmp/parity_status_check.sh"
cp "$repo/.ai/evidence/WI-178-post-release-adopter-finalization-reconciliation.verification.json" "$tmp/.ai/evidence/"
cp "$repo/.ai/evidence/WI-179-post-release-parity-v0-2-22.verification.json" "$tmp/.ai/evidence/"
cp "$repo/.ai/evidence/WI-180-parity-status-closure-correction.verification.json" "$tmp/.ai/evidence/"
cp "$repo/.ai/decisions/WI-178-post-release-adopter-finalization-reconciliation.finalize.json" "$tmp/.ai/decisions/"
cp "$repo/.ai/decisions/WI-178-post-release-adopter-finalization-reconciliation.close.json" "$tmp/.ai/decisions/"
cp "$repo/.ai/decisions/WI-179-post-release-parity-v0-2-22.finalize.json" "$tmp/.ai/decisions/"
cp "$repo/.ai/decisions/WI-179-post-release-parity-v0-2-22.close.json" "$tmp/.ai/decisions/"
cp "$repo/.ai/decisions/WI-180-parity-status-closure-correction.finalize.json" "$tmp/.ai/decisions/"
cp "$repo/.ai/decisions/WI-180-parity-status-closure-correction.close.json" "$tmp/.ai/decisions/"

if sed -i.bak 's#; `.ai/evidence/WI-180-parity-status-closure-correction.verification.json`##' "$tmp/docs/reference/reference-parity.md"; then
  rm -f "$tmp/docs/reference/reference-parity.md.bak"
else
  sed -i '' 's#; `.ai/evidence/WI-180-parity-status-closure-correction.verification.json`##' "$tmp/docs/reference/reference-parity.md"
fi

if bash "$tmp/parity_status_check.sh" "$tmp" >/dev/null 2>&1; then
  printf 'parity status regression: missing evidence binding was accepted\n' >&2
  exit 1
fi

printf 'parity status regression passed\n'
