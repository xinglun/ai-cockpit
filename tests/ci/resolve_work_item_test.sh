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
  --event workflow_dispatch \
  --head "$head" \
  --to-tag v0.2.91 \
  --publish-existing-tag true \
  --work-item-id WI-NO-CONTRACT \
  --output "$empty_inventory_output" >/dev/null 2>&1
empty_inventory_result=$?
set -e
if [[ "$empty_inventory_result" -eq 0 ]]; then
  echo 'expected empty active Contract inventory to fail' >&2
  exit 1
fi
jq -e '.failureCode == "contract_not_regular" and .state == "failed"' \
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
"$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/wi-provisional \
  --pr-url https://github.com/example/repo/pull/11 \
  --output "$fixture/provisional.json"
jq -e '.state == "ready" and .selectionMethod == "ordinary_repository_route" and .workItemId == null' \
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

cat > "$fixture/repo/.ai/work-items/active/WI-AMBIGUOUS.contract.json" <<JSON
{
  "workItemId": "WI-AMBIGUOUS",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "resourceContext": {
    "branch": "codex/wi-ambiguous",
    "pullRequest": "https://github.com/example/repo/pull/14"
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
    "pullRequest": "https://github.com/example/repo/pull/14"
  },
  "scope": ["README.md"]
}
JSON
if "$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/wi-ambiguous \
  --pr-url https://github.com/example/repo/pull/14 \
  --output "$fixture/ambiguous.json" >/dev/null 2>&1; then
  echo 'expected ambiguous branch Work Item binding to fail' >&2
  exit 1
fi
jq -e '.failureCode == "work_item_contract_ambiguous" and (.message | contains("WI-AMBIGUOUS")) and (.message | contains("WI-AMBIGUOUS-ALT"))' \
  "$fixture/ambiguous.json" >/dev/null

"$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/wi-missing \
  --pr-url https://github.com/example/repo/pull/10 \
  --output "$fixture/unmatched.json"
jq -e '.state == "ready" and .selectionMethod == "ordinary_repository_route" and .workItemId == null' "$fixture/unmatched.json" >/dev/null

mkdir -p "$fixture/repo/.ai/work-items/archive"
cp "$fixture/repo/.ai/work-items/active/WI-TEST.contract.json" \
  "$fixture/repo/.ai/work-items/archive/WI-TEST.contract.json"
collision_contract_digest="sha256:$(shasum -a 256 "$fixture/repo/.ai/work-items/archive/WI-TEST.contract.json" | awk '{print $1}')"
cat > "$fixture/repo/.ai/work-items/archive/WI-TEST.archive.json" <<JSON
{
  "workItemId": "WI-TEST",
  "state": "archived",
  "files": {
    "contractPath": ".ai/work-items/archive/WI-TEST.contract.json",
    "contractDigest": "$collision_contract_digest"
  }
}
JSON
if "$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/test-route \
  --pr-url https://github.com/example/repo/pull/7 \
  --output "$fixture/collision.json" >/dev/null 2>&1; then
  echo 'expected active/archive Contract collision to fail closed' >&2
  exit 1
fi
jq -e '.failureCode == "work_item_contract_ambiguous" and .state == "failed"' \
  "$fixture/collision.json" >/dev/null

cat > "$fixture/repo/.ai/work-items/archive/WI-ARCHIVED.contract.json" <<JSON
{
  "workItemId": "WI-ARCHIVED",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "resourceContext": {
    "branch": "codex/archived-route",
    "pullRequest": "https://github.com/example/repo/pull/8"
  },
  "scope": ["README.md"]
}
JSON
archived_contract_digest="sha256:$(shasum -a 256 "$fixture/repo/.ai/work-items/archive/WI-ARCHIVED.contract.json" | awk '{print $1}')"
cat > "$fixture/repo/.ai/work-items/archive/WI-ARCHIVED.archive.json" <<JSON
{
  "workItemId": "WI-ARCHIVED",
  "state": "archived",
  "files": {
    "contractPath": ".ai/work-items/archive/WI-ARCHIVED.contract.json",
    "contractDigest": "$archived_contract_digest"
  }
}
JSON
archived_pull_request_output="$fixture/archived-pull-request.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/archived-route \
  --pr-url https://github.com/example/repo/pull/8 \
  --output "$archived_pull_request_output"
jq -e \
  '.state == "ready" and .mode == "pull_request" and .selectionMethod == "archived_contract_read_only" and .workItemId == "WI-ARCHIVED" and .contractPath == ".ai/work-items/archive/WI-ARCHIVED.contract.json" and .baseRevision == $base and .sourceWorkItemId == .workItemId and .sourceContractPath == .contractPath and .sourceContractDigest == .contractDigest and .sourceBaseRevision == .baseRevision and .sourceSelectionMethod == "same_as_governance"' \
  --arg base "$base" "$archived_pull_request_output" >/dev/null

archived_dual_identity_output="$fixture/archived-dual-identity.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/archived-route \
  --pr-url https://github.com/example/repo/pull/8 \
  --source-work-item-id WI-SOURCE \
  --output "$archived_dual_identity_output"
jq -e \
  '.workItemId == "WI-ARCHIVED" and .selectionMethod == "archived_contract_read_only" and .sourceWorkItemId == "WI-SOURCE" and .sourceContractPath == ".ai/work-items/active/WI-SOURCE.contract.json" and .sourceSelectionMethod == "explicit_source_work_item_id" and .sourceContractDigest != .contractDigest' \
  "$archived_dual_identity_output" >/dev/null

cat > "$fixture/repo/.ai/work-items/archive/WI-ARCHIVED-NO-RESOURCE.contract.json" <<JSON
{
  "workItemId": "WI-ARCHIVED-NO-RESOURCE",
  "state": "implementation_active",
  "repositoryId": "fixture-repository",
  "baseRevision": "$base",
  "scope": ["README.md"]
}
JSON
no_resource_contract_digest="sha256:$(shasum -a 256 "$fixture/repo/.ai/work-items/archive/WI-ARCHIVED-NO-RESOURCE.contract.json" | awk '{print $1}')"
cat > "$fixture/repo/.ai/work-items/archive/WI-ARCHIVED-NO-RESOURCE.archive.json" <<JSON
{
  "workItemId": "WI-ARCHIVED-NO-RESOURCE",
  "state": "archived",
  "files": {
    "contractPath": ".ai/work-items/archive/WI-ARCHIVED-NO-RESOURCE.contract.json",
    "contractDigest": "$no_resource_contract_digest"
  }
}
JSON
no_resource_pull_request_output="$fixture/archived-no-resource-pull-request.json"
"$resolver" \
  --repo "$fixture/repo" \
  --event pull_request \
  --head "$head" \
  --pr-head-ref codex/archived-no-resource \
  --pr-url https://github.com/example/repo/pull/9 \
  --output "$no_resource_pull_request_output"
jq -e \
  '.state == "ready" and .mode == "pull_request" and .selectionMethod == "ordinary_repository_route" and .contractPath == null and .workItemId == null' \
  "$no_resource_pull_request_output" >/dev/null

mkdir -p "$fixture/fake-bin"
cat > "$fixture/fake-bin/gh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == api ]]; then
  exit 0
fi
echo "unexpected fake gh invocation" >&2
exit 1
SH
chmod +x "$fixture/fake-bin/gh"
ordinary_push_output="$fixture/ordinary-push.json"
PATH="$fixture/fake-bin:$PATH" GH_TOKEN=test-token GITHUB_REPOSITORY=example/repo GITHUB_REF=refs/heads/codex/wi-push-no-resource GITHUB_REF_NAME=codex/wi-push-no-resource \
  "$resolver" \
    --repo "$fixture/repo" \
    --event push \
    --head "$head" \
    --output "$ordinary_push_output"
jq -e \
  '.state == "ready" and .mode == "merge" and .selectionMethod == "ordinary_repository_route" and .contractPath == null and .workItemId == null and .baseRevision == null and .releaseSourceRevision == null' \
  "$ordinary_push_output" >/dev/null

cat > "$fixture/fake-bin/gh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == api ]]; then
  printf 'https://github.com/example/repo/pull/7\tcodex/test-route\n'
  exit 0
fi
echo "unexpected fake gh invocation" >&2
exit 1
SH
chmod +x "$fixture/fake-bin/gh"
tag_push_output="$fixture/tag-push.json"
PATH="$fixture/fake-bin:$PATH" GH_TOKEN=test-token GITHUB_REPOSITORY=example/repo GITHUB_REF=refs/tags/v0.2.91 GITHUB_REF_NAME=v0.2.91 \
  "$resolver" \
    --repo "$fixture/repo" \
    --event push \
    --head "$head" \
    --release-source-revision "$base" \
    --output "$tag_push_output"
jq -e \
  '.state == "ready" and .mode == "tag_release" and .selectionMethod == "merged_pull_request_binding" and .workItemId == "WI-TEST" and .releaseSourceRevision == $source' \
  --arg source "$base" "$tag_push_output" >/dev/null

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
