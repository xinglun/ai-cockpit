#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/parity-status.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
fixture="$root/tests/ci/fixtures/governance-integrity"
python3 "$fixture/build_fixture.py" --spec "$fixture/valid.json" --output "$tmp/repo"
mkdir -p "$tmp/repo/tests/ci" "$tmp/repo/tests/docs"
cp "$root/tests/ci/governance_integrity_gate.py" "$tmp/repo/tests/ci/"
cp "$root/tests/docs/parity_status_check.sh" "$tmp/repo/tests/docs/"

bash "$tmp/repo/tests/docs/parity_status_check.sh" "$tmp/repo" "$tmp/pass.json" >/dev/null
rm "$tmp/repo/.ai/evidence/WI-900-release-v9-9-9.verification.json"
if bash "$tmp/repo/tests/docs/parity_status_check.sh" "$tmp/repo" "$tmp/fail.json" >/dev/null 2>&1; then
  printf 'parity status regression accepted missing evidence\n' >&2
  exit 1
fi
jq -e '.findings[] | select(.code == "missing_evidence")' "$tmp/fail.json" >/dev/null
printf 'parity status regression passed\n'
