#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
policy="$root/tests/conformance/reference_source_policy.py"
lock="$root/tests/conformance/reference-source.lock"
python3 "$policy" --lock "$lock"

tmp=$(mktemp -d "${TMPDIR:-/tmp}/reference-source-policy-check.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
fixture="$tmp/reference"
git init -q "$fixture"
git -C "$fixture" config user.name fixture
git -C "$fixture" config user.email fixture@example.invalid
printf 'reference fixture\n' > "$fixture/README.md"
git -C "$fixture" add README.md
git -C "$fixture" commit -qm fixture
commit=$(git -C "$fixture" rev-parse HEAD)
fixture_lock="$tmp/reference-source.lock"
printf '%s\n' \
  'schema = 1' \
  'source = "local-git-checkout"' \
  'path_env = "AI_COCKPIT_REFERENCE_ROOT"' \
  "commit = \"$commit\"" \
  'network_access = false' > "$fixture_lock"

python3 "$policy" --lock "$fixture_lock" --reference "$fixture"
if python3 "$policy" --lock "$fixture_lock" --reference "$tmp/missing"; then
  echo 'reference policy accepted a missing checkout' >&2
  exit 1
fi
printf 'dirty\n' >> "$fixture/README.md"
if python3 "$policy" --lock "$fixture_lock" --reference "$fixture"; then
  echo 'reference policy accepted a dirty checkout' >&2
  exit 1
fi

active_files=(
  "$root/.github/workflows/ci.yml"
  "$root/tests/conformance/final_replacement_acceptance.sh"
  "$root/tests/conformance/reference_file_inventory.py"
  "$root/tests/conformance/reference_file_inventory.json"
  "$root/tests/conformance/v1-reference.lock"
  "$root/tests/conformance/reference-source.lock"
)
needle='https://github.com/spirex-ds-dev/'"ai-cockpit-template"
if rg -n "$needle" "${active_files[@]}"; then
  echo 'active reference policy still contains a public source URL' >&2
  exit 1
fi

echo 'local reference-source policy regression passed'
