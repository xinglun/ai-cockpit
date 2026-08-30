#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
manifest="$root/tests/conformance/reference_file_inventory.json"
script="$root/tests/conformance/reference_file_inventory.py"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/reference-file-inventory-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

python3 "$script" --manifest "$manifest" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b --check
python3 "$root/tests/conformance/reference_inventory_docs_test.py"

test "$(jq -r '.referenceTrackedFileCount' "$manifest")" -eq "$(jq '.records | length' "$manifest")"
test "$(jq -r '.targetWorkingTreeFileCount' "$manifest")" -eq "$(jq -r '.targetTrackedFileCount' "$manifest")"
test "$(jq -r '.targetWorkingTreePathDigest' "$manifest")" = "$(jq -r '.targetTrackedPathDigest' "$manifest")"
test "$(jq -r '.targetCommit' "$manifest")" = "bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b"
test "$(jq '[.records[] | select(.batch == "WI-302-reference-file-comparison-batch-01")] | length' "$manifest")" -eq 8
test "$(jq '[.records[] | select(.batch == "WI-302-reference-file-comparison-batch-01" and .classification == "deferred-next-batch")] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-304-reference-file-comparison-batch-02")] | length' "$manifest")" -eq 2
test "$(jq '[.records[] | select(.batch == "WI-304-reference-file-comparison-batch-02" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 2
test "$(jq '[.records[] | select(.batch == "WI-305-reference-file-comparison-batch-03")] | length' "$manifest")" -eq 4
test "$(jq '[.records[] | select(.batch == "WI-305-reference-file-comparison-batch-03" and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 4
test "$(jq -r '.records[] | select(.referencePath == "docs/architecture/interactive-installation-wizard.md") | .classification' "$manifest")" = "reference-only"
test "$(jq '[.records[] | select(.batch == "WI-305-reference-file-comparison-batch-03" and .classification == "deferred-next-batch")] | length' "$manifest")" -eq 0
test "$(jq -r '.records[] | select(.referencePath == ".ai/cockpit/bandit_low_risk_baseline.json") | .classification' "$manifest")" = "not-applicable"
test "$(jq -r '.records[] | select(.referencePath == ".github/workflows/release.yml") | .classification' "$manifest")" = "implemented-different-by-design"
test "$(jq -r '.records[] | select(.referencePath == "Makefile") | .classification' "$manifest")" = "implemented-different-by-design"
test "$(jq -r '.records[] | select(.referencePath == "CONTRIBUTING.md") | .classification' "$manifest")" = "implemented-different-by-design"
test "$(jq '[.records[] | select(.batch == "WI-308-reference-file-comparison-batch-04-retry")] | length' "$manifest")" -eq 4
test "$(jq '[.records[] | select(.batch == "WI-308-reference-file-comparison-batch-04-retry" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 3
test "$(jq -r '.records[] | select(.referencePath == "docs/assets/ai-cockpit-demo.gif") | .classification' "$manifest")" = "reference-only"
test "$(jq '[.records[] | select(.batch == "WI-308-reference-file-comparison-batch-04-retry" and .classification == "deferred-next-batch")] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-323-reference-documentation-foundation")] | length' "$manifest")" -eq 9
test "$(jq '[.records[] | select(.batch == "WI-323-reference-documentation-foundation" and ((.classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0) or .classification == "reference-only") and (.reason | length) > 0)] | length' "$manifest")" -eq 9
test "$(jq '[.records[] | select(.batch == "WI-323-reference-documentation-foundation" and .classification == "deferred-next-batch")] | length' "$manifest")" -eq 0
test "$(jq -r '.records[] | select(.referencePath == "docs/examples/trust-layer-demo.sh") | .classification' "$manifest")" = "reference-only"
test "$(jq -r '.records[] | select(.referencePath == "docs/features/human-benefit-report.md") | .classification' "$manifest")" = "implemented-different-by-design"
test "$(jq '[.records[] | select(.batch == "WI-326-reference-file-comparison-batch-06")] | length' "$manifest")" -eq 9
test "$(jq '[.records[] | select(.batch == "WI-326-reference-file-comparison-batch-06" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 8
test "$(jq '[.records[] | select(.batch == "WI-326-reference-file-comparison-batch-06" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 1
test "$(jq '[.records[] | select(.batch == "WI-326-reference-file-comparison-batch-06" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
test "$(jq -r '.records[] | select(.batch == "WI-326-reference-file-comparison-batch-06" and .referencePath == "docs/plans/harden-work-item-pr-closure.md") | .classification' "$manifest")" = "reference-only"
test "$(jq '[.records[] | select(.batch == "WI-327-reference-file-comparison-batch-07")] | length' "$manifest")" -eq 9
test "$(jq '[.records[] | select(.batch == "WI-327-reference-file-comparison-batch-07" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 8
test "$(jq '[.records[] | select(.batch == "WI-327-reference-file-comparison-batch-07" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 1
test "$(jq '[.records[] | select(.batch == "WI-327-reference-file-comparison-batch-07" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
test "$(jq -r '.records[] | select(.referencePath == "docs/reference/bandit-synchronization-security-audit.md") | .classification' "$manifest")" = "reference-only"
test "$(jq '[.records[] | select(.batch == "WI-328-reference-file-comparison-batch-08")] | length' "$manifest")" -eq 9
test "$(jq '[.records[] | select(.batch == "WI-328-reference-file-comparison-batch-08" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 5
test "$(jq '[.records[] | select(.batch == "WI-328-reference-file-comparison-batch-08" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 4
test "$(jq '[.records[] | select(.batch == "WI-328-reference-file-comparison-batch-08" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
test "$(jq -r '.records[] | select(.referencePath == "docs/reference/capability-truth-matrix.json") | .classification' "$manifest")" = "reference-only"
jq -r '.records[] | select(.referencePath == "docs/reference/capability-claim-authoring.md") | .reason' "$manifest" | grep -q "bounded capability-claim/evidence"
test "$(jq '[.records[] | select(.batch == "WI-331-reference-file-comparison-batch-09")] | length' "$manifest")" -eq 2
test "$(jq '[.records[] | select(.batch == "WI-331-reference-file-comparison-batch-09" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 2
test "$(jq '[.records[] | select(.batch == "WI-331-reference-file-comparison-batch-09" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-332-reference-file-comparison-batch-10")] | length' "$manifest")" -eq 3
test "$(jq '[.records[] | select(.batch == "WI-332-reference-file-comparison-batch-10" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 3
test "$(jq '[.records[] | select(.batch == "WI-332-reference-file-comparison-batch-10" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-333-reference-file-comparison-batch-11")] | length' "$manifest")" -eq 12
test "$(jq '[.records[] | select(.batch == "WI-333-reference-file-comparison-batch-11" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 12
test "$(jq '[.records[] | select(.batch == "WI-333-reference-file-comparison-batch-11" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-334-reference-file-comparison-batch-12")] | length' "$manifest")" -eq 10
test "$(jq '[.records[] | select(.batch == "WI-334-reference-file-comparison-batch-12" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 10
test "$(jq '[.records[] | select(.batch == "WI-334-reference-file-comparison-batch-12" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
wi347_paths=(
  docs/reference/human-report-semantic-quality.md
  docs/reference/implementation-knowledge.ja.md
  docs/reference/implementation-knowledge.md
  docs/reference/implementation-knowledge.zh-CN.md
  docs/reference/input-trust-dataflow.ja.md
  docs/reference/input-trust-dataflow.md
  docs/reference/input-trust-dataflow.zh-CN.md
  docs/reference/installed-lifecycle.md
  docs/reference/instruction-traceability.md
  docs/reference/japanese-capability-assessment.json
)
for wi347_path in "${wi347_paths[@]}"; do
  test "$(jq --arg path "$wi347_path" '[.records[] | select(.referencePath == $path and .batch == "WI-347-reference-knowledge-trust-lifecycle-assessment" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-347-reference-knowledge-trust-lifecycle-assessment")] | length' "$manifest")" -eq 10
test "$(jq '[.records[] | select(.batch == "WI-347-reference-knowledge-trust-lifecycle-assessment" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
wi368_paths=(
  docs/reference/pre-release-documentation-alignment.md
  docs/reference/pre-release-documentation-review.json
  docs/reference/project-test-timing-baseline.json
  docs/reference/provider-backed-governance-validation.md
  docs/reference/real-absurd-injection-cases.md
  docs/reference/real-absurd-injection-cases.zh-CN.md
  docs/reference/real-absurd-injection-cases.ja.md
  docs/reference/real-adopter-reference-validation.md
  docs/reference/reference-impact-gate.md
  docs/reference/reference-impact-gate.zh-CN.md
  docs/reference/reference-impact-gate.ja.md
)
for wi368_path in "${wi368_paths[@]}"; do
  test "$(jq --arg path "$wi368_path" '[.records[] | select(.referencePath == $path and .batch == "WI-368-reference-file-comparison-batch-16" and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.reason | length) > 0)] | length' "$manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-368-reference-file-comparison-batch-16")] | length' "$manifest")" -eq 11
test "$(jq '[.records[] | select(.batch == "WI-368-reference-file-comparison-batch-16" and .classification == "implemented-different-by-design")] | length' "$manifest")" -eq 6
test "$(jq '[.records[] | select(.batch == "WI-368-reference-file-comparison-batch-16" and .classification == "reference-only")] | length' "$manifest")" -eq 5
test "$(jq '[.records[] | select(.batch == "WI-368-reference-file-comparison-batch-16" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
test "$(jq -r '.records[] | select(.referencePath == "docs/reference/reference-impact-gate.md") | .classification' "$manifest")" = "reference-only"
wi411_paths=(
  examples/fixtures/java-multimodule/.gitignore
  examples/fixtures/java-multimodule/app/src/main/java/fixture/app/Main.java
  examples/fixtures/java-multimodule/app/src/test/java/fixture/app/MainTest.java
  examples/fixtures/java-multimodule/core/src/main/java/fixture/core/Decision.java
  examples/fixtures/java-multimodule/core/src/test/java/fixture/core/DecisionTest.java
  examples/fixtures/java-multimodule/evidence.json
  examples/fixtures/java-multimodule/fixture.json
  examples/fixtures/java-multimodule/pom.xml
  examples/fixtures/java-multimodule/scripts/lifecycle.sh
)
for wi411_path in "${wi411_paths[@]}"; do
  test "$(jq --arg path "$wi411_path" '[.records[] | select(.referencePath == $path and .batch == "WI-411-reference-java-fixture-boundary" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-411-reference-java-fixture-boundary")] | length' "$manifest")" -eq 9
test "$(jq '[.records[] | select(.batch == "WI-411-reference-java-fixture-boundary" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
grep -q "Java multi-module fixture" "$root/docs/reference/reference-file-comparison.md"
grep -q "Java 多模块 fixture" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "Java マルチモジュール fixture" "$root/docs/reference/reference-file-comparison.ja.md"
wi414_paths=(
  examples/fixtures/python/fixture.json
  examples/fixtures/python/pyproject.toml
  examples/fixtures/python/src/service.py
  examples/fixtures/python/tests/test_service.py
)
for wi414_path in "${wi414_paths[@]}"; do
  test "$(jq --arg path "$wi414_path" '[.records[] | select(.referencePath == $path and .batch == "WI-414-reference-python-fixture-boundary" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-414-reference-python-fixture-boundary")] | length' "$manifest")" -eq 4
test "$(jq '[.records[] | select(.batch == "WI-414-reference-python-fixture-boundary" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
grep -q "Python fixture boundary" "$root/docs/reference/reference-file-comparison.md"
grep -q "Python fixture 边界" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "Python fixture の境界" "$root/docs/reference/reference-file-comparison.ja.md"
grep -q "Python fixture adaptation" "$root/docs/reference/README.md"
grep -q "Python fixture 适配" "$root/docs/reference/README.zh-CN.md"
grep -q "Python fixture 適応" "$root/docs/reference/README.ja.md"
test "$(jq -r '.records[] | select(.batch == "governance-entrypoints" and .classification == "deferred-next-batch") | .referencePath' "$manifest" | wc -l | tr -d ' ')" -eq 0
grep -q "static reference-impact scanner is not a Rust Runtime" "$root/docs/reference/governance-profiles.md"
grep -q "静态 reference-impact scanner 不是本版本 Rust Runtime" "$root/docs/reference/governance-profiles.zh-CN.md"
grep -q "static reference-impact scanner は、この Release の Rust" "$root/docs/reference/governance-profiles.ja.md"
grep -q "命名场景数量不一致" "$root/docs/work-items/WI-368-reference-file-comparison-batch-16.zh-CN.md"
grep -q "named scenario count" "$root/docs/work-items/WI-368-reference-file-comparison-batch-16.ja.md"
test "$(jq '[.records[] | select(.batch == "capability-status-projection")] | length' "$manifest")" -eq 6
test "$(jq '[.records[] | select(.batch == "capability-status-projection" and (.classification == "deferred-next-batch" or .classification == ""))] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "capability-status-projection" and ((.rustCounterparts | length) == 0 and (.reason | contains("no exact Rust counterpart") | not)))] | length' "$manifest")" -eq 0
test "$(jq '[.records[] | select(.classification == "migrate-gap")] | length' "$manifest")" -eq 0
for gap in \
  .ai/project/adopter-capability-manifest.json \
  .ai/project/capabilities.json \
  .ai/project/success_criteria.json \
  .ai/project_profile.yaml; do
  test "$(jq -r --arg gap "$gap" '.records[] | select(.referencePath == $gap) | .classification' "$manifest")" = "implemented-different-by-design"
done

for agent_rule in \
  scripts/ai_check_agent_risk.py \
  templates/agents/AI_COCKPIT_RULES.md \
  tests/test_ai_check_agent_risk.py \
  tests/test_outcome_lifecycle_rules.py; do
  test "$(jq -r --arg path "$agent_rule" '.records[] | select(.referencePath == $path) | .batch' "$manifest")" = "WI-272-reference-agent-rule-batch"
  test "$(jq -r --arg path "$agent_rule" '.records[] | select(.referencePath == $path) | .classification' "$manifest")" = "implemented-different-by-design"
  test "$(jq -r --arg path "$agent_rule" '.records[] | select(.referencePath == $path) | (.rustCounterparts | length)' "$manifest")" -gt 0
done

jq '(.records[] | select(.referencePath == ".ai/project/adopter-capability-manifest.json") | .classification) = ""' "$manifest" > "$tmp/empty-capability-classification.json"
if python3 "$script" --manifest "$tmp/empty-capability-classification.json" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b --check; then
  echo "inventory accepted an empty scoped classification" >&2
  exit 1
fi

jq '.records[0].classification = "unclassified"' "$manifest" > "$tmp/invalid.json"
if python3 "$script" --manifest "$tmp/invalid.json" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b --check; then
  echo "inventory accepted an unclassified record" >&2
  exit 1
fi

jq '.targetWorkingTreeFileCount += 1' "$manifest" > "$tmp/working-tree-drift.json"
if python3 "$script" --manifest "$tmp/working-tree-drift.json" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b --check; then
  echo "inventory accepted target working-tree metadata outside the immutable baseline" >&2
  exit 1
fi

cp "$manifest" "$tmp/getting-started.json"
python3 "$script" \
  --manifest "$tmp/getting-started.json" \
  --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf \
  --target-commit bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b \
  --apply-getting-started-batch
test "$(jq '[.records[] | select(.referencePath | startswith("docs/getting-started/"))] | length' "$tmp/getting-started.json")" -eq 35
test "$(jq '[.records[] | select((.referencePath | startswith("docs/getting-started/")) and .batch == "getting-started-onboarding" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0)] | length' "$tmp/getting-started.json")" -eq 35
python3 "$script" \
  --manifest "$tmp/getting-started.json" \
  --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf \
  --target-commit bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b \
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
