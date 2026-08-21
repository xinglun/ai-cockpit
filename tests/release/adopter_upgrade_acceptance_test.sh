#!/usr/bin/env bash
set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/adopter_upgrade_acceptance.sh"
bash -n "$script"
grep -q -- '--from-tag' "$script"
grep -q -- '--to-tag' "$script"
grep -q -- 'releasePublished' "$script"
grep -q -- 'platform' "$script"
grep -q -- 'runtimeVersion' "$script"
grep -q -- 'runtimeDigest' "$script"
grep -q -- 'historical Runtime predates verify identity fields' "$script"
grep -q -- 'MIGRATION_REQUIRED' "$script"
grep -q -- 'migrate plan' "$script"
grep -q -- 'migrate apply' "$script"
grep -q -- '2:2' "$script"
grep -q -- 'not_required' "$script"
grep -q -- 'chainLength' "$script"
grep -q -- 'oldEvidenceDigest' "$script"
grep -q -- 'byte-identical' "$script"
grep -q -- 'SHA256SUMS' "$script"
if grep -Eq 'cargo (build|run)|target/debug/ai-cockpit|workspace binary' "$script"; then
  echo 'upgrade acceptance must not fall back to source builds or workspace binaries' >&2
  exit 1
fi
if "$script" --repository xinglun/ai-cockpit --from-tag v0.1.1 --to-tag v0.1.1 --target aarch64-apple-darwin --output "$(mktemp -d)" --source-repo "$(git rev-parse --show-toplevel)" >/dev/null 2>&1; then
  echo 'same Release tags must be rejected' >&2
  exit 1
fi
echo 'adopter upgrade acceptance static checks passed'
