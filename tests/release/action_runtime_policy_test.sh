#!/usr/bin/env bash
set -euo pipefail

script=tests/release/action_runtime_policy.sh
ci=.github/workflows/ci.yml
release=.github/workflows/release.yml

test -x "$script"
test -f "$ci"
test -f "$release"
grep -Fq 'actions/checkout' "$script"
grep -Fq '3d3c42e5aac5ba805825da76410c181273ba90b1' "$script"
grep -Fq 'actions/upload-artifact' "$script"
grep -Fq '043fb46d1a93c77aae656e7c1c64a875d1fc6a0a' "$script"
grep -Fq 'actions/download-artifact' "$script"
grep -Fq '3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c' "$script"
grep -Fq 'actions/attest-build-provenance' "$script"
grep -Fq '4d101475d8b20a2381f78447822ac1eab6504dd8' "$script"
grep -Fq 'anchore/sbom-action' "$script"
grep -Fq 'e22c389904149dbc22b58101806040fa8d37a610' "$script"
grep -Fq 'softprops/action-gh-release' "$script"
grep -Fq 'fe965f7af51af5f2602596916f38a38df2e33de0' "$script"
grep -Fq 'runtime_baseline' "$script"
grep -Fq 'run_repository_gates.py' "$ci"
grep -Fq 'run_repository_gates.py' "$release"

"$script" "$ci" "$release"

fixture=$(mktemp -d)
trap 'find "$fixture" -type f -delete; find "$fixture" -type d -depth -empty -delete; rmdir "$fixture" 2>/dev/null || true' EXIT
cp "$ci" "$fixture/ci.yml"
sed -i.bak 's#actions/checkout@[0-9a-f]*#actions/checkout@v7#g' "$fixture/ci.yml"
set +e
output=$("$script" "$fixture/ci.yml" "$release" 2>&1)
result=$?
set -e
[[ "$result" -ne 0 ]] || { printf '%s\n' "$output" >&2; exit 1; }
grep -Fq 'unpinned action' <<<"$output"

sed -i.bak 's#actions/checkout@v7#actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683#g' "$fixture/ci.yml"
set +e
output=$("$script" "$fixture/ci.yml" "$release" 2>&1)
result=$?
set -e
[[ "$result" -ne 0 ]] || { printf '%s\n' "$output" >&2; exit 1; }
grep -Fq 'stale actions/checkout' <<<"$output"

printf 'action runtime policy static checks passed\n'
