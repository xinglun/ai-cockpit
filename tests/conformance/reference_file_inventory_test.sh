#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
manifest="$root/tests/conformance/reference_file_inventory.json"
script="$root/tests/conformance/reference_file_inventory.py"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/reference-file-inventory-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

python3 "$script" --manifest "$manifest" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit 46e426625a8cae450f1190d0bdbafd6d8e648a90 --check

test "$(jq -r '.referenceTrackedFileCount' "$manifest")" -eq "$(jq '.records | length' "$manifest")"
test "$(jq -r '.records[] | select(.referencePath == "CONTRIBUTING.md") | .classification' "$manifest")" = "implemented-different-by-design"
test "$(jq -r '.records[] | select(.batch == "governance-entrypoints" and .classification == "deferred-next-batch") | .referencePath' "$manifest" | wc -l | tr -d ' ')" -eq 0
test "$(jq '[.records[] | select(.batch == "capability-status-projection")] | length' "$manifest")" -eq 6
test "$(jq '[.records[] | select(.batch == "capability-status-projection" and (.classification == "deferred-next-batch" or .classification == ""))] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "capability-status-projection" and ((.rustCounterparts | length) == 0 and (.reason | contains("no exact Rust counterpart") | not)))] | length' "$manifest")" -eq 0

jq '(.records[] | select(.referencePath == ".ai/project/adopter-capability-manifest.json") | .classification) = ""' "$manifest" > "$tmp/empty-capability-classification.json"
if python3 "$script" --manifest "$tmp/empty-capability-classification.json" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit 46e426625a8cae450f1190d0bdbafd6d8e648a90 --check; then
  echo "inventory accepted an empty scoped classification" >&2
  exit 1
fi

jq '.records[0].classification = "unclassified"' "$manifest" > "$tmp/invalid.json"
if python3 "$script" --manifest "$tmp/invalid.json" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit 46e426625a8cae450f1190d0bdbafd6d8e648a90 --check; then
  echo "inventory accepted an unclassified record" >&2
  exit 1
fi

echo "reference file inventory regression passed"
