#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: resolve_release_plan.sh --repo ROOT --event workflow_dispatch --head SHA
  --from-tag TAG --to-tag TAG --output FILE [--plan-bin PATH]
  [--publish-existing-tag true|false] [--publish-candidate true|false]
  [--post-release-acceptance true|false]
  [--close-only true|false] [--work-item-id ID] [--contract-path PATH]
  [--source-work-item-id ID] [--source-contract-path PATH]
  [--handoff-run-id ID] [--reuse-run-id ID] [--reuse-acceptance-run-id ID]
  [--recovery-evidence PATH] [--release-source-revision SHA]
EOF
  exit 64
}

repo=''
event=''
head=''
from_tag=''
to_tag=''
output=''
plan_bin=''
publish_existing_tag=false
publish_candidate=false
post_release_acceptance=false
close_only=false
work_item_id=''
source_work_item_id=''
contract_path_arg=''
source_contract_path_arg=''
handoff_run_id=''
reuse_run_id=''
reuse_acceptance_run_id=''
release_source_revision=''
recovery_evidence_paths=()

while (($# > 0)); do
  case "$1" in
    --repo) repo=${2:?missing value for --repo}; shift 2 ;;
    --event) event=${2:?missing value for --event}; shift 2 ;;
    --head) head=${2:?missing value for --head}; shift 2 ;;
    --from-tag) from_tag=${2:?missing value for --from-tag}; shift 2 ;;
    --to-tag) to_tag=${2:?missing value for --to-tag}; shift 2 ;;
    --output) output=${2:?missing value for --output}; shift 2 ;;
    --plan-bin) plan_bin=${2:?missing value for --plan-bin}; shift 2 ;;
    --publish-existing-tag) publish_existing_tag=${2:?missing value for --publish-existing-tag}; shift 2 ;;
    --publish-candidate) publish_candidate=${2:?missing value for --publish-candidate}; shift 2 ;;
    --post-release-acceptance) post_release_acceptance=${2:?missing value for --post-release-acceptance}; shift 2 ;;
    --close-only) close_only=${2:?missing value for --close-only}; shift 2 ;;
    --work-item-id) work_item_id=${2:?missing value for --work-item-id}; shift 2 ;;
    --source-work-item-id) source_work_item_id=${2:?missing value for --source-work-item-id}; shift 2 ;;
    --contract-path) contract_path_arg=${2:?missing value for --contract-path}; shift 2 ;;
    --source-contract-path) source_contract_path_arg=${2:?missing value for --source-contract-path}; shift 2 ;;
    --handoff-run-id) handoff_run_id=${2:?missing value for --handoff-run-id}; shift 2 ;;
    --reuse-run-id) reuse_run_id=${2:?missing value for --reuse-run-id}; shift 2 ;;
    --reuse-acceptance-run-id) reuse_acceptance_run_id=${2:?missing value for --reuse-acceptance-run-id}; shift 2 ;;
    --recovery-evidence) recovery_evidence_paths+=(${2:?missing value for --recovery-evidence}); shift 2 ;;
    --release-source-revision) release_source_revision=${2:?missing value for --release-source-revision}; shift 2 ;;
    -h|--help) usage ;;
    *) echo "unknown argument: $1" >&2; usage ;;
  esac
done

[[ -n "$repo" && -n "$event" && -n "$head" && -n "$from_tag" && -n "$to_tag" && -n "$output" ]] || usage
[[ "$event" == workflow_dispatch ]] || { echo 'release plan requires workflow_dispatch' >&2; exit 2; }
repo_root=$(cd "$repo" && pwd -P)
output_path=$output
[[ "$output_path" = /* ]] || output_path="$repo_root/$output_path"
mkdir -p "$(dirname "$output_path")"

fail() {
  local code=$1 message=$2
  jq -n --arg code "$code" --arg message "$message" --arg head "$head" \
    '{schemaVersion:1,state:"failed",failureCode:$code,message:$message,headRevision:$head}' >"$output_path"
  printf '%s: %s\n' "$code" "$message" >&2
  exit 2
}

is_semver_tag() {
  [[ "$1" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]
}

[[ "$head" =~ ^[0-9a-f]{40}$ ]] || fail invalid_head_revision 'head must be a full commit SHA'
git -C "$repo_root" cat-file -e "${head}^{commit}" 2>/dev/null || fail invalid_head_revision 'head is not present in the checkout'
is_semver_tag "$from_tag" || fail invalid_from_tag 'from_tag must be a canonical semantic tag'
is_semver_tag "$to_tag" || fail invalid_to_tag 'to_tag must be a canonical semantic tag'
[[ "$from_tag" != "$to_tag" ]] || fail invalid_release_pair 'from_tag and to_tag must be distinct'
for value in "$publish_existing_tag" "$publish_candidate" "$post_release_acceptance" "$close_only"; do
  [[ "$value" == true || "$value" == false ]] || fail invalid_boolean 'release mode flags must be true or false'
done
[[ ! ( "$publish_existing_tag" == true && "$post_release_acceptance" == true ) ]] || \
  fail conflicting_release_modes 'publish_existing_tag and post_release_acceptance cannot both be true'
[[ ! ( "$publish_existing_tag" == true && "$publish_candidate" == true ) ]] || \
  fail conflicting_release_modes 'publish_existing_tag and publish_candidate cannot both be true'
[[ ! ( "$close_only" == true && "$post_release_acceptance" != true ) ]] || \
  fail invalid_close_only_mode 'close_only requires post_release_acceptance=true'

if [[ -n "$plan_bin" && "$plan_bin" != /* ]]; then
  plan_bin="$repo_root/$plan_bin"
fi
plan_bin=${plan_bin:-${AI_COCKPIT_RELEASE_PLAN_BIN:-$repo_root/target/release/ai-cockpit}}
[[ -x "$plan_bin" ]] || fail release_plan_binary_missing 'typed ReleasePlan resolver binary is missing'

expected_repository_id=$(jq -er '.repositoryId' "$repo_root/.ai/agent-interface.json") || \
  fail repository_identity_missing 'repository interface has no repositoryId'

contract_path=''
contract_digest=''
base_revision="$head"
if [[ -n "$contract_path_arg" || -n "$work_item_id" ]]; then
  if [[ -n "$contract_path_arg" ]]; then
    contract_path="$contract_path_arg"
    [[ "$contract_path" = /* ]] || contract_path="$repo_root/$contract_path"
  else
    contract_path="$repo_root/.ai/work-items/active/$work_item_id.contract.json"
  fi
  [[ -f "$contract_path" && ! -L "$contract_path" ]] || fail contract_not_regular 'selected Contract must be a regular file'
  selected_id=$(jq -er '.workItemId' "$contract_path") || fail contract_invalid 'selected Contract has no workItemId'
  [[ -z "$work_item_id" || "$selected_id" == "$work_item_id" ]] || fail work_item_id_mismatch 'Contract identity does not match work_item_id'
  selected_repository_id=$(jq -er '.repositoryId' "$contract_path") || fail contract_invalid 'Contract has no repositoryId'
  [[ "$selected_repository_id" == "$expected_repository_id" ]] || fail contract_repository_mismatch 'Contract belongs to another repository'
  base_revision=$(jq -er '.baseRevision' "$contract_path") || fail contract_invalid 'Contract has no baseRevision'
  [[ "$base_revision" =~ ^[0-9a-f]{40}$ ]] || fail contract_base_invalid 'Contract baseRevision is not a full commit SHA'
  git -C "$repo_root" cat-file -e "${base_revision}^{commit}" 2>/dev/null || fail contract_base_missing 'Contract baseRevision is not present'
  contract_digest="sha256:$(shasum -a 256 "$contract_path" | awk '{print $1}')"
  work_item_id="$selected_id"
fi

if [[ "$publish_existing_tag" == true || "$publish_candidate" == true || "$post_release_acceptance" == true || "$close_only" == true ]]; then
  [[ -n "$work_item_id" ]] || fail work_item_id_required 'this release mode requires an explicit Work Item identity'
fi
if [[ "$publish_existing_tag" != true && "$publish_candidate" != true && "$post_release_acceptance" != true ]]; then
  [[ -n "$handoff_run_id" ]] || fail handoff_run_id_required 'independent public acceptance requires handoff_run_id'
fi
if [[ "$publish_existing_tag" == true ]]; then
  ((${#recovery_evidence_paths[@]} > 0)) || fail recovery_evidence_required 'historical tag recovery requires archived recovery evidence'
fi
if [[ "$publish_candidate" == true ]]; then
  ((${#recovery_evidence_paths[@]} == 0)) || fail recovery_evidence_forbidden 'normal release cannot carry historical recovery evidence'
fi

source_contract_path=''
source_contract_digest=''
if [[ -n "$source_work_item_id" && -z "$source_contract_path_arg" ]]; then
  source_contract_path_arg="$repo_root/.ai/work-items/active/$source_work_item_id.contract.json"
fi
if [[ -n "$source_contract_path_arg" ]]; then
  source_contract_path="$source_contract_path_arg"
  [[ "$source_contract_path" = /* ]] || source_contract_path="$repo_root/$source_contract_path"
  [[ -f "$source_contract_path" && ! -L "$source_contract_path" ]] || fail source_contract_not_regular 'source Contract must be a regular file'
  selected_source_id=$(jq -er '.workItemId' "$source_contract_path") || fail source_contract_invalid 'source Contract has no workItemId'
  [[ -z "$source_work_item_id" || "$selected_source_id" == "$source_work_item_id" ]] || fail source_work_item_id_mismatch 'source Contract identity does not match source_work_item_id'
  selected_source_repository_id=$(jq -er '.repositoryId' "$source_contract_path") || fail source_contract_invalid 'source Contract has no repositoryId'
  [[ "$selected_source_repository_id" == "$expected_repository_id" ]] || fail source_contract_repository_mismatch 'source Contract belongs to another repository'
  source_contract_digest="sha256:$(shasum -a 256 "$source_contract_path" | awk '{print $1}')"
  source_work_item_id="$selected_source_id"
fi

evidence_json='[]'
if [[ -n "${recovery_evidence_paths[*]-}" ]]; then
  for evidence_path in "${recovery_evidence_paths[@]}"; do
    [[ -f "$evidence_path" && ! -L "$evidence_path" ]] || fail recovery_evidence_not_regular 'recovery evidence must be a regular file'
    evidence_digest="sha256:$(shasum -a 256 "$evidence_path" | awk '{print $1}')"
    evidence_json=$(jq -c --arg path "${evidence_path#$repo_root/}" --arg digest "$evidence_digest" \
      '. + [{class:"archived_recovery",path:$path,digest:$digest}]' <<<"$evidence_json")
  done
fi

if [[ -n "$release_source_revision" ]]; then
  [[ "$release_source_revision" =~ ^[0-9a-f]{40}$ ]] || fail invalid_source_revision 'release source revision must be a full commit SHA'
else
  release_source_revision=$(git -C "$repo_root" ls-remote origin "refs/tags/$to_tag^{}" | awk 'NR == 1 {print $1}')
fi
[[ "$release_source_revision" =~ ^[0-9a-f]{40}$ ]] || fail release_tag_missing 'immutable release tag does not resolve to a commit'

request_path=$(mktemp "${TMPDIR:-/tmp}/ai-cockpit-release-request.XXXXXX")
trap 'rm -f "$request_path"' EXIT
jq -n \
  --arg repositoryId "$expected_repository_id" \
  --arg event "$event" \
  --arg headRevision "$head" \
  --arg baseRevision "$base_revision" \
  --arg sourceRevision "$release_source_revision" \
  --arg version "${to_tag#v}" \
  --arg fromTag "$from_tag" \
  --arg toTag "$to_tag" \
  --arg workItemId "$work_item_id" \
  --arg sourceWorkItemId "$source_work_item_id" \
  --arg contractPath "${contract_path#$repo_root/}" \
  --arg contractDigest "$contract_digest" \
  --arg sourceContractPath "${source_contract_path#$repo_root/}" \
  --arg sourceContractDigest "$source_contract_digest" \
  --arg handoffRunId "$handoff_run_id" \
  --arg reuseRunId "$reuse_run_id" \
  --arg reuseAcceptanceRunId "$reuse_acceptance_run_id" \
  --argjson publishExistingTag "$publish_existing_tag" \
  --argjson publishCandidate "$publish_candidate" \
  --argjson postReleaseAcceptance "$post_release_acceptance" \
  --argjson closeOnly "$close_only" \
  --argjson recoveryEvidence "$evidence_json" \
  '{schemaVersion:1,repositoryId:$repositoryId,event:$event,headRevision:$headRevision,baseRevision:$baseRevision,sourceRevision:$sourceRevision,version:$version,fromTag:$fromTag,toTag:$toTag,publishExistingTag:$publishExistingTag,publishCandidate:$publishCandidate,postReleaseAcceptance:$postReleaseAcceptance,closeOnly:$closeOnly,workItemId:(if $workItemId == "" then null else $workItemId end),sourceWorkItemId:(if $sourceWorkItemId == "" then null else $sourceWorkItemId end),contractPath:(if $contractPath == "" then null else $contractPath end),contractDigest:(if $contractDigest == "" then null else $contractDigest end),sourceContractPath:(if $sourceContractPath == "" then null else $sourceContractPath end),sourceContractDigest:(if $sourceContractDigest == "" then null else $sourceContractDigest end),recoveryEvidence:$recoveryEvidence,reuseRunId:(if $reuseRunId == "" then null else $reuseRunId end),reuseAcceptanceRunId:(if $reuseAcceptanceRunId == "" then null else $reuseAcceptanceRunId end),handoffRunId:(if $handoffRunId == "" then null else $handoffRunId end),requestedMode:null}' \
  >"$request_path"

if ! "$plan_bin" release-plan --input "$request_path" --output "$output_path" >/dev/null; then
  fail release_plan_resolution_failed 'Runtime rejected the ReleaseRequest'
fi
jq -e '.plan.schemaVersion == 1 and (.planDigest | type == "string") and (.plan.request.mode | type == "string")' \
  "$output_path" >/dev/null || fail release_plan_invalid 'Runtime returned an invalid ReleasePlan envelope'
