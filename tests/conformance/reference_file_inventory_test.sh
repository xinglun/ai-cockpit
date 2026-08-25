#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
manifest="$root/tests/conformance/reference_file_inventory.json"
script="$root/tests/conformance/reference_file_inventory.py"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/reference-file-inventory-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

python3 "$script" --manifest "$manifest" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit 87bfd86645adf7f4a6f86e447763542988371039 --check

test "$(jq -r '.referenceTrackedFileCount' "$manifest")" -eq "$(jq '.records | length' "$manifest")"
test "$(jq -r '.targetWorkingTreeFileCount' "$manifest")" -eq "$(jq -r '.targetTrackedFileCount' "$manifest")"
test "$(jq -r '.targetWorkingTreePathDigest' "$manifest")" = "$(jq -r '.targetTrackedPathDigest' "$manifest")"
test "$(jq -r '.records[] | select(.referencePath == "CONTRIBUTING.md") | .classification' "$manifest")" = "implemented-different-by-design"
test "$(jq -r '.records[] | select(.batch == "governance-entrypoints" and .classification == "deferred-next-batch") | .referencePath' "$manifest" | wc -l | tr -d ' ')" -eq 0
test "$(jq '[.records[] | select(.batch == "capability-status-projection")] | length' "$manifest")" -eq 6
test "$(jq '[.records[] | select(.batch == "capability-status-projection" and (.classification == "deferred-next-batch" or .classification == ""))] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "capability-status-projection" and ((.rustCounterparts | length) == 0 and (.reason | contains("no exact Rust counterpart") | not)))] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.classification == "migrate-gap")] | length' "$manifest")" -eq 4
for gap in \
  .ai/project/adopter-capability-manifest.json \
  .ai/project/capabilities.json \
  .ai/project/success_criteria.json \
  .ai/project_profile.yaml; do
  test "$(jq -r --arg gap "$gap" '.records[] | select(.referencePath == $gap) | .classification' "$manifest")" = "migrate-gap"
done

jq '(.records[] | select(.referencePath == ".ai/project/adopter-capability-manifest.json") | .classification) = ""' "$manifest" > "$tmp/empty-capability-classification.json"
if python3 "$script" --manifest "$tmp/empty-capability-classification.json" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit 87bfd86645adf7f4a6f86e447763542988371039 --check; then
  echo "inventory accepted an empty scoped classification" >&2
  exit 1
fi

jq '.records[0].classification = "unclassified"' "$manifest" > "$tmp/invalid.json"
if python3 "$script" --manifest "$tmp/invalid.json" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit 87bfd86645adf7f4a6f86e447763542988371039 --check; then
  echo "inventory accepted an unclassified record" >&2
  exit 1
fi

jq '.targetWorkingTreeFileCount += 1' "$manifest" > "$tmp/working-tree-drift.json"
if python3 "$script" --manifest "$tmp/working-tree-drift.json" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit 87bfd86645adf7f4a6f86e447763542988371039 --check; then
  echo "inventory accepted target working-tree metadata outside the immutable baseline" >&2
  exit 1
fi

cp "$manifest" "$tmp/getting-started.json"
python3 "$script" \
  --manifest "$tmp/getting-started.json" \
  --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf \
  --target-commit 87bfd86645adf7f4a6f86e447763542988371039 \
  --apply-getting-started-batch
test "$(jq '[.records[] | select(.referencePath | startswith("docs/getting-started/"))] | length' "$tmp/getting-started.json")" -eq 35
test "$(jq '[.records[] | select((.referencePath | startswith("docs/getting-started/")) and .batch == "getting-started-onboarding" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0)] | length' "$tmp/getting-started.json")" -eq 35
python3 "$script" \
  --manifest "$tmp/getting-started.json" \
  --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf \
  --target-commit 87bfd86645adf7f4a6f86e447763542988371039 \
  --check

reference_fixture="$tmp/reference"
target_fixture="$tmp/target"
git init -q "$reference_fixture"
git -C "$reference_fixture" config user.name fixture
git -C "$reference_fixture" config user.email fixture@example.invalid
mkdir -p \
  "$reference_fixture/.ai/project" \
  "$reference_fixture/.ai/cockpit/work-items"
for capability_path in \
  .ai/project/adopter-capability-manifest.json \
  .ai/project/capabilities.json \
  .ai/project/success_criteria.json \
  .ai/project_profile.yaml \
  .ai/cockpit/work-items/index.json \
  .ai/cockpit/work-items/wi-06-status-interface.status.json; do
  printf 'fixture\n' > "$reference_fixture/$capability_path"
done
printf 'reference\n' > "$reference_fixture/AGENTS.md"
mkdir -p "$reference_fixture/docs/concepts"
printf 'reference decision states\n' > "$reference_fixture/docs/concepts/decision-states.md"
git -C "$reference_fixture" add AGENTS.md .ai docs/concepts/decision-states.md
git -C "$reference_fixture" commit -qm reference
reference_revision=$(git -C "$reference_fixture" rev-parse HEAD)

git init -q "$target_fixture"
git -C "$target_fixture" config user.name fixture
git -C "$target_fixture" config user.email fixture@example.invalid
printf 'target\n' > "$target_fixture/AGENTS.md"
git -C "$target_fixture" add AGENTS.md
git -C "$target_fixture" commit -qm baseline
target_revision=$(git -C "$target_fixture" rev-parse HEAD)
printf 'later\n' > "$target_fixture/later.txt"
git -C "$target_fixture" add later.txt
git -C "$target_fixture" commit -qm later
printf 'untracked\n' > "$target_fixture/untracked.txt"

python3 "$script" \
  --reference "$reference_fixture" \
  --target "$target_fixture" \
  --source-commit "$reference_revision" \
  --target-commit "$target_revision" \
  --output "$tmp/generated.json"
test "$(jq -r '.targetTrackedFileCount' "$tmp/generated.json")" -eq 1
test "$(jq -r '.targetWorkingTreeFileCount' "$tmp/generated.json")" -eq 1
test "$(jq '[.records[] | select(.batch == "WI-270-reference-contract-batch")] | length' "$tmp/generated.json")" -eq 1
test "$(jq -r '.records[] | select(.referencePath == "docs/concepts/decision-states.md") | .classification' "$tmp/generated.json")" = "implemented-different-by-design"

echo "reference file inventory regression passed"
