#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
source "$root/tests/release/release_close_state.sh"

state=passed
reasons=()
record_release_close_failure reused_acceptance_run_invalid
finalize_release_close_state

receipt=$(jq -n \
  --arg state "$state" \
  --argjson failureRoots "$(printf '%s\n' "${reasons[@]}" | jq -R . | jq -s .)" \
  '{state:$state,failureRoots:$failureRoots,requiredReceipts:{publishHandoff:{jobState:"success"},versionConsistency:{jobState:"success"},publicInstall:{jobState:"success"},publicUpgrade:{jobState:"success"}}}')

jq -e '.state == "failed" and (.failureRoots | index("reused_acceptance_run_invalid")) != null and ([.requiredReceipts[] | .jobState] | all(. == "success"))' \
  <<<"$receipt" >/dev/null
if jq -e '.state == "passed"' <<<"$receipt" >/dev/null; then
  echo 'close-only recovery failure was incorrectly persisted as passed' >&2
  exit 1
fi

echo 'release close failure-state regression passed'
