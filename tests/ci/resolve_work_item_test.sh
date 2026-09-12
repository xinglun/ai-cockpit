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

empty_inventory_output="$fixture/empty-inventory.json"
set +e
"$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/no-contract \
  --pr-url https://github.com/example/repo/pull/6 \
  --output "$empty_inventory_output" >/dev/null 2>&1
empty_inventory_result=$?
set -e
if [[ "$empty_inventory_result" -eq 0 ]]; then
  echo 'expected empty active Contract inventory to fail' >&2
  exit 1
fi
jq -e '.failureCode == "work_item_contract_unresolved" and .state == "failed"' \
  "$empty_inventory_output" >/dev/null

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

cat > "$fixture/repo/.ai/work-items/active/WI-NO-RESOURCE.contract.json" <<JSON
{
  "workItemId": "WI-NO-RESOURCE",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "scope": ["README.md"]
}
JSON
no_resource_output="$fixture/no-resource-pull-request.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/wi-no-resource \
  --pr-url https://github.com/example/repo/pull/8 \
  --output "$no_resource_output"
jq -e \
  '.mode == "pull_request" and .workItemId == "WI-NO-RESOURCE" and .selectionMethod == "pull_request_branch_work_item" and .contractPath == ".ai/work-items/active/WI-NO-RESOURCE.contract.json"' \
  "$no_resource_output" >/dev/null

cat > "$fixture/repo/.ai/work-items/active/WI-PROVISIONAL.contract.json" <<JSON
{
  "workItemId": "WI-PROVISIONAL",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "resourceContext": {
    "branch": "codex/wi-provisional",
    "pullRequest": "pending"
  },
  "scope": ["README.md"]
}
JSON
if "$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/wi-provisional \
  --pr-url https://github.com/example/repo/pull/11 \
  --output "$fixture/provisional.json" >/dev/null 2>&1; then
  echo 'expected provisional resource Contract to require exact PR binding' >&2
  exit 1
fi
jq -e '.failureCode == "work_item_contract_unresolved" and .state == "failed"' \
  "$fixture/provisional.json" >/dev/null

cat > "$fixture/repo/.ai/work-items/active/WI-SHADOW.contract.json" <<JSON
{
  "workItemId": "WI-SHADOW",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "scope": ["README.md"]
}
JSON
cat > "$fixture/repo/.ai/work-items/active/WI-SHADOW-BOUND.contract.json" <<JSON
{
  "workItemId": "WI-SHADOW-BOUND",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "resourceContext": {
    "branch": "codex/wi-shadow",
    "pullRequest": "https://github.com/example/repo/pull/12"
  },
  "scope": ["README.md"]
}
JSON
if "$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/wi-shadow \
  --pr-url https://github.com/example/repo/pull/13 \
  --output "$fixture/shadow-conflict.json" >/dev/null 2>&1; then
  echo 'expected same-branch resource Contract conflict to fail closed' >&2
  exit 1
fi
jq -e '.failureCode == "work_item_branch_conflict" and .state == "failed"' \
  "$fixture/shadow-conflict.json" >/dev/null

mkdir -p "$fixture/bin"
cat > "$fixture/bin/gh" <<'SH'
#!/usr/bin/env bash
exit 0
SH
chmod +x "$fixture/bin/gh"
cat > "$fixture/repo/.ai/work-items/active/WI-PUSH-NO-RESOURCE.contract.json" <<JSON
{
  "workItemId": "WI-PUSH-NO-RESOURCE",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "scope": ["README.md"]
}
JSON
push_output="$fixture/ordinary-push.json"
set +e
GITHUB_REF=refs/heads/codex/wi-push-no-resource \
GITHUB_REPOSITORY=fixture/repo \
GH_TOKEN=test-token \
PATH="$fixture/bin:$PATH" \
  "$resolver" \
    --repo "$fixture/repo" \
    --event push \
    --head "$head" \
    --output "$push_output"
push_result=$?
set -e
if [[ "$push_result" -eq 0 ]]; then
  jq -e \
    '.mode == "merge" and .workItemId == null and .selectionMethod == "ordinary_repository_route"' \
    "$push_output" >/dev/null
else
  jq -e \
    '.failureCode == "work_item_contract_unresolved" and .state == "failed"' \
    "$push_output" >/dev/null
fi

cat > "$fixture/repo/.ai/work-items/active/WI-AMBIGUOUS.contract.json" <<JSON
{
  "workItemId": "WI-AMBIGUOUS",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "resourceContext": {
    "branch": "codex/wi-ambiguous",
    "pullRequest": "https://github.com/example/repo/pull/9"
  },
  "scope": ["README.md"]
}
JSON
cat > "$fixture/repo/.ai/work-items/active/WI-AMBIGUOUS-ALT.contract.json" <<JSON
{
  "workItemId": "WI-AMBIGUOUS-ALT",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "resourceContext": {
    "branch": "codex/wi-ambiguous",
    "pullRequest": "https://github.com/example/repo/pull/9"
  },
  "scope": ["README.md"]
}
JSON
if "$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/wi-ambiguous \
  --pr-url https://github.com/example/repo/pull/9 \
  --output "$fixture/ambiguous.json" >/dev/null 2>&1; then
  echo 'expected ambiguous branch Work Item binding to fail' >&2
  exit 1
fi
jq -e '.failureCode == "work_item_contract_ambiguous" and (.message | contains("WI-AMBIGUOUS")) and (.message | contains("WI-AMBIGUOUS-ALT"))' "$fixture/ambiguous.json" >/dev/null

if "$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/wi-missing \
  --pr-url https://github.com/example/repo/pull/10 \
  --output "$fixture/unmatched.json" >/dev/null 2>&1; then
  echo 'expected unmatched branch Work Item binding to fail' >&2
  exit 1
fi
jq -e '.failureCode == "work_item_contract_unresolved" and .state == "failed"' "$fixture/unmatched.json" >/dev/null

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
