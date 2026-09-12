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
  '.mode == "release_recovery" and .workItemId == "WI-TEST" and .releaseSourceRevision == $source and .headRevision == $head' \
  --arg source "$base" --arg head "$head" "$recovery_output" >/dev/null

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
