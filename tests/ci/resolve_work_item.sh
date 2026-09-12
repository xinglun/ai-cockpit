#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: resolve_work_item.sh --repo ROOT --event EVENT --head SHA --output FILE
  [--pr-head-ref REF] [--pr-url URL] [--work-item-id ID]
  [--contract-path PATH] [--from-tag TAG] [--to-tag TAG]
  [--publish-existing-tag true|false] [--handoff-run-id ID]
  [--github-repository OWNER/REPO] [--release-source-revision SHA]
EOF
  exit 64
}

repo=''
event=''
head=''
output=''
pr_head_ref=''
pr_url=''
work_item_id=''
contract_path_arg=''
from_tag=''
to_tag=''
publish_existing_tag=false
handoff_run_id=''
github_repository="${GITHUB_REPOSITORY:-}"
release_source_revision_arg=''
recovery_lineage=''

while (($# > 0)); do
  case "$1" in
    --repo) repo=${2:?missing value for --repo}; shift 2 ;;
    --event) event=${2:?missing value for --event}; shift 2 ;;
    --head) head=${2:?missing value for --head}; shift 2 ;;
    --output) output=${2:?missing value for --output}; shift 2 ;;
    --pr-head-ref) pr_head_ref=${2:?missing value for --pr-head-ref}; shift 2 ;;
    --pr-url) pr_url=${2:?missing value for --pr-url}; shift 2 ;;
    --work-item-id) work_item_id=${2:?missing value for --work-item-id}; shift 2 ;;
    --contract-path) contract_path_arg=${2:?missing value for --contract-path}; shift 2 ;;
    --from-tag) from_tag=${2:?missing value for --from-tag}; shift 2 ;;
    --to-tag) to_tag=${2:?missing value for --to-tag}; shift 2 ;;
    --publish-existing-tag) publish_existing_tag=${2:?missing value for --publish-existing-tag}; shift 2 ;;
    --handoff-run-id) handoff_run_id=${2:?missing value for --handoff-run-id}; shift 2 ;;
    --github-repository) github_repository=${2:?missing value for --github-repository}; shift 2 ;;
    --release-source-revision) release_source_revision_arg=${2:?missing value for --release-source-revision}; shift 2 ;;
    -h|--help) usage ;;
    *) echo "unknown argument: $1" >&2; usage ;;
  esac
done

[[ -n "$repo" && -n "$event" && -n "$head" && -n "$output" ]] || usage
repo_root=$(cd "$repo" && pwd -P)
if [[ "$output" = /* ]]; then
  output_path=$output
else
  output_path="$repo_root/$output"
fi
mkdir -p "$(dirname "$output_path")"

fail() {
  local code=$1 message=$2
  jq -n \
    --arg code "$code" \
    --arg message "$message" \
    --arg event "$event" \
    --arg head "$head" \
    '{schemaVersion:1,kind:"work_item_selection",state:"failed",failureCode:$code,message:$message,event:$event,headRevision:$head}' \
    > "$output_path"
  printf '%s: %s\n' "$code" "$message" >&2
  exit 2
}

is_semver_tag() {
  [[ "$1" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]
}

is_release_tag_push() {
  [[ "$event" == push && "${GITHUB_REF:-}" == refs/tags/* ]]
}

emit_ordinary_route() {
  local method=$1 mode=$2
  jq -n \
    --arg event "$event" \
    --arg mode "$mode" \
    --arg head "$head" \
    --arg method "$method" \
    '{schemaVersion:1,kind:"work_item_selection",state:"ready",event:$event,mode:$mode,headRevision:$head,releaseSourceRevision:null,workItemId:null,contractPath:null,contractDigest:null,baseRevision:null,selectionMethod:$method,recoveryLineage:null,fromTag:null,toTag:null}' \
    > "$output_path"
  exit 0
}

validate_archived_contract_anchor() {
  local archive_contract=$1
  local archive_id=$2
  local archive_manifest="$archive_dir/$archive_id.archive.json"
  [[ -f "$archive_contract" && ! -L "$archive_contract" ]] || \
    fail archived_contract_not_regular 'selected archived Contract must be a regular non-symlink file'
  [[ -f "$archive_manifest" && ! -L "$archive_manifest" ]] || \
    fail archive_manifest_missing 'selected archived Contract has no regular archive manifest'
  local manifest_id manifest_state manifest_path expected_digest actual_digest
  manifest_id=$(jq -er '.workItemId' "$archive_manifest" 2>/dev/null) || \
    fail archive_manifest_invalid 'selected archive manifest has no valid workItemId'
  [[ "$manifest_id" == "$archive_id" ]] || \
    fail archive_manifest_identity_mismatch 'archive manifest Work Item identity does not match the selected Contract'
  manifest_state=$(jq -er '.state' "$archive_manifest" 2>/dev/null) || \
    fail archive_manifest_invalid 'selected archive manifest has no state'
  [[ "$manifest_state" == archived || "$manifest_state" == superseded ]] || \
    fail archive_manifest_invalid 'selected archive manifest state is not archived or superseded'
  manifest_path=$(jq -er '.files.contractPath' "$archive_manifest" 2>/dev/null) || \
    fail archive_manifest_invalid 'selected archive manifest has no contractPath'
  [[ "$manifest_path" == ".ai/work-items/archive/$archive_id.contract.json" ]] || \
    fail archive_manifest_identity_mismatch 'archive manifest contractPath does not match the selected Contract'
  expected_digest="sha256:$(shasum -a 256 "$archive_contract" | awk '{print $1}')"
  actual_digest=$(jq -er '.files.contractDigest' "$archive_manifest" 2>/dev/null) || \
    fail archive_manifest_invalid 'selected archive manifest has no contractDigest'
  [[ "$actual_digest" == "$expected_digest" ]] || \
    fail archive_manifest_digest_mismatch 'archive manifest Contract digest does not match the selected Contract'
}

[[ "$head" =~ ^[0-9a-f]{40}$ ]] || fail invalid_head_revision 'head revision must be a full commit SHA'
git -C "$repo_root" cat-file -e "${head}^{commit}" 2>/dev/null || fail invalid_head_revision 'head revision is not present in the checkout'

if [[ "$event" == workflow_dispatch ]]; then
  is_semver_tag "$to_tag" || fail invalid_to_tag 'to_tag must be a canonical semantic Release tag'
  if [[ -n "$from_tag" ]]; then
    is_semver_tag "$from_tag" || fail invalid_from_tag 'from_tag must be a canonical semantic Release tag'
    [[ "$from_tag" != "$to_tag" ]] || fail invalid_release_pair 'from_tag and to_tag must be distinct'
  fi
  [[ "$publish_existing_tag" == true || "$publish_existing_tag" == false ]] || \
    fail invalid_publish_mode 'publish_existing_tag must be true or false'
fi

if [[ "$event" == workflow_dispatch && "$publish_existing_tag" != true ]]; then
  [[ -n "$handoff_run_id" && "$handoff_run_id" =~ ^[1-9][0-9]*$ ]] || \
    fail handoff_run_id_required 'independent public acceptance requires a completed handoff_run_id'
  jq -n \
    --arg event "$event" \
    --arg mode public_acceptance \
    --arg head "$head" \
    --arg from "$from_tag" \
    --arg to "$to_tag" \
    --arg handoff "$handoff_run_id" \
    '{schemaVersion:1,kind:"work_item_selection",state:"ready",event:$event,mode:$mode,headRevision:$head,fromTag:$from,toTag:$to,handoffRunId:$handoff,contractPath:null,workItemId:null,baseRevision:null,releaseSourceRevision:null,selectionMethod:"explicit_public_handoff"}' \
    > "$output_path"
  exit 0
fi

if [[ "$event" == workflow_dispatch && "$publish_existing_tag" == true ]]; then
  [[ -n "$work_item_id" || -n "$contract_path_arg" ]] || \
    fail work_item_id_required 'recovery requires an explicit work_item_id or contract path'
  [[ -n "$to_tag" ]] || fail invalid_to_tag 'recovery requires to_tag'
fi

if [[ "$event" == pull_request ]]; then
  [[ -n "$pr_head_ref" && -n "$pr_url" ]] || fail pull_request_identity_required 'pull request branch and URL are required'
fi

active_dir="$repo_root/.ai/work-items/active"
archive_dir="$repo_root/.ai/work-items/archive"
all_contracts=()
if [[ -d "$active_dir" ]]; then
  while IFS= read -r contract_path_from_find; do
    [[ -n "$contract_path_from_find" ]] && all_contracts+=("$contract_path_from_find")
  done < <(find "$active_dir" -maxdepth 1 -type f -name '*.contract.json' -print | sort)
fi

archive_contracts=()
if [[ -d "$archive_dir" ]]; then
  while IFS= read -r contract_path_from_find; do
    [[ -n "$contract_path_from_find" ]] && archive_contracts+=("$contract_path_from_find")
  done < <(find "$archive_dir" -maxdepth 1 -type f -name '*.contract.json' -print | sort)
fi

candidate_contracts=()
archived_candidate_contracts=()
selection_method=''
if [[ -n "$contract_path_arg" ]]; then
  if [[ "$contract_path_arg" = /* ]]; then
    explicit_path=$contract_path_arg
  else
    explicit_path="$repo_root/$contract_path_arg"
  fi
  explicit_dir=$(dirname "$explicit_path")
  [[ -d "$explicit_dir" ]] || fail contract_path_missing 'explicit Contract path parent directory does not exist'
  explicit_path=$(cd "$explicit_dir" && pwd -P)/$(basename "$explicit_path")
  [[ "$explicit_path" == "$active_dir/"*.contract.json ]] || \
    fail contract_path_out_of_scope 'contract path must point into the active Work Item directory'
  candidate_contracts=("$explicit_path")
  selection_method='explicit_contract_path'
elif [[ -n "$work_item_id" ]]; then
  [[ "$work_item_id" =~ ^WI-[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || \
    fail invalid_work_item_id 'work_item_id is not a safe Work Item identifier'
  candidate_contracts=("$active_dir/$work_item_id.contract.json")
  selection_method='explicit_work_item_id'
elif [[ "$event" == pull_request ]]; then
  selection_method='pull_request_binding'
  for contract_path in "${all_contracts[@]}"; do
    if ! branch=$(jq -r '.resourceContext.branch // empty' "$contract_path" 2>/dev/null); then
      fail contract_invalid 'an active Contract is not valid JSON'
    fi
    if ! bound_pr=$(jq -r '.resourceContext.pullRequest // empty' "$contract_path" 2>/dev/null); then
      fail contract_invalid 'an active Contract is not valid JSON'
    fi
    if [[ "$branch" == "$pr_head_ref" && "$bound_pr" == "$pr_url" ]]; then
      candidate_contracts+=("$contract_path")
    fi
  done
  if ((${#archive_contracts[@]} > 0)); then
    for contract_path in "${archive_contracts[@]}"; do
      if ! branch=$(jq -r '.resourceContext.branch // empty' "$contract_path" 2>/dev/null); then
        fail contract_invalid 'an archived Contract is not valid JSON'
      fi
      if ! bound_pr=$(jq -r '.resourceContext.pullRequest // empty' "$contract_path" 2>/dev/null); then
        fail contract_invalid 'an archived Contract is not valid JSON'
      fi
      [[ -n "$branch" && -n "$bound_pr" ]] || continue
      if [[ "$branch" == "$pr_head_ref" && "$bound_pr" == "$pr_url" ]]; then
        archived_candidate_contracts+=("$contract_path")
      fi
    done
  fi
  if ((${#archived_candidate_contracts[@]} > 0)); then
    if ((${#candidate_contracts[@]} > 0)); then
      fail work_item_contract_ambiguous 'active and archived Contracts are bound to the same pull request identity'
    fi
    candidate_contracts=("${archived_candidate_contracts[@]}")
    selection_method='archived_contract_read_only'
  fi
elif [[ "$event" == push ]]; then
  [[ -n "$github_repository" ]] || fail github_repository_required 'tag/merge selection requires GITHUB_REPOSITORY'
  [[ -n "${GH_TOKEN:-}" ]] || fail github_token_required 'tag/merge selection requires GH_TOKEN'
  selection_method='merged_pull_request_binding'
  merged_pr_output=''
  merged_pr_output=$(gh api "repos/$github_repository/commits/$head/pulls" \
    -H 'Accept: application/vnd.github+json' \
    --jq '.[] | select(.merged_at != null) | [.html_url, .head.ref] | @tsv' \
  ) || fail merged_pr_lookup_failed 'could not resolve the merged pull request for the release commit'
  merged_prs=()
  while IFS= read -r merged_pr; do
    [[ -n "$merged_pr" ]] && merged_prs+=("$merged_pr")
  done <<< "$merged_pr_output"
  if ((${#merged_prs[@]} > 0)); then
    for pr in "${merged_prs[@]}"; do
      IFS=$'\t' read -r merged_pr_url merged_pr_ref <<< "$pr"
      for contract_path in "${all_contracts[@]}"; do
        if ! branch=$(jq -r '.resourceContext.branch // empty' "$contract_path" 2>/dev/null); then
          fail contract_invalid 'an active Contract is not valid JSON'
        fi
        if ! bound_pr=$(jq -r '.resourceContext.pullRequest // empty' "$contract_path" 2>/dev/null); then
          fail contract_invalid 'an active Contract is not valid JSON'
        fi
        if [[ "$branch" == "$merged_pr_ref" && "$bound_pr" == "$merged_pr_url" ]]; then
          candidate_contracts+=("$contract_path")
        fi
      done
    done
  fi
else
  fail unsupported_event 'event must be pull_request, push, or workflow_dispatch'
fi

if ((${#candidate_contracts[@]} == 0)); then
  if [[ "$event" == workflow_dispatch && "$publish_existing_tag" == true ]]; then
    fail work_item_contract_missing 'the explicitly requested active Contract does not exist'
  fi
  if [[ "$event" == pull_request ]]; then
    emit_ordinary_route ordinary_repository_route pull_request
  fi
  if [[ "$event" == push ]] && ! is_release_tag_push; then
    emit_ordinary_route ordinary_repository_route merge
  fi
  fail work_item_contract_unresolved 'no active or exact archived Contract is explicitly bound to this event identity'
fi
if ((${#candidate_contracts[@]} > 1)); then
  candidates=$(printf '%s\n' "${candidate_contracts[@]}" | sed 's#^.*/##' | paste -sd, -)
  fail work_item_contract_ambiguous "multiple active Contracts are bound to this event: $candidates"
fi

contract_path=${candidate_contracts[0]}
[[ -f "$contract_path" && ! -L "$contract_path" ]] || fail contract_not_regular 'selected Contract must be a regular non-symlink file'
contract_relative_candidate=${contract_path#"$repo_root/"}
archived_contract_selected=false
if [[ "$contract_relative_candidate" == .ai/work-items/archive/*.contract.json ]]; then
  archived_contract_selected=true
  [[ "$event" == pull_request && "$selection_method" == archived_contract_read_only ]] || \
    fail contract_path_out_of_scope 'archived Contracts are allowed only for exact pull-request history routes'
  validate_archived_contract_anchor "$contract_path" "$(basename "$contract_path" .contract.json)"
fi
contract_id=$(jq -er '.workItemId' "$contract_path") || fail contract_invalid 'selected Contract is not valid JSON or has no workItemId'
if [[ -n "$work_item_id" && "$contract_id" != "$work_item_id" ]]; then
  fail work_item_id_mismatch 'selected Contract workItemId does not match the requested work_item_id'
fi
expected_repository_id=$(jq -er '.repositoryId' "$repo_root/.ai/agent-interface.json") || \
  fail repository_identity_missing 'repository interface has no repositoryId'
contract_repository_id=$(jq -er '.repositoryId' "$contract_path") || fail contract_invalid 'selected Contract has no repositoryId'
[[ "$contract_repository_id" == "$expected_repository_id" ]] || \
  fail contract_repository_mismatch 'selected Contract belongs to a different repository'
contract_state=$(jq -er '.state' "$contract_path") || fail contract_invalid 'selected Contract has no state'
if [[ "$archived_contract_selected" != true ]]; then
  [[ "$contract_state" != archived && "$contract_state" != closed ]] || \
    fail contract_not_active 'selected Contract is archived or closed'
fi
base_revision=$(jq -er '.baseRevision' "$contract_path") || fail contract_invalid 'selected Contract has no baseRevision'
[[ "$base_revision" =~ ^[0-9a-f]{40}$ ]] || fail contract_base_invalid 'selected Contract baseRevision is not a full commit SHA'
git -C "$repo_root" cat-file -e "${base_revision}^{commit}" 2>/dev/null || \
  fail contract_base_missing 'selected Contract baseRevision is not present in the checkout'
jq -e '.scope | type == "array" and length > 0' "$contract_path" >/dev/null || \
  fail contract_scope_missing 'selected Contract has no non-empty scope'

if [[ "$event" == workflow_dispatch && "$publish_existing_tag" == true ]]; then
  predecessor_id=$(jq -r '.predecessorWorkItemId // empty' "$contract_path")
  predecessor_digest=$(jq -r '.predecessorContractDigest // empty' "$contract_path")
  decision_relative=$(jq -r '.recoveryDecisionPath // empty' "$contract_path")
  if [[ -z "$predecessor_id" && -z "$predecessor_digest" && -z "$decision_relative" ]]; then
    # An explicitly selected Work Item without predecessor fields is a
    # technical retry of that same Work Item, not an implicit successor.
    # A human/provider still selected the identity explicitly above; do not
    # manufacture lineage merely because an immutable Release is being
    # recovered.
    recovery_lineage=standalone_retry
  else
    [[ -n "$predecessor_id" && -n "$predecessor_digest" && -n "$decision_relative" ]] || \
      fail recovery_binding_missing 'recovery Contract has no complete predecessor binding'
    decision_path="$repo_root/$decision_relative"
    [[ "$decision_path" == "$repo_root/.ai/decisions/"* ]] || \
      fail recovery_binding_invalid 'recovery decision path is outside the repository decision directory'
    [[ -f "$decision_path" && ! -L "$decision_path" ]] || \
      fail recovery_binding_invalid 'recovery decision must be a regular non-symlink file'
    jq -e \
      --arg predecessor "$predecessor_id" \
      --arg successor "$contract_id" \
      '.decision == "successor" and .predecessorWorkItemId == $predecessor and .successorWorkItemId == $successor' \
      "$decision_path" >/dev/null || fail recovery_binding_invalid 'recovery decision does not bind the selected predecessor and successor'
    recovery_lineage=successor
  fi
fi

release_source_revision=''
if [[ "$event" == workflow_dispatch ]] || is_release_tag_push; then
  tag="$to_tag"
  [[ -n "$tag" ]] || tag="${GITHUB_REF_NAME:-}"
  is_semver_tag "$tag" || fail invalid_release_tag 'release tag must be canonical'
  if [[ -n "$release_source_revision_arg" ]]; then
    release_source_revision=$release_source_revision_arg
  else
    release_source_revision=$(git -C "$repo_root" ls-remote origin "refs/tags/$tag^{}" | awk 'NR == 1 {print $1}')
  fi
  [[ "$release_source_revision" =~ ^[0-9a-f]{40}$ ]] || fail release_tag_missing 'immutable release tag does not resolve to a commit'
fi

if [[ "$event" == workflow_dispatch && "$publish_existing_tag" == true ]]; then
  mode=release_recovery
elif [[ "$event" == pull_request ]]; then
  mode=pull_request
elif is_release_tag_push; then
  mode=tag_release
else
  mode=merge
fi
contract_relative=${contract_path#"$repo_root/"}
contract_digest="sha256:$(shasum -a 256 "$contract_path" | awk '{print $1}')"
jq -n \
  --arg event "$event" \
  --arg mode "$mode" \
  --arg head "$head" \
  --arg source "$release_source_revision" \
  --arg id "$contract_id" \
  --arg path "$contract_relative" \
  --arg digest "$contract_digest" \
  --arg base "$base_revision" \
  --arg method "$selection_method" \
  --arg lineage "$recovery_lineage" \
  --arg from "$from_tag" \
  --arg to "$to_tag" \
  '{schemaVersion:1,kind:"work_item_selection",state:"ready",event:$event,mode:$mode,headRevision:$head,releaseSourceRevision:$source,workItemId:$id,contractPath:$path,contractDigest:$digest,baseRevision:$base,selectionMethod:$method,recoveryLineage:(if $lineage == "" then null else $lineage end),fromTag:(if $from == "" then null else $from end),toTag:(if $to == "" then null else $to end)}' \
  > "$output_path"
