#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
current_manifest="$root/tests/conformance/reference_file_inventory.json"
script="$root/tests/conformance/reference_file_inventory.py"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/reference-file-inventory-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

previous_manifest_revision=$(jq -r '.previousManifestGitRevision' "$current_manifest")
test "$previous_manifest_revision" != "null"
git show "${previous_manifest_revision}:tests/conformance/reference_file_inventory.json" > "$tmp/historical.json"
manifest="$tmp/historical.json"

python3 "$script" --manifest "$manifest" --source-commit e5acb677da6621004d96f0ef353c58fe8d3acfbf --target-commit bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b --check
python3 "$script" --manifest "$current_manifest" --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd --check

# A check must never regenerate the ledger.  Supplying generation arguments
# with --check is rejected before any write, and the input manifest remains
# byte-identical.
guarded_manifest="$tmp/guarded.json"
cp "$current_manifest" "$guarded_manifest"
before_digest=$(shasum -a 256 "$guarded_manifest" | awk '{print $1}')
if python3 "$script" --reference "$root" --target "$root" --manifest "$guarded_manifest" --check \
  --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 \
  --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd 2>"$tmp/guarded.stderr"; then
  echo "check mode unexpectedly accepted generation arguments" >&2
  exit 1
fi
grep -q -- "--check is read-only" "$tmp/guarded.stderr"
after_digest=$(shasum -a 256 "$guarded_manifest" | awk '{print $1}')
test "$before_digest" = "$after_digest"

python3 "$root/tests/conformance/reference_source_policy.py" --lock "$root/tests/conformance/reference-source.lock"
bash "$root/tests/conformance/reference_source_policy_check.sh"
python3 "$root/tests/conformance/reference_inventory_docs_test.py"

test "$(jq -r '.referenceTrackedFileCount' "$manifest")" -eq "$(jq '.records | length' "$manifest")"
test "$(jq -r '.referenceRepository' "$manifest")" = "local-git-checkout"
test "$(jq -r '.referencePathEnv' "$manifest")" = "AI_COCKPIT_REFERENCE_ROOT"
test "$(jq -r '.referenceNetworkAccess' "$manifest")" = "false"
test "$(jq -r '.targetWorkingTreeFileCount' "$manifest")" -eq "$(jq -r '.targetTrackedFileCount' "$manifest")"
test "$(jq -r '.targetWorkingTreePathDigest' "$manifest")" = "$(jq -r '.targetTrackedPathDigest' "$manifest")"
test "$(jq -r '.targetCommit' "$manifest")" = "bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b"
test "$(jq -r '.referenceTrackedFileCount' "$current_manifest")" -eq "$(jq '[.retiredReferencePaths[]] as $retired | [.records[] as $r | select(($retired | index($r.referencePath)) == null)] | length' "$current_manifest")"
test "$(jq -r '.referenceCommit' "$current_manifest")" = "fde3380f81fea5fd2e288f7a8849f737dc074060"
test "$(jq -r '.targetCommit' "$current_manifest")" = "cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd"
test "$(jq '[.retiredReferencePaths[]] as $retired | [.records[] as $r | select(($retired | index($r.referencePath)) == null)] | length' "$current_manifest")" -eq "$(jq -r '.referenceTrackedFileCount' "$current_manifest")"
test "$(jq -r '.referenceChangedPathCount' "$current_manifest")" -eq 160
test "$(jq -r '.referenceChangedPaths | length' "$current_manifest")" -eq 160
test "$(jq -r '.retiredReferencePathCount' "$current_manifest")" -eq 669
test "$(jq '.retiredReferencePaths | length' "$current_manifest")" -eq 669
test "$(jq '[.retiredReferencePaths[] | select((if type == "object" then .referencePath else . end) == ".ai/project/adopter-capability-manifest.json")] | length' "$current_manifest")" -eq 1
test "$(jq '[.records[] | select(.referencePath == ".ai/project/adopter-capability-manifest.json")] | length' "$current_manifest")" -eq 1
for current_capability_path in \
  .ai/project/capabilities.json \
  .ai/project/success_criteria.json \
  .ai/project_profile.yaml; do
  test "$(jq --arg path "$current_capability_path" '[.records[] | select(.referencePath == $path and .batch == "capability-status-projection" and .classification == "implemented-different-by-design")] | length' "$current_manifest")" -eq 1
done
# Bounded rebaseline batches resolve changed source records; WI-521 resolves
# one of the previously changed deferred records. Keep this regression count
# tied to the current pinned source ledger.
test "$(jq '[.records[] | select(.classification == "deferred-next-batch" and .sourceChangedSincePrevious == true and .previousClassification != null)] | length' "$current_manifest")" -eq 51
wi437_paths=(
  .ai/cockpit/README.ja.md
  .ai/cockpit/README.md
  .ai/cockpit/adoption.ja.md
  .ai/guards/changed_critical_coverage_policy.json
  .ai/guards/coverage_policy.yaml
  .ai/quality/governance-routing.yaml
  .ai/schemas/task_outcome.schema.json
)
for wi437_path in "${wi437_paths[@]}"; do
  test "$(jq --arg path "$wi437_path" '[.records[] | select(.referencePath == $path and .batch == "WI-437-reference-rebaseline-governance" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-437-reference-rebaseline-governance")] | length' "$current_manifest")" -eq 7
test "$(jq '[.records[] | select(.batch == "WI-437-reference-rebaseline-governance" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
wi441_paths=(
  AGENTS.md
  GEMINI.md
  docs/README.md
  docs/README.zh-CN.md
  docs/README.ja.md
  docs/capabilities.md
  docs/capabilities.zh-CN.md
  docs/capabilities.ja.md
  docs/features/task-outcome-report.md
)
for wi441_path in "${wi441_paths[@]}"; do
  test "$(jq --arg path "$wi441_path" '[.records[] | select(.referencePath == $path and .batch == "WI-441-reference-entrypoint-parity" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-441-reference-entrypoint-parity")] | length' "$current_manifest")" -eq 9
test "$(jq '[.records[] | select(.batch == "WI-441-reference-entrypoint-parity" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
wi461_paths=(
  docs/getting-started/first-work-item.md
  docs/getting-started/first-work-item.zh-CN.md
  docs/getting-started/first-work-item.ja.md
  docs/getting-started/security-release-verification.md
  docs/getting-started/security-release-verification.zh-CN.md
  docs/getting-started/security-release-verification.ja.md
  docs/getting-started/standard-adoption-guide.md
  docs/getting-started/standard-adoption-guide.zh-CN.md
  docs/getting-started/standard-adoption-guide.ja.md
)
for wi461_path in "${wi461_paths[@]}"; do
  test "$(jq --arg path "$wi461_path" '[.records[] | select(.referencePath == $path and .batch == "getting-started-onboarding" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "getting-started-onboarding" and (.referencePath as $p | $p == "docs/getting-started/first-work-item.md" or $p == "docs/getting-started/first-work-item.zh-CN.md" or $p == "docs/getting-started/first-work-item.ja.md" or $p == "docs/getting-started/security-release-verification.md" or $p == "docs/getting-started/security-release-verification.zh-CN.md" or $p == "docs/getting-started/security-release-verification.ja.md" or $p == "docs/getting-started/standard-adoption-guide.md" or $p == "docs/getting-started/standard-adoption-guide.zh-CN.md" or $p == "docs/getting-started/standard-adoption-guide.ja.md"))] | length' "$current_manifest")" -eq 9
wi464_paths=(
  .github/workflows/compatibility.yml
  .github/workflows/release.yml
  .github/workflows/smoke.yml
  Makefile
)
for wi464_path in "${wi464_paths[@]}"; do
  test "$(jq --arg path "$wi464_path" '[.records[] | select(.referencePath == $path and .batch == "WI-464-reference-file-comparison-batch-24" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-464-reference-file-comparison-batch-24")] | length' "$current_manifest")" -eq 4
test "$(jq '[.records[] | select(.batch == "WI-464-reference-file-comparison-batch-24" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
wi475_paths=(
  docs/features/human-benefit-report.md
  docs/features/human-benefit-report.zh-CN.md
  docs/features/human-benefit-report.ja.md
  docs/maintainers/task-outcome-events.md
  docs/operations/quality-gates.md
  docs/operations/quality-gates.zh-CN.md
  docs/operations/quality-gates.ja.md
)
for wi475_path in "${wi475_paths[@]}"; do
  test "$(jq --arg path "$wi475_path" '[.records[] | select(.referencePath == $path and .batch == "WI-475-reference-file-comparison-batch-25" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0 and .sourceChangedSincePrevious == true and .previousClassification != null)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-475-reference-file-comparison-batch-25")] | length' "$current_manifest")" -eq 7
test "$(jq '[.records[] | select(.batch == "WI-475-reference-file-comparison-batch-25" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
grep -q "WI-441 local-reference entrypoint and Agent parity" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-441：本地参考源入口与 Agent 语义对齐" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-441 local-reference entrypoint と Agent parity" "$root/docs/reference/reference-file-comparison.ja.md"
grep -q "WI-475" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-475" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-475" "$root/docs/reference/reference-file-comparison.ja.md"
wi482_paths=(
  docs/operations/work-item-lifecycle.md
  docs/operations/work-item-lifecycle.zh-CN.md
  docs/operations/work-item-lifecycle.ja.md
  docs/reference/agent-parallel-work-items.md
  docs/reference/ai-cockpit-work-item-lifecycle.md
  docs/trust-layer.md
  docs/trust-layer.zh-CN.md
  docs/trust-layer.ja.md
)
for wi482_path in "${wi482_paths[@]}"; do
  test "$(jq --arg path "$wi482_path" '[.records[] | select(.referencePath == $path and .batch == "WI-482-reference-file-comparison-batch-26" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0 and .sourceChangedSincePrevious == true and .previousClassification != null)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-482-reference-file-comparison-batch-26")] | length' "$current_manifest")" -eq 8
test "$(jq '[.records[] | select(.batch == "WI-482-reference-file-comparison-batch-26" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
grep -q "WI-482" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-482" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-482" "$root/docs/reference/reference-file-comparison.ja.md"
wi494_paths=(
  docs/reference/capability-truth-matrix.json
  docs/reference/comprehension-validation-responses/peter_01.en.json
  docs/reference/comprehension-validation-responses/tanaka_01.ja.json
  docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json
  docs/reference/comprehension-validation-results.json
  docs/reference/comprehension-validation-results.md
  docs/reference/deprecated-assets-registry.json
)
for wi494_path in "${wi494_paths[@]}"; do
  test "$(jq --arg path "$wi494_path" '[.records[] | select(.referencePath == $path and .batch == "WI-494-reference-file-comparison-batch-27" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0 and .sourceChangedSincePrevious == true and .previousClassification == "reference-only")] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-494-reference-file-comparison-batch-27")] | length' "$current_manifest")" -eq 7
test "$(jq '[.records[] | select(.batch == "WI-494-reference-file-comparison-batch-27" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
grep -q "WI-494" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-494" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-494" "$root/docs/reference/reference-file-comparison.ja.md"
wi496_paths=(
  docs/reference/distribution.md
  docs/reference/distribution.ja.md
  docs/reference/documentation-context-registry.json
  docs/reference/governance-profiles.md
  docs/reference/governance-profiles.zh-CN.md
  docs/reference/governance-profiles.ja.md
  docs/reference/japanese-capability-assessment.json
  docs/reference/japanese-capability-assessment.md
  docs/reference/pre-release-documentation-alignment.json
  docs/reference/pre-release-documentation-alignment.md
)
for wi496_path in "${wi496_paths[@]}"; do
  test "$(jq --arg path "$wi496_path" '[.records[] | select(.referencePath == $path and .batch == "WI-496-reference-file-comparison-batch-28" and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.rustCounterparts | length) > 0 and (.reason | length) > 0 and .sourceChangedSincePrevious == true and .previousClassification != null)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-496-reference-file-comparison-batch-28")] | length' "$current_manifest")" -eq 10
test "$(jq '[.records[] | select(.batch == "WI-496-reference-file-comparison-batch-28" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
grep -q "WI-496" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-496" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-496" "$root/docs/reference/reference-file-comparison.ja.md"
wi504_paths=(
  docs/reference/repository-workflow.ja.md
  docs/reference/troubleshooting.md
  docs/reference/verification-evidence-reuse.md
  docs/reference/work-item-lifecycle-closure.md
  docs/upgrade.md
)
for wi504_path in "${wi504_paths[@]}"; do
  test "$(jq --arg path "$wi504_path" '[.records[] | select(.referencePath == $path and .batch == "WI-504-reference-file-comparison-batch-29" and .classification == "implemented-different-by-design" and (.rustCounterparts | length) > 0 and (.reason | length) > 0 and .sourceChangedSincePrevious == true and .previousClassification != null)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-504-reference-file-comparison-batch-29")] | length' "$current_manifest")" -eq 5
test "$(jq '[.records[] | select(.batch == "WI-504-reference-file-comparison-batch-29" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
grep -q "WI-504" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-504" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-504" "$root/docs/reference/reference-file-comparison.ja.md"
wi508_paths=(
  examples/python/README.md
  examples/ruby/README.md
  examples/rust/README.md
  examples/swift/README.md
  examples/typescript/README.md
)
for wi508_path in "${wi508_paths[@]}"; do
  test "$(jq --arg path "$wi508_path" '[.records[] | select(.referencePath == $path and .batch == "WI-508-reference-file-comparison-batch-31" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-508-reference-file-comparison-batch-31")] | length' "$current_manifest")" -eq 5
test "$(jq '[.records[] | select(.batch == "WI-508-reference-file-comparison-batch-31" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
grep -q "WI-508" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-508" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-508" "$root/docs/reference/reference-file-comparison.ja.md"
wi512_paths=(
  docs/reference/schemas.md
  docs/reference/test-architecture.md
  docs/reference/test-weakening-guard.md
  docs/reference/test-weakening-guard.zh-CN.md
  docs/reference/test-weakening-guard.ja.md
  docs/reference/verification-fixture-boundary.md
  docs/reference/troubleshooting.md
  docs/reference/troubleshooting.ja.md
  docs/reference/upgrade.md
  docs/reference/upgrade.ja.md
  docs/reference/work-item-lifecycle-closure.md
  docs/reference/work-item-lifecycle-closure.ja.md
)
for wi512_path in "${wi512_paths[@]}"; do
  test "$(jq --arg path "$wi512_path" '[.records[] | select(.referencePath == $path and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-512-reference-docs-batch-33")] | length' "$current_manifest")" -eq 10
test "$(jq '[.records[] | select(.batch == "WI-512-reference-docs-batch-33" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
grep -q "WI-512" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-512" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-512" "$root/docs/reference/reference-file-comparison.ja.md"
test "$(jq '(.records | map(.referencePath)) as $recordPaths | (.retiredReferencePaths) as $retiredPaths | (($recordPaths - $retiredPaths) | length) == (.referenceTrackedFileCount)' "$current_manifest")" = "true"
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
wi432_paths=(
  examples/fixtures/typescript-web/.gitignore
  examples/fixtures/typescript-web/evidence.json
  examples/fixtures/typescript-web/fixture.json
  examples/fixtures/typescript-web/package-lock.json
  examples/fixtures/typescript-web/package.json
  examples/fixtures/typescript-web/scripts/format-check.mjs
  examples/fixtures/typescript-web/scripts/lifecycle.mjs
  examples/fixtures/typescript-web/scripts/lint.mjs
  examples/fixtures/typescript-web/src/index.ts
  examples/fixtures/typescript-web/test/index.test.mjs
  examples/fixtures/typescript-web/tsconfig.json
)
for wi432_path in "${wi432_paths[@]}"; do
  test "$(jq --arg path "$wi432_path" '[.records[] | select(.referencePath == $path and .batch == "WI-432-reference-typescript-fixture-boundary" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-432-reference-typescript-fixture-boundary")] | length' "$manifest")" -eq 11
test "$(jq '[.records[] | select(.batch == "WI-432-reference-typescript-fixture-boundary" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
grep -q "TypeScript web fixture adaptation" "$root/docs/reference/README.md"
grep -q "TypeScript Web fixture 适配" "$root/docs/reference/README.zh-CN.md"
grep -q "TypeScript Web fixture 適応" "$root/docs/reference/README.ja.md"
grep -q "TypeScript web fixture boundary" "$root/docs/reference/reference-file-comparison.md"
grep -q "TypeScript web fixture 边界" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "TypeScript web fixture の境界" "$root/docs/reference/reference-file-comparison.ja.md"
wi421_paths=(
  examples/fixtures/mixed-monorepo/fixture.json
  examples/fixtures/mixed-monorepo/package.json
  examples/fixtures/mixed-monorepo/pyproject.toml
  examples/fixtures/mixed-monorepo/services/api/app.py
  examples/fixtures/mixed-monorepo/services/api/tests/test_app.py
)
for wi421_path in "${wi421_paths[@]}"; do
  test "$(jq --arg path "$wi421_path" '[.records[] | select(.referencePath == $path and .batch == "WI-421-reference-mixed-monorepo" and .classification == "reference-only" and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-421-reference-mixed-monorepo")] | length' "$manifest")" -eq 5
test "$(jq '[.records[] | select(.batch == "WI-421-reference-mixed-monorepo" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$manifest")" -eq 0
grep -q "mixed-monorepo fixture boundary" "$root/docs/reference/reference-file-comparison.md"
grep -q "mixed-monorepo fixture 边界" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "mixed-monorepo fixture 境界" "$root/docs/reference/reference-file-comparison.ja.md"
grep -q "Mixed-monorepo fixture adaptation" "$root/docs/reference/README.md"
grep -q "Mixed-monorepo fixture 适配" "$root/docs/reference/README.zh-CN.md"
grep -q "Mixed-monorepo fixture 適応" "$root/docs/reference/README.ja.md"
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

# WI-550 records the lifecycle/Outcome/trust/observability comparison one
# source path at a time. Keep the exact slice and classifications explicit so
# a rebaseline cannot silently return any of it to deferred work.
wi550_paths=(
  scripts/ai_finish.py
  scripts/ai_generate_human_report.py
  scripts/ai_generate_status.py
  scripts/ai_generate_task_outcome.py
  scripts/ai_governance_compression.py
  scripts/ai_input_trust.py
  scripts/ai_japanese_capability.py
  scripts/ai_lifecycle_facts.py
  scripts/ai_lifecycle_truth.py
  scripts/ai_multilingual_semantic_parity.py
  scripts/ai_observability.py
  scripts/ai_post_archive_recovery.py
  scripts/ai_render_task_outcome.py
  scripts/ai_render_task_outcome_multilingual.py
  scripts/ai_render_task_outcome_pr.py
  scripts/ai_required_evidence.py
)
for wi550_path in "${wi550_paths[@]}"; do
  test "$(jq --arg path "$wi550_path" '[.records[] | select(.referencePath == $path and .batch == "WI-550-reference-file-comparison-batch-39" and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-550-reference-file-comparison-batch-39")] | length' "$current_manifest")" -eq 16
test "$(jq '[.records[] | select(.batch == "WI-550-reference-file-comparison-batch-39" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-550-reference-file-comparison-batch-39" and .classification == "reference-only")] | length' "$current_manifest")" -eq 1
grep -q "WI-550" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-550" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-550" "$root/docs/reference/reference-file-comparison.ja.md"
grep -q "WI-550" "$root/docs/reference/reference-parity.md"
grep -q "WI-550" "$root/docs/reference/reference-parity.zh-CN.md"
grep -q "WI-550" "$root/docs/reference/reference-parity.ja.md"

# WI-552 compares the pinned installer/upgrade paths one by one.  The source
# installer remains an explicit boundary: repository onboarding and migration
# are Rust-native, while provider catalogs and the Python launcher are not
# copied into the shared Runtime.
wi552_paths=(
  scripts/ai_install_facts.py
  scripts/ai_install_plan.py
  scripts/ai_install_status.py
  scripts/ai_install_wizard.py
  scripts/ai_installer_bootstrap.py
  scripts/ai_installer_catalog.json
  scripts/ai_installer_detection.py
  scripts/ai_installer_evidence.py
  scripts/ai_installer_managed_regions.py
  scripts/ai_installer_ownership.py
  scripts/ai_installer_repository.py
  scripts/ai_installer_transaction.py
  scripts/ai_installer_upgrade.py
  scripts/ai_upgrade_apply.py
  scripts/ai_upgrade_conflict_report.py
  scripts/ai_upgrade_proposal.py
  scripts/install_ai_cockpit.py
)
for wi552_path in "${wi552_paths[@]}"; do
  test "$(jq --arg path "$wi552_path" '[.records[] | select(.referencePath == $path and .batch == "WI-552-reference-file-comparison-batch-40" and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-552-reference-file-comparison-batch-40")] | length' "$current_manifest")" -eq 17
test "$(jq '[.records[] | select(.batch == "WI-552-reference-file-comparison-batch-40" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-552-reference-file-comparison-batch-40" and .classification == "reference-only")] | length' "$current_manifest")" -eq 2
grep -q "WI-552" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-552" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-552" "$root/docs/reference/reference-file-comparison.ja.md"
grep -q "WI-552" "$root/docs/reference/reference-parity.md"
grep -q "WI-552" "$root/docs/reference/reference-parity.zh-CN.md"
grep -q "WI-552" "$root/docs/reference/reference-parity.ja.md"

# WI-557 compares the next thirteen maintained reference scripts one by one.
# Keep the exact slice and explicit classifications in the regression so a
# future rebaseline cannot silently return these paths to deferred work.
wi557_paths=(
  scripts/ai_issue_log.py
  scripts/ai_linked_worktree_recovery.py
  scripts/ai_ownership.py
  scripts/ai_performance_budget.py
  scripts/ai_project_profile.py
  scripts/ai_purge.py
  scripts/ai_readiness_policy.py
  scripts/ai_recovery_usability.py
  scripts/ai_review_readiness_policy.py
  scripts/ai_risk_policy.py
  scripts/ai_rollback.py
  scripts/ai_safety_gate.py
  scripts/ai_schema_migration.py
)
for wi557_path in "${wi557_paths[@]}"; do
  test "$(jq --arg path "$wi557_path" '[.records[] | select(.referencePath == $path and .batch == "WI-557-reference-file-comparison-batch-41" and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-557-reference-file-comparison-batch-41")] | length' "$current_manifest")" -eq 13
test "$(jq '[.records[] | select(.batch == "WI-557-reference-file-comparison-batch-41" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-557-reference-file-comparison-batch-41" and .classification == "reference-only")] | length' "$current_manifest")" -eq 1
grep -q "WI-557" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-557" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-557" "$root/docs/reference/reference-file-comparison.ja.md"
grep -q "WI-557" "$root/docs/reference/reference-parity.md"
grep -q "WI-557" "$root/docs/reference/reference-parity.zh-CN.md"
grep -q "WI-557" "$root/docs/reference/reference-parity.ja.md"

# WI-559 compares the next twenty maintained source scripts one by one.
wi559_paths=(
  scripts/ai_onboard.py
  scripts/ai_prepare_hosted_verification.py
  scripts/ai_project_doctor.py
  scripts/ai_projection_lease.py
  scripts/ai_provider_merge_state_recovery.py
  scripts/ai_quality_architecture.py
  scripts/ai_resume_work_item.py
  scripts/ai_start.py
  scripts/ai_start_receipt.py
  scripts/ai_task_event_log.py
  scripts/ai_terminology.py
  scripts/ai_trust_guards.py
  scripts/ai_trust_schema.py
  scripts/ai_uninstall_facts.py
  scripts/ai_uninstall_proposal.py
  scripts/ai_unknown_confirmation.py
  scripts/ai_validate_java_runtime.py
  scripts/ai_verification_context.py
  scripts/ai_verification_policy.py
  scripts/ai_verify.py
)
for wi559_path in "${wi559_paths[@]}"; do
  test "$(jq --arg path "$wi559_path" '[.records[] | select(.referencePath == $path and .batch == "WI-559-reference-file-comparison-batch-42" and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-559-reference-file-comparison-batch-42")] | length' "$current_manifest")" -eq 20
test "$(jq '[.records[] | select(.batch == "WI-559-reference-file-comparison-batch-42" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-559-reference-file-comparison-batch-42" and .classification == "reference-only")] | length' "$current_manifest")" -eq 3
grep -q "WI-559" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-559" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-559" "$root/docs/reference/reference-file-comparison.ja.md"
grep -q "WI-559" "$root/docs/reference/reference-parity.md"
grep -q "WI-559" "$root/docs/reference/reference-parity.zh-CN.md"
grep -q "WI-559" "$root/docs/reference/reference-parity.ja.md"

# WI-563 compares the next twenty maintained source scripts one by one.
# Keep this exact slice and its explicit non-claims in the regression so a
# future rebaseline cannot silently return the paths to deferred work.
wi563_paths=(
  scripts/ai_wizard_io.py
  scripts/ai_wizard_localization.py
  scripts/ai_work_item_intelligence.py
  scripts/ai_work_item_intelligence_benchmark.py
  scripts/ai_work_item_status.py
  scripts/bootstrap_repository.py
  scripts/bootstrap_wizard.py
  scripts/bootstrap_write_boundary.py
  scripts/check_bandit_baseline.py
  scripts/check_changed_critical_coverage.py
  scripts/check_ci_release_evidence.sh
  scripts/check_critical_coverage.py
  scripts/check_deprecated_assets.py
  scripts/check_dev_tool_versions.py
  scripts/check_docs_metadata.py
  scripts/check_governance_complexity.py
  scripts/check_instruction_traceability.py
  scripts/check_pre_release_documentation_alignment.py
  scripts/check_real_absurd_injection_docs.py
  scripts/check_release_distribution.py
)
for wi563_path in "${wi563_paths[@]}"; do
  test "$(jq --arg path "$wi563_path" '[.records[] | select(.referencePath == $path and .batch == "WI-563-reference-file-comparison-batch-43" and (.classification == "implemented-different-by-design" or .classification == "reference-only" or .classification == "not-applicable") and (.classification == "not-applicable" or (.rustCounterparts | length) > 0) and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-563-reference-file-comparison-batch-43")] | length' "$current_manifest")" -eq 20
test "$(jq '[.records[] | select(.batch == "WI-563-reference-file-comparison-batch-43" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-563-reference-file-comparison-batch-43" and .classification == "reference-only")] | length' "$current_manifest")" -eq 5
test "$(jq '[.records[] | select(.batch == "WI-563-reference-file-comparison-batch-43" and .classification == "not-applicable")] | length' "$current_manifest")" -eq 1
grep -q "WI-563" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-563" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-563" "$root/docs/reference/reference-file-comparison.ja.md"
grep -q "WI-563" "$root/docs/reference/reference-parity.md"
grep -q "WI-563" "$root/docs/reference/reference-parity.zh-CN.md"
grep -q "WI-563" "$root/docs/reference/reference-parity.ja.md"

# WI-568 compares the next twenty maintained release, governance, adopter,
# and installer paths one by one.  Keep the exact set and classification
# counts in the regression to prevent silent return to deferred work.
wi568_paths=(
  scripts/check_release_preflight.py
  scripts/check_release_state_consistency.py
  scripts/check_supply_chain.py
  scripts/check_system_invariants.py
  scripts/check_trust_layer_docs.py
  scripts/cross_stack_long_cycle.py
  scripts/determine_governance_profile.py
  scripts/determine_quality_scope.py
  scripts/end_to_end_adoption_validation.py
  scripts/ensure_locked_dev_environment.py
  scripts/external_adopter_long_cycle.py
  scripts/finalize_release_freeze.py
  scripts/fixture_harness.py
  scripts/installed_lifecycle_e2e.py
  scripts/installer/__init__.py
  scripts/installer/application.py
  scripts/installer/cli.py
  scripts/installer/confirmation.py
  scripts/installer/conflict_matrix.py
  scripts/installer/evidence.py
)
for wi568_path in "${wi568_paths[@]}"; do
  test "$(jq --arg path "$wi568_path" '[.records[] | select(.referencePath == $path and .batch == "WI-568-reference-file-comparison-batch-44" and (.classification == "implemented-different-by-design" or .classification == "reference-only") and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-568-reference-file-comparison-batch-44")] | length' "$current_manifest")" -eq 20
test "$(jq '[.records[] | select(.batch == "WI-568-reference-file-comparison-batch-44" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
test "$(jq '[.records[] | select(.batch == "WI-568-reference-file-comparison-batch-44" and .classification == "implemented-different-by-design")] | length' "$current_manifest")" -eq 17
test "$(jq '[.records[] | select(.batch == "WI-568-reference-file-comparison-batch-44" and .classification == "reference-only")] | length' "$current_manifest")" -eq 3
for wi568_doc in \
  reference-file-comparison.md reference-file-comparison.zh-CN.md reference-file-comparison.ja.md \
  reference-parity.md reference-parity.zh-CN.md reference-parity.ja.md; do
  grep -q "WI-568" "$root/docs/reference/$wi568_doc"
done

# WI-521 resolves the next pinned local source scripts one by one.  Keep this
# explicit so a future rebaseline cannot silently return the slice to deferred
# without a new human-owned comparison decision.
for wi521_path in \
  scripts/ai_check_adoption_ready.py \
  scripts/ai_check_archive_recovery.py \
  scripts/ai_check_backtrack.py \
  scripts/ai_check_budget_impact.py \
  scripts/ai_check_capability_claims.py \
  scripts/ai_check_coverage_guard.py \
  scripts/ai_check_dependabot_intake.py \
  scripts/ai_check_diff_ownership.py \
  scripts/ai_check_guard_calibration.py \
  scripts/ai_check_guards.py \
  tests/test_ai_check_archive_recovery.py \
  tests/test_ai_check_budget_impact.py; do
  test "$(jq --arg path "$wi521_path" '[.records[] | select(.referencePath == $path and .batch == "WI-521-reference-file-comparison-batch-35" and (.classification == "implemented-different-by-design" or .classification == "reference-only" or .classification == "not-applicable") and (.rustCounterparts | length) > 0 and (.reason | length) > 0)] | length' "$current_manifest")" -eq 1
done
test "$(jq '[.records[] | select(.batch == "WI-521-reference-file-comparison-batch-35")] | length' "$current_manifest")" -eq 12
test "$(jq '[.records[] | select(.batch == "WI-521-reference-file-comparison-batch-35" and (.classification == "deferred-next-batch" or .classification == "migrate-gap"))] | length' "$current_manifest")" -eq 0
grep -q "WI-521" "$root/docs/reference/reference-file-comparison.md"
grep -q "WI-521" "$root/docs/reference/reference-file-comparison.zh-CN.md"
grep -q "WI-521" "$root/docs/reference/reference-file-comparison.ja.md"

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
