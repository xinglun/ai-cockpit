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
if [[ "$runtime_bin" != /* ]]; then runtime_bin="$repository/$runtime_bin"; fi
if [[ "$contract" != /* ]]; then contract="$repository/$contract"; fi
if [[ "$artifact_dir" != /* ]]; then artifact_dir="$repository/$artifact_dir"; fi
if [[ ! -x "$runtime_bin" ]]; then
  printf 'candidate Runtime is not executable: %s\n' "$runtime_bin" >&2
  exit 2
fi
if [[ ! -f "$contract" || -L "$contract" ]]; then
  printf 'Work Item Contract is missing or not a regular file: %s\n' "$contract" >&2
  exit 2
fi
mkdir -p "$artifact_dir" || exit 2

status_before="$artifact_dir/hosted-runtime-status-before.json"
status_after="$artifact_dir/hosted-runtime-status-after.json"
preflight_output="$artifact_dir/hosted-runtime-preflight.json"
verification_output="$artifact_dir/hosted-runtime-execution.json"
formal_receipt_output="$artifact_dir/hosted-runtime-verification.json"
orchestration_output="$artifact_dir/hosted-runtime-orchestration.json"

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
  jq -n \
    --arg workItemId "$work_item_id" \
    --arg runtimeDigest "$runtime_digest" \
    --arg preflightState "$preflight_state" \
    --arg verificationState "$verification_state" \
    --arg failureReason "$failure_reason" \
    --argjson initialActions "$initial_actions" \
    --argjson refreshedActions "$refreshed_actions" \
    '{schemaVersion:1,workItemId:$workItemId,runtimeDigest:$runtimeDigest,initialActions:$initialActions,refreshedActions:$refreshedActions,preflightState:$preflightState,verificationState:$verificationState,failureReason:(if $failureReason == "" then null else $failureReason end)}' \
    >"$orchestration_output"
}

query_status() {
  local output=$1
  local error_output=$2
  "$runtime_bin" work-item status \
    --repo "$repository" \
    --id "$work_item_id" \
    --json >"$output" 2>"$error_output"
}

status_allows() {
  local status_path=$1
  local action=$2
  jq -e --arg action "$action" '(.safeActions // []) | index($action) != null' "$status_path" >/dev/null
}

if ! query_status "$status_before" "$artifact_dir/hosted-runtime-status-before.stderr"; then
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

if status_allows "$status_before" run_preflight; then
  preflight_state=admitted
  preflight_exit=0
  "$runtime_bin" preflight \
    --repo "$repository" \
    --contract "$contract" >"$preflight_output" 2>"$artifact_dir/hosted-runtime-preflight.stderr" || preflight_exit=$?
  if [[ "$preflight_exit" != 0 ]]; then
    preflight_state=failed
    failure_reason=preflight_failed
    query_status "$status_after" "$artifact_dir/hosted-runtime-status-after.stderr" || true
    refreshed_actions=$(jq -c '.safeActions // []' "$status_after" 2>/dev/null || printf '[]')
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

verification_exit=0
"$runtime_bin" verify \
  --repo "$repository" \
  --work-item "$work_item_id" \
  --workers 1 >"$verification_output" 2>"$artifact_dir/hosted-runtime-verification.stderr" || verification_exit=$?
if [[ "$verification_exit" != 0 ]]; then
  verification_state=failed
  failure_reason=verification_failed
else
  formal_evidence="$repository/.ai/evidence/$work_item_id.verification.json"
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
  elif ! cp "$formal_evidence" "$formal_receipt_output" || ! cmp -s "$formal_evidence" "$formal_receipt_output"; then
    verification_state=invalidated
    failure_reason=formal_verification_evidence_copy_failed
    verification_exit=1
  else
    verification_state=passed
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
exit "$verification_exit"
