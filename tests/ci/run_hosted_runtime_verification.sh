#!/usr/bin/env bash
set -uo pipefail

if [[ $# -ne 4 ]]; then
  printf 'usage: %s <runtime-bin> <repository> <contract> <artifact-dir>\n' "$0" >&2
  exit 2
fi

runtime_bin=$1
repository=$2
contract=$3
artifact_dir=$4
repository=$(cd "$repository" && pwd -P) || exit 2
source_repository=$repository
execution_repository=$source_repository
if [[ "$runtime_bin" != /* ]]; then runtime_bin="$source_repository/$runtime_bin"; fi
if [[ "$contract" != /* ]]; then contract="$source_repository/$contract"; fi
if [[ "$artifact_dir" != /* ]]; then artifact_dir="$source_repository/$artifact_dir"; fi
contract_directory=$(cd "$(dirname "$contract")" && pwd -P) || exit 2
contract="$contract_directory/$(basename "$contract")"
if [[ ! -x "$runtime_bin" ]]; then
  printf 'candidate Runtime is not executable: %s\n' "$runtime_bin" >&2
  exit 2
fi
if [[ ! -f "$contract" || -L "$contract" ]]; then
  printf 'Work Item Contract is missing or not a regular file: %s\n' "$contract" >&2
  exit 2
fi
case "$contract" in
  "$source_repository"/*) contract_relative=${contract#"$source_repository"/} ;;
  *)
    printf 'Work Item Contract must be inside the source repository: %s\n' "$contract" >&2
    exit 2
    ;;
esac
mkdir -p "$artifact_dir" || exit 2

status_before="$artifact_dir/hosted-runtime-status-before.json"
status_after="$artifact_dir/hosted-runtime-status-after.json"
status_verified="$artifact_dir/hosted-runtime-status-verified.json"
validation_before="$artifact_dir/hosted-runtime-validation-before.json"
validation_after="$artifact_dir/hosted-runtime-validation-after.json"
preflight_output="$artifact_dir/hosted-runtime-preflight.json"
verification_output="$artifact_dir/hosted-runtime-execution.json"
formal_receipt_output="$artifact_dir/hosted-runtime-verification.json"
orchestration_output="$artifact_dir/hosted-runtime-orchestration.json"
cleanup_output="$artifact_dir/hosted-runtime-worktree-cleanup.json"

worktree_parent=""
worktree_path=""
execution_isolation_state=not_needed_fresh_receipt

work_item_id=$(jq -er '.workItemId | strings | select(length > 0)' "$contract") || {
  printf 'Contract does not declare a Work Item ID\n' >&2
  exit 2
}
runtime_digest=$(python3 - "$runtime_bin" <<'PY'
import hashlib
import sys
from pathlib import Path

print("sha256:" + hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
) || exit 2
: >"$formal_receipt_output" || exit 2

preflight_state=not_started
verification_state=not_started
failure_reason=""
initial_actions='[]'
refreshed_actions='[]'

write_orchestration_report() {
  local formal_receipt_digest=""
  if [[ -s "$formal_receipt_output" ]]; then
    formal_receipt_digest=$(python3 - "$formal_receipt_output" <<'PY'
import hashlib
import sys
from pathlib import Path

print("sha256:" + hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
) || return 1
  fi
  jq -n \
    --arg workItemId "$work_item_id" \
    --arg runtimeDigest "$runtime_digest" \
    --arg executionRepository "$execution_repository" \
    --arg formalReceiptDigest "$formal_receipt_digest" \
    --arg preflightState "$preflight_state" \
    --arg verificationState "$verification_state" \
    --arg failureReason "$failure_reason" \
    --argjson initialActions "$initial_actions" \
    --argjson refreshedActions "$refreshed_actions" \
    '{schemaVersion:1,workItemId:$workItemId,runtimeDigest:$runtimeDigest,executionRepository:$executionRepository,formalReceiptDigest:(if $formalReceiptDigest == "" then null else $formalReceiptDigest end),initialActions:$initialActions,refreshedActions:$refreshedActions,preflightState:$preflightState,verificationState:$verificationState,failureReason:(if $failureReason == "" then null else $failureReason end)}' \
    >"$orchestration_output"
}

cleanup_execution_worktree() {
  local command_status=$?
  local cleanup_status=0
  local cleanup_state=removed
  local defer_cleanup=false
  trap - EXIT

  if [[ "${AI_COCKPIT_DEFER_WORKTREE_CLEANUP:-false}" == true \
    && "$command_status" == 0 \
    && "$verification_state" == passed \
    && -n "$worktree_path" ]]; then
    defer_cleanup=true
    cleanup_state=deferred_for_consumer
  elif [[ -n "$worktree_path" ]]; then
    if git -C "$source_repository" worktree list --porcelain | awk -v path="$worktree_path" '$1 == "worktree" && substr($0, 10) == path { found = 1 } END { exit !found }'; then
      git -C "$source_repository" worktree remove --force "$worktree_path" >/dev/null || cleanup_status=$?
    elif [[ -d "$worktree_path" ]]; then
      rmdir "$worktree_path" || cleanup_status=$?
    fi
  fi
  if [[ "$defer_cleanup" != true && -n "$worktree_parent" && -d "$worktree_parent" ]]; then
    rmdir "$worktree_parent" || cleanup_status=$?
  fi

  [[ -z "$worktree_path" ]] && cleanup_state=$execution_isolation_state
  [[ "$cleanup_status" == 0 ]] || cleanup_state=failed
  jq -n \
    --arg state "$cleanup_state" \
    --arg sourceRepository "$source_repository" \
    --arg isolatedRepository "$worktree_path" \
    --argjson cleanupExitCode "$cleanup_status" \
    '{schemaVersion:1,state:$state,sourceRepository:$sourceRepository,isolatedRepository:(if $isolatedRepository == "" then null else $isolatedRepository end),cleanupExitCode:$cleanupExitCode}' \
    >"$cleanup_output" || cleanup_status=$?

  if [[ "$defer_cleanup" == true ]]; then
    exit "$command_status"
  fi

  if [[ "$cleanup_status" != 0 ]]; then
    printf 'failed to clean hosted Runtime worktree (exit %s): %s\n' "$cleanup_status" "$worktree_path" >&2
    exit 1
  fi
  exit "$command_status"
}
trap cleanup_execution_worktree EXIT

prepare_isolated_execution() {
  local source_head
  local untracked
  execution_isolation_state=preparing
  if ! git -C "$source_repository" diff --quiet HEAD --; then
    execution_isolation_state=failed_precondition
    printf 'source checkout has tracked changes; refusing to verify a different committed snapshot\n' >&2
    return 1
  fi
  untracked=$(git -C "$source_repository" ls-files --others --exclude-standard) || return 1
  if [[ -n "$untracked" ]]; then
    execution_isolation_state=failed_precondition
    printf 'source checkout has untracked files; refusing to verify a different committed snapshot:\n%s\n' "$untracked" >&2
    return 1
  fi

  source_head=$(git -C "$source_repository" rev-parse --verify 'HEAD^{commit}') || {
    execution_isolation_state=failed_to_resolve_source_head
    return 1
  }
  worktree_parent=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/ai-cockpit-hosted-runtime.XXXXXX") || {
    execution_isolation_state=failed_to_create_parent
    return 1
  }
  worktree_parent=$(cd "$worktree_parent" && pwd -P) || {
    execution_isolation_state=failed_to_resolve_parent
    return 1
  }
  worktree_path="$worktree_parent/source"
  if ! git -C "$source_repository" worktree add --detach "$worktree_path" "$source_head" >"$artifact_dir/hosted-runtime-worktree-create.log" 2>&1; then
    execution_isolation_state=failed_to_create
    cat "$artifact_dir/hosted-runtime-worktree-create.log" >&2
    return 1
  fi

  execution_repository=$worktree_path
  contract="$execution_repository/$contract_relative"
  execution_isolation_state=created
  if ! query_status "$status_before" "$artifact_dir/hosted-runtime-status-before.stderr"; then
    failure_reason=isolated_initial_status_query_failed
    printf 'candidate Runtime status query failed in the isolated worktree\n' >&2
    return 1
  fi
  if ! jq -e --arg id "$work_item_id" '.workItemId == $id and (.safeActions | type == "array")' "$status_before" >/dev/null; then
    failure_reason=isolated_initial_status_identity_or_shape_invalid
    printf 'candidate Runtime returned an invalid isolated Work Item status projection\n' >&2
    return 1
  fi
  initial_actions=$(jq -c '.safeActions' "$status_before") || return 1
}

query_status() {
  local output=$1
  local error_output=$2
  "$runtime_bin" work-item status \
    --repo "$execution_repository" \
    --id "$work_item_id" \
    --json >"$output" 2>"$error_output"
}

query_validate() {
  local output=$1
  local error_output=$2
  "$runtime_bin" work-item validate \
    --repo "$execution_repository" \
    --id "$work_item_id" \
    --json >"$output" 2>"$error_output"
}

validation_report_is_valid() {
  local report_path=$1
  jq -e 'type == "object"
    and (.state | type == "string")
    and (.unknowns | type == "array")
    and (.findings | type == "array")' "$report_path" >/dev/null
}

status_allows() {
  local status_path=$1
  local action=$2
  jq -e --arg action "$action" '(.safeActions // []) | index($action) != null' "$status_path" >/dev/null
}

if ! query_status "$status_before" "$artifact_dir/hosted-runtime-status-source.stderr"; then
  failure_reason=initial_status_query_failed
  write_orchestration_report
  printf 'candidate Runtime status query failed; see %s\n' "$artifact_dir/hosted-runtime-status-before.stderr" >&2
  exit 1
fi
if ! jq -e --arg id "$work_item_id" '.workItemId == $id and (.safeActions | type == "array")' "$status_before" >/dev/null; then
  failure_reason=initial_status_identity_or_shape_invalid
  write_orchestration_report
  printf 'candidate Runtime returned an invalid Work Item status projection\n' >&2
  exit 1
fi
initial_actions=$(jq -c '.safeActions' "$status_before") || exit 1
if ! query_validate "$validation_before" "$artifact_dir/hosted-runtime-validation-before.stderr"; then
  verification_state=invalidated
  failure_reason=initial_validation_query_failed
  write_orchestration_report
  printf 'candidate Runtime Work Item validation query failed; see %s\n' \
    "$artifact_dir/hosted-runtime-validation-before.stderr" >&2
  exit 1
fi
if ! validation_report_is_valid "$validation_before"; then
  verification_state=invalidated
  failure_reason=validation_report_invalid
  write_orchestration_report
  printf 'candidate Runtime returned an invalid Work Item validation report\n' >&2
  exit 1
fi

verification_exit=0
if jq -e '.verification == "verified" and .evidenceFreshness.state == "fresh"' "$status_before" >/dev/null; then
  # A complete fresh Runtime receipt is already the required formal result.
  # Reuse its exact bytes; do not rerun preflight or verification merely to
  # recreate evidence for the same Work Item snapshot.
  preflight_state=not_required_existing_fresh_receipt
  refreshed_actions=$initial_actions
  formal_evidence="$repository/.ai/evidence/$work_item_id.verification.json"
  if [[ -L "$repository/.ai" || -L "$repository/.ai/evidence" || ! -f "$formal_evidence" || -L "$formal_evidence" ]]; then
    verification_state=invalidated
    failure_reason=existing_fresh_receipt_missing_or_unsafe
    verification_exit=1
  elif ! jq -e \
    --arg id "$work_item_id" \
    --arg digest "$runtime_digest" \
    --argjson status "$(<"$status_before")" \
    '.workItemId == $id
      and .passed == true
      and .runtimeDigest == $digest
      and (.runtimeVersion | type == "string" and length > 0)
      and (.contractDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
      and (.repositoryId | type == "string" and test("^sha256:[0-9a-f]{64}$"))
      and (.repositorySnapshotDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
      and .workItemId == $status.workItemId
      and .repositoryId == $status.repositoryId
      and .contractDigest == $status.sourceDigests.contract
      and .repositorySnapshotDigest == $status.sourceDigests.repositorySnapshot
      and (.receipt | type == "object")
      and .receipt.workItemId == $id
      and .receipt.passed == true
      and .receipt.runtimeDigest == $digest
      and .receipt.runtimeVersion == .runtimeVersion
      and .receipt.repositoryId == .repositoryId
      and (.receipt.results | type == "array" and length > 0)
      and .receipt.nodesPlanned == (.receipt.results | length)
      and .receipt.nodesExecuted + .receipt.nodesReused == .receipt.nodesPlanned
      and all(.receipt.results[]; .passed == true and (.nodeId | type == "string" and length > 0))
      and ([.receipt.results[].nodeId] | unique | length) == (.receipt.results | length)
      and (.receipt.planReceipt | type == "object")
      and .receipt.planReceipt.workItemId == $id
      and .receipt.planReceipt.repositoryId == .repositoryId
      and .receipt.planReceipt.baseRevision == $status.baseCommit
      and .receipt.planReceipt.stage == "task"
      and .receipt.planReceipt.repositorySnapshotDigest == .repositorySnapshotDigest
      and (.receipt.planReceipt.executedNodes | type == "array")
      and (.receipt.planReceipt.reusedNodes | type == "array")
      and (([.receipt.planReceipt.executedNodes[], .receipt.planReceipt.reusedNodes[]] | sort) == ([.receipt.results[].nodeId] | sort))
      and (.receipt.planReceipt.coverageManifest.nodeIds | type == "array" and length > 0)
      and ([.receipt.results[].nodeId] as $result_ids | all(.receipt.planReceipt.coverageManifest.nodeIds[]; . as $node | ($result_ids | index($node)) != null))' \
    "$formal_evidence" >/dev/null; then
    verification_state=invalidated
    failure_reason=existing_fresh_receipt_identity_mismatch
    verification_exit=1
  elif ! cp "$formal_evidence" "$formal_receipt_output" || ! cmp -s "$formal_evidence" "$formal_receipt_output"; then
    verification_state=invalidated
    failure_reason=formal_verification_evidence_copy_failed
    verification_exit=1
  else
    verification_state=reused
  fi
else
  if ! prepare_isolated_execution; then
    [[ -n "$failure_reason" ]] || failure_reason=isolated_worktree_preparation_failed
    verification_state=failed
    write_orchestration_report
    exit 1
  fi
  if status_allows "$status_before" run_preflight; then
    preflight_state=admitted
    preflight_exit=0
    "$runtime_bin" preflight \
      --repo "$execution_repository" \
      --contract "$contract" >"$preflight_output" 2>"$artifact_dir/hosted-runtime-preflight.stderr" || preflight_exit=$?
    if [[ "$preflight_exit" != 0 ]]; then
      preflight_state=failed
      failure_reason=preflight_failed
      if query_status "$status_after" "$artifact_dir/hosted-runtime-status-after.stderr"; then
        refreshed_actions=$(jq -c '.safeActions // []' "$status_after" 2>/dev/null) || refreshed_actions='[]'
      else
        refreshed_actions='[]'
      fi
      write_orchestration_report
      printf 'candidate Runtime preflight failed with exit %s\n' "$preflight_exit" >&2
      exit "$preflight_exit"
    fi
    preflight_state=passed
  elif status_allows "$status_before" run_verification; then
    preflight_state=not_admitted_and_not_required
  else
    failure_reason=no_admitted_preflight_or_verification_action
    write_orchestration_report
    printf 'candidate Runtime admits neither run_preflight nor run_verification\n' >&2
    exit 1
  fi

  # Re-read admission after the optional preflight and immediately before
  # verification. A preflight receipt or an earlier status snapshot is not
  # authority for the following action.
  if ! query_status "$status_after" "$artifact_dir/hosted-runtime-status-after.stderr"; then
    failure_reason=refreshed_status_query_failed
    write_orchestration_report
    printf 'candidate Runtime status refresh failed; see %s\n' "$artifact_dir/hosted-runtime-status-after.stderr" >&2
    exit 1
  fi
  if ! jq -e --arg id "$work_item_id" '.workItemId == $id and (.safeActions | type == "array")' "$status_after" >/dev/null; then
    failure_reason=refreshed_status_identity_or_shape_invalid
    write_orchestration_report
    printf 'candidate Runtime returned an invalid refreshed Work Item status projection\n' >&2
    exit 1
  fi
  refreshed_actions=$(jq -c '.safeActions' "$status_after") || exit 1
  if ! status_allows "$status_after" run_verification; then
    failure_reason=run_verification_not_admitted_after_refresh
    write_orchestration_report
    printf 'candidate Runtime no longer admits run_verification after status refresh\n' >&2
    exit 1
  fi
  if ! query_validate "$validation_after" "$artifact_dir/hosted-runtime-validation-after.stderr"; then
    verification_state=invalidated
    failure_reason=validation_query_failed
    write_orchestration_report
    printf 'candidate Runtime Work Item validation refresh failed; see %s\n' \
      "$artifact_dir/hosted-runtime-validation-after.stderr" >&2
    exit 1
  fi
  if ! validation_report_is_valid "$validation_after"; then
    verification_state=invalidated
    failure_reason=validation_report_invalid
    write_orchestration_report
    printf 'candidate Runtime returned an invalid refreshed Work Item validation report\n' >&2
    exit 1
  fi

  "$runtime_bin" verify \
    --repo "$execution_repository" \
    --work-item "$work_item_id" \
    --workers 1 >"$verification_output" 2>"$artifact_dir/hosted-runtime-verification.stderr" || verification_exit=$?
  if [[ "$verification_exit" != 0 ]]; then
    verification_state=failed
    failure_reason=verification_failed
  else
    formal_evidence="$execution_repository/.ai/evidence/$work_item_id.verification.json"
    if [[ -L "$repository/.ai" || -L "$repository/.ai/evidence" || ! -f "$formal_evidence" || -L "$formal_evidence" ]]; then
      verification_state=invalidated
      failure_reason=formal_verification_evidence_missing_or_unsafe
      verification_exit=1
    elif ! jq -e \
      --arg id "$work_item_id" \
      --arg digest "$runtime_digest" \
      '.workItemId == $id
        and .passed == true
        and .runtimeDigest == $digest
        and (.runtimeVersion | type == "string" and length > 0)
        and (.contractDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
        and (.repositoryId | type == "string" and test("^sha256:[0-9a-f]{64}$"))
        and (.repositorySnapshotDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
        and (.receipt | type == "object")
        and .receipt.workItemId == $id
        and .receipt.passed == true
        and .receipt.runtimeDigest == $digest
        and .receipt.runtimeVersion == .runtimeVersion
        and .receipt.repositoryId == .repositoryId
        and .receipt.planReceipt.repositorySnapshotDigest == .repositorySnapshotDigest' \
      "$formal_evidence" >/dev/null; then
      verification_state=invalidated
      failure_reason=formal_verification_evidence_identity_mismatch
      verification_exit=1
    elif ! query_status "$status_verified" "$artifact_dir/hosted-runtime-status-verified.stderr"; then
      verification_state=invalidated
      failure_reason=post_verification_status_query_failed
      verification_exit=1
    elif ! jq -e \
      --arg id "$work_item_id" \
      --arg contract_digest "$(jq -r '.contractDigest' "$formal_evidence")" \
      --arg snapshot_digest "$(jq -r '.repositorySnapshotDigest' "$formal_evidence")" \
      '.workItemId == $id
        and .verification == "verified"
        and .evidenceFreshness.state == "fresh"
        and .sourceDigests.contract == $contract_digest
        and .sourceDigests.repositorySnapshot == $snapshot_digest' \
      "$status_verified" >/dev/null; then
      verification_state=invalidated
      failure_reason=post_verification_status_mismatch
      verification_exit=1
    elif ! cp "$formal_evidence" "$formal_receipt_output" || ! cmp -s "$formal_evidence" "$formal_receipt_output"; then
      verification_state=invalidated
      failure_reason=formal_verification_evidence_copy_failed
      verification_exit=1
    else
      verification_state=passed
    fi
  fi
fi

final_runtime_digest=$(python3 - "$runtime_bin" <<'PY'
import hashlib
import sys
from pathlib import Path

print("sha256:" + hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
) || exit 2
if [[ "$final_runtime_digest" != "$runtime_digest" ]]; then
  verification_state=invalidated
  failure_reason=candidate_runtime_changed_during_verification
  verification_exit=1
fi

write_orchestration_report
if [[ "$failure_reason" == post_verification_status_mismatch ]]; then
  printf 'candidate Runtime does not accept the formal receipt as fresh for the current Contract and source snapshot\n' >&2
fi
if [[ "$failure_reason" == existing_fresh_receipt_* || "$failure_reason" == formal_verification_evidence_copy_failed ]]; then
  printf 'candidate Runtime reports fresh verification, but its formal receipt is not reusable (%s)\n' "$failure_reason" >&2
fi
exit "$verification_exit"
