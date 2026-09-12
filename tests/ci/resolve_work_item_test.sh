#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
resolver="$root/tests/ci/resolve_work_item.sh"
fixture=$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-work-item-resolution.XXXXXX")
trap 'rm -rf "$fixture"' EXIT

git init -q "$fixture/repo"
git -C "$fixture/repo" config user.name "Work Item Resolution Test"
git -C "$fixture/repo" config user.email "work-item-resolution@example.invalid"
mkdir -p "$fixture/repo/.ai/work-items/active"
mkdir -p "$fixture/repo/.ai/decisions"
cat > "$fixture/repo/.ai/agent-interface.json" <<'JSON'
{"repositoryId":"fixture-repository"}
JSON
git -C "$fixture/repo" add .
git -C "$fixture/repo" commit -qm base
base=$(git -C "$fixture/repo" rev-parse HEAD)
printf 'head\n' > "$fixture/repo/README.md"
git -C "$fixture/repo" add README.md
git -C "$fixture/repo" commit -qm head
head=$(git -C "$fixture/repo" rev-parse HEAD)

cat > "$fixture/repo/.ai/work-items/active/WI-TEST.contract.json" <<JSON
{
  "workItemId": "WI-TEST",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "resourceContext": {
    "branch": "codex/test-route",
    "pullRequest": "https://github.com/example/repo/pull/7"
  },
  "predecessorWorkItemId": "WI-PREVIOUS",
  "predecessorContractDigest": "sha256:previous",
  "recoveryDecisionPath": ".ai/decisions/WI-TEST.recovery.json",
  "scope": ["README.md"]
}
JSON
cat > "$fixture/repo/.ai/decisions/WI-TEST.recovery.json" <<'JSON'
{"decision":"successor","predecessorWorkItemId":"WI-PREVIOUS","successorWorkItemId":"WI-TEST"}
JSON

cat > "$fixture/repo/.ai/work-items/active/WI-SOURCE.contract.json" <<JSON
{
  "workItemId": "WI-SOURCE",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$head",
  "scope": ["README.md"]
}
JSON

pull_request_output="$fixture/pull-request.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/test-route \
  --pr-url https://github.com/example/repo/pull/7 \
  --output "$pull_request_output"
jq -e \
  '.mode == "pull_request" and .workItemId == "WI-TEST" and .contractPath == ".ai/work-items/active/WI-TEST.contract.json" and .baseRevision == $base' \
  --arg base "$base" "$pull_request_output" >/dev/null

recovery_output="$fixture/recovery.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event workflow_dispatch \
  --head "$head" \
  --from-tag v0.2.90 \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --work-item-id WI-TEST \
  --release-source-revision "$base" \
  --output "$recovery_output"
jq -e \
  '.mode == "release_recovery" and .workItemId == "WI-TEST" and .recoveryLineage == "successor" and .releaseSourceRevision == $source and .headRevision == $head' \
  --arg source "$base" --arg head "$head" "$recovery_output" >/dev/null

dual_identity_output="$fixture/dual-identity.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event workflow_dispatch \
  --head "$head" \
  --from-tag v0.2.90 \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --work-item-id WI-TEST \
  --source-work-item-id WI-SOURCE \
  --release-source-revision "$base" \
  --output "$dual_identity_output"
jq -e \
  '.workItemId == "WI-TEST" and .contractPath == ".ai/work-items/active/WI-TEST.contract.json" and .sourceWorkItemId == "WI-SOURCE" and .sourceContractPath == ".ai/work-items/active/WI-SOURCE.contract.json" and .sourceBaseRevision == $source_base and .sourceSelectionMethod == "explicit_source_work_item_id" and .contractDigest != .sourceContractDigest' \
  --arg source_base "$head" "$dual_identity_output" >/dev/null

default_source_output="$fixture/default-source.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event workflow_dispatch \
  --head "$head" \
  --from-tag v0.2.90 \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --work-item-id WI-TEST \
  --release-source-revision "$base" \
  --output "$default_source_output"
jq -e \
  '.sourceWorkItemId == .workItemId and .sourceContractPath == .contractPath and .sourceContractDigest == .contractDigest and .sourceBaseRevision == .baseRevision and .sourceSelectionMethod == "same_as_governance"' \
  "$default_source_output" >/dev/null

if "$resolver" \
  --repo "$fixture/repo" \
  --event workflow_dispatch \
  --head "$head" \
  --from-tag v0.2.90 \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --work-item-id WI-TEST \
  --source-work-item-id WI-MISSING \
  --release-source-revision "$base" \
  --output "$fixture/missing-source.json" >/dev/null 2>&1; then
  echo 'expected missing source Work Item to fail' >&2
  exit 1
fi
jq -e '.failureCode == "source_contract_not_regular" and .state == "failed"' "$fixture/missing-source.json" >/dev/null

if "$resolver" \
  --repo "$fixture/repo" \
  --event workflow_dispatch \
  --head "$head" \
  --from-tag v0.2.90 \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --work-item-id WI-TEST \
  --source-work-item-id WI-SOURCE \
  --source-contract-path .ai/work-items/active/WI-TEST.contract.json \
  --release-source-revision "$base" \
  --output "$fixture/mismatched-source.json" >/dev/null 2>&1; then
  echo 'expected mismatched source identity to fail' >&2
  exit 1
fi
jq -e '.failureCode == "source_work_item_id_mismatch" and .state == "failed"' "$fixture/mismatched-source.json" >/dev/null

cat > "$fixture/repo/.ai/work-items/active/WI-STANDALONE.contract.json" <<JSON
{
  "workItemId": "WI-STANDALONE",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "scope": ["README.md"]
}
JSON
standalone_output="$fixture/standalone-retry.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event workflow_dispatch \
  --head "$head" \
  --from-tag v0.2.90 \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --work-item-id WI-STANDALONE \
  --release-source-revision "$base" \
  --output "$standalone_output"
jq -e \
  '.mode == "release_recovery" and .workItemId == "WI-STANDALONE" and .recoveryLineage == "standalone_retry"' \
  "$standalone_output" >/dev/null

cat > "$fixture/repo/.ai/work-items/active/WI-PARTIAL.contract.json" <<JSON
{
  "workItemId": "WI-PARTIAL",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "predecessorWorkItemId": "WI-PREVIOUS",
  "scope": ["README.md"]
}
JSON
if "$resolver" \
  --repo "$fixture/repo" \
  --event workflow_dispatch \
  --head "$head" \
  --from-tag v0.2.90 \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --work-item-id WI-PARTIAL \
  --release-source-revision "$base" \
  --output "$fixture/partial-binding.json" >/dev/null 2>&1; then
  echo 'expected partial successor binding to fail' >&2
  exit 1
fi
jq -e '.failureCode == "recovery_binding_missing" and .state == "failed"' "$fixture/partial-binding.json" >/dev/null

if "$resolver" \
  --repo "$fixture/repo" \
  --event workflow_dispatch \
  --head "$head" \
  --from-tag v0.2.90 \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --output "$fixture/missing-id.json" >/dev/null 2>&1; then
  echo 'expected recovery without work_item_id to fail' >&2
  exit 1
fi
jq -e '.failureCode == "work_item_id_required" and .state == "failed"' "$fixture/missing-id.json" >/dev/null

printf 'work item resolution regression passed\n'
