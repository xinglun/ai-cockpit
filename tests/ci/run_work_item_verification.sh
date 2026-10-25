#!/usr/bin/env bash
# Explicit managed entry; the selected Runtime remains the admission authority.
set -euo pipefail

runtime=""
repository=""
work_item=""
task_caller=""
plan_only=false
declare -a verify_options=()
declare -A seen=()
while [[ $# -gt 0 ]]; do
  flag=$1
  if [[ -n ${seen[$flag]:-} ]]; then
    printf 'duplicate managed verification argument: %s\n' "$flag" >&2; exit 2
  fi
  seen[$flag]=true
  case "$flag" in
    --runtime|--repo|--work-item|--task-caller|--workers|--stage|--timeout-seconds|--base-revision)
      if [[ $# -lt 2 || -z "$2" || "$2" == --* ]]; then
        printf 'missing value for %s\n' "$flag" >&2; exit 2
      fi
      case "$flag" in
        --runtime) runtime=$2 ;;
        --repo) repository=$2 ;;
        --work-item) work_item=$2 ;;
        --task-caller) task_caller=$2 ;;
        *) verify_options+=("$flag" "$2") ;;
      esac
      shift 2 ;;
    --plan-only) plan_only=true; verify_options+=("$flag"); shift ;;
    *) printf 'unsupported managed verification argument: %s\n' "$flag" >&2; exit 2 ;;
  esac
done
if [[ -z "$runtime" || -z "$repository" || -z "$work_item" ]]; then
  printf 'usage: %s --runtime <approved-binary> --repo <source-root> --work-item <id> [verification options]\n' "$0" >&2
  exit 2
fi
if [[ ! -x "$runtime" ]]; then
  printf 'selected Runtime is not executable: %s\n' "$runtime" >&2; exit 2
fi
if [[ ! "$work_item" =~ ^[A-Za-z0-9_-]+$ ]]; then
  printf 'unsafe Work Item ID\n' >&2; exit 2
fi
repository=$(cd "$repository" && pwd -P)
declare -a caller_options=()
if [[ -n "$task_caller" ]]; then caller_options+=(--task-caller "$task_caller"); fi
if [[ "$plan_only" == true ]]; then
  exec "$runtime" verify --repo "$repository" --work-item "$work_item" "${caller_options[@]}" "${verify_options[@]}"
fi

validation=$("$runtime" work-item validate --repo "$repository" --id "$work_item" "${caller_options[@]}" --json)
if ! jq -e 'type == "object" and (.findings | type == "array") and
    all(.findings[]; .severity != "error")' <<<"$validation" >/dev/null; then
  printf '%s\n' "$validation" >&2
  printf 'canonical planning validation has errors or an invalid report\n' >&2
  exit 1
fi
read_actions() {
  "$runtime" work-item status --repo "$repository" --id "$work_item" "${caller_options[@]}" --json
}
status=$(read_actions)
has_action() {
  jq -e --arg id "$work_item" --arg action "$1" '
    type == "object" and .workItemId == $id and (.safeActions | type == "array") and
    (.safeActions | index($action) != null)' <<<"$status" >/dev/null
}
if ! has_action run_verification && has_action run_preflight; then
  contract="$repository/.ai/work-items/active/$work_item.contract.json"
  if [[ ! -f "$contract" || -L "$contract" ]]; then
    printf 'active Contract is missing or unsafe: %s\n' "$contract" >&2; exit 1
  fi
  "$runtime" preflight --repo "$repository" --contract "$contract" "${caller_options[@]}"
  status=$(read_actions)
fi
if ! has_action run_verification; then
  printf '%s\n' "$status" >&2
  printf 'Runtime does not admit Work Item verification\n' >&2
  exit 1
fi
bash "$(dirname "$0")/run_repository_prechecks.sh" --repo "$repository"
exec "$runtime" verify --repo "$repository" --work-item "$work_item" "${caller_options[@]}" "${verify_options[@]}"
