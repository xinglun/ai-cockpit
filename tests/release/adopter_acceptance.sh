#!/usr/bin/env bash
set -euo pipefail

# Post-release acceptance harness. AI Cockpit is obtained only from the
# requested public Release. Cargo is used only to create and test the adopter.

usage() {
  cat <<'USAGE'
Usage: adopter_acceptance.sh \
  --repository OWNER/REPOSITORY \
  --tag vX.Y.Z \
  --target TARGET \
  --output DIRECTORY \
  [--candidate-dir DIRECTORY] \
  [--source-repo DIRECTORY] \
  [--publish-handoff FILE] \
  [--resume]
USAGE
}

die() {
  failure_reason=$*
  printf 'adopter acceptance failed: %s\n' "$failure_reason" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || die "required command is unavailable: $1"
}

# GitHub's Release API is rate-limited for anonymous callers.  Prefer the
# workflow token when one is provided, while keeping local/public use working
# without credentials.  This helper is intentionally limited to API metadata;
# release assets remain fetched from the immutable public Release URLs below.
github_api_get() {
  local url=$1
  local destination=$2
  local token="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
  if [[ -n "$token" ]]; then
    curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
      -H 'Accept: application/vnd.github+json' -H "Authorization: Bearer $token" \
      "$url" > "$destination"
  else
    curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
      "$url" > "$destination"
  fi
}

repository=''
tag=''
target=''
output=''
source_repo=''
candidate_dir=''
publish_handoff=''
resume=false

while (($# > 0)); do
  case "$1" in
    --repository)
      [[ $# -ge 2 ]] || die "--repository requires a value"
      repository=$2
      shift 2
      ;;
    --tag)
      [[ $# -ge 2 ]] || die "--tag requires a value"
      tag=$2
      shift 2
      ;;
    --target)
      [[ $# -ge 2 ]] || die "--target requires a value"
      target=$2
      shift 2
      ;;
    --output)
      [[ $# -ge 2 ]] || die "--output requires a value"
      output=$2
      shift 2
      ;;
    --source-repo)
      [[ $# -ge 2 ]] || die "--source-repo requires a value"
      source_repo=$2
      shift 2
      ;;
    --candidate-dir)
      [[ $# -ge 2 ]] || die "--candidate-dir requires a value"
      candidate_dir=$2
      shift 2
      ;;
    --publish-handoff)
      [[ $# -ge 2 ]] || die "--publish-handoff requires a value"
      publish_handoff=$2
      shift 2
      ;;
    --resume)
      resume=true
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      usage >&2
      die "unknown argument: $1"
      ;;
  esac
done

[[ "$repository" =~ ^[^/]+/[^/]+$ ]] || die 'repository must be OWNER/REPOSITORY'
[[ "$tag" =~ ^v[0-9]+[.][0-9]+[.][0-9]+$ ]] || die 'tag must be a canonical vX.Y.Z tag'
[[ "$target" =~ ^(aarch64-apple-darwin|aarch64-unknown-linux-gnu|x86_64-apple-darwin|x86_64-pc-windows-msvc|x86_64-unknown-linux-gnu)$ ]] || die "unsupported target: $target"
[[ -n "$output" ]] || die '--output is required'

for command_name in bash curl jq git cargo tar; do
  require_command "$command_name"
done

if [[ "$target" == x86_64-pc-windows-msvc ]]; then
  require_command unzip
  archive_extension=zip
else
  if ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then
    die 'required SHA-256 implementation is unavailable'
  fi
  archive_extension=tar.gz
fi

if [[ -z "$source_repo" ]]; then
  if source_repo="$(git rev-parse --show-toplevel 2>/dev/null)"; then :; fi
fi
[[ -n "$source_repo" && -d "$source_repo" ]] || die 'source repository is unavailable; pass --source-repo'
source_repo="$(cd "$source_repo" && pwd -P)"
source_top="$(git -C "$source_repo" rev-parse --show-toplevel 2>/dev/null)"
[[ -n "$source_top" ]] || die 'source repository is not a Git checkout'
source_top="$(cd "$source_top" && pwd -P)"
[[ "$source_top" == "$source_repo" ]] || die '--source-repo must identify the Git top-level directory'
source_project="$source_repo/.ai/project.json"
[[ -f "$source_project" && ! -L "$source_project" ]] || die 'source repository .ai/project.json is missing or symlinked'
source_repository_id="$(jq -er '.repositoryId | select(type == "string" and test("^sha256:[0-9a-f]{64}$"))' "$source_project")" || die 'source repositoryId is missing or malformed'
if [[ -n "$candidate_dir" ]]; then
  [[ -d "$candidate_dir" && ! -L "$candidate_dir" ]] || die 'candidate directory must be a regular directory'
  candidate_dir="$(cd "$candidate_dir" && pwd -P)"
fi
if [[ -n "$publish_handoff" ]]; then
  [[ -f "$publish_handoff" && ! -L "$publish_handoff" ]] || die 'publish handoff must be a regular, non-symlinked file'
  publish_handoff="$(cd "$(dirname "$publish_handoff")" && pwd -P)/$(basename "$publish_handoff")"
fi

mkdir -p "$output"
output="$(cd "$output" && pwd)"
if [[ "$resume" != true && -n "$(find "$output" -mindepth 1 -print -quit 2>/dev/null)" ]]; then
  die "output directory must be empty: $output"
fi
resume_cache="$output/.resume-cache"
phase_identity="$output/phase-identity.json"
phase_receipts="$output/phase-receipts.json"

tmpdir=''
if tmpdir="$(printenv TMPDIR)"; then :; fi
[[ -n "$tmpdir" ]] || tmpdir=/tmp
run_parent="$(cd "$tmpdir" 2>/dev/null && pwd -P)" || die "TMPDIR is not a directory: $tmpdir"
run_root=''
runtime_root=''
adopter_root=''
isolated_home=''
isolated_xdg=''
isolated_tmp=''
isolated_cargo=''
download_root=''
steps_jsonl=''
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
finished_at=''
overall_state=failed
failure_reason=''
release_published=false
staged_candidate=false
runtime_version=''
runtime_digest=''
repository_id=''
source_ai_state=unknown
source_before_status=''
source_after_status=''
runtime_bin=''
phase_plan=''
acceptance_scope=''
acceptance_phase=''
acceptance_evidence=''
current_phase='prepare'
close_ready=false
rustup_home=''
rustup_toolchain=''
cleanup_state=not_started
cleanup_removed=false
cleanup_validated=false
cleanup_reason=''
run_root_identity=''
cleanup_target_path=''
cleanup_target_parent=''
cleanup_target_basename=''
cleanup_target_identity=''
prior_cleanup_recovery_state=not_attempted
prior_cleanup_recovery_reason=''
prior_cleanup_unresolved=false
adopter_repository_id=''
close_decision_work_item=''
close_decision_repository_id=''
close_decision_digest=''
close_decision_path=''
close_decision_validated=false
record_step() {
  local name=$1
  local state=$2
  local reason=''
  if [[ $# -ge 3 ]]; then reason=$3; fi
  jq -cn --arg name "$name" --arg state "$state" --arg reason "$reason" \
    '{name:$name,state:$state} + (if $reason == "" then {} else {reason:$reason} end)' >> "$steps_jsonl"
}

mark_passed() {
  local reason=''
  if [[ $# -ge 2 ]]; then reason=$2; fi
  record_step "$1" passed "$reason"
}

source "$(cd "$(dirname "$0")" && pwd)/isolation_manifest.sh"

write_sums() {
  : > "$output/SHA256SUMS"
  while IFS= read -r evidence_path; do
    [[ "$evidence_path" == "$output/SHA256SUMS" ]] && continue
    relative_path="$(printf '%s' "$evidence_path" | sed "s#^$output/##")"
    printf '%s  %s\n' "$(sha256_file "$evidence_path")" "$relative_path" >> "$output/SHA256SUMS"
  done < <(find "$output" -type f ! -name SHA256SUMS -print | LC_ALL=C sort)
}

cleanup_run_root() {
  local parent_real root_real root_name
  cleanup_state=failed
  cleanup_removed=false
  cleanup_validated=false
  cleanup_reason='run_root cleanup was not attempted'

  if [[ "${prior_cleanup_unresolved:-false}" == true ]]; then
    cleanup_reason="${prior_cleanup_recovery_reason:-prior cleanup target could not be safely recovered}"
    return 1
  fi
  [[ -n "${run_root:-}" ]] || {
    cleanup_state=passed
    cleanup_removed=true
    cleanup_validated=true
    cleanup_reason='run_root was never created'
    return 0
  }
  [[ -d "$run_parent" ]] || {
    cleanup_reason='run_root parent directory is missing'
    return 1
  }
  if [[ ! -e "$run_root" && ! -L "$run_root" ]]; then
    cleanup_state=passed
    cleanup_removed=true
    cleanup_validated=true
    cleanup_reason='run_root was already absent'
    return 0
  fi
  [[ -d "$run_root" && ! -L "$run_root" ]] || {
    cleanup_reason='run_root is not the original regular directory'
    return 1
  }
  [[ -n "$run_root_identity" && "$(path_identity "$run_root")" == "$run_root_identity" ]] || {
    cleanup_reason='run_root device/inode identity changed'
    return 1
  }

  parent_real="$(cd "$run_parent" 2>/dev/null && pwd -P)" || {
    cleanup_reason='run_root parent could not be canonicalized'
    return 1
  }
  root_real="$(cd "$run_root" 2>/dev/null && pwd -P)" || {
    cleanup_reason='run_root could not be canonicalized'
    return 1
  }
  root_name="${root_real##*/}"
  [[ "$parent_real" != / && "$root_real" != "$parent_real" ]] || {
    cleanup_reason='run_root safety boundary rejected the path'
    return 1
  }
  case "$root_real" in
    "$parent_real"/ai-cockpit-adopter-acceptance.*) ;;
    *)
      cleanup_reason='run_root name or parent did not match the acceptance temp boundary'
      return 1
      ;;
  esac
  [[ "$root_name" == ai-cockpit-adopter-acceptance.* ]] || {
    cleanup_reason='run_root basename did not match the acceptance temp boundary'
    return 1
  }
  cleanup_validated=true
  if remove_exact_tree "$run_root"; then
    cleanup_state=passed
    cleanup_removed=true
    cleanup_reason='validated run_root removed'
    return 0
  fi
  cleanup_reason='validated run_root removal failed'
  return 1
}

# The acceptance roots are canonical absolute paths validated above. Avoid
# `rm --` because BSD/macOS rm rejects that GNU-only option. Git may also
# finish background maintenance immediately after a commit; retry only the
# exact validated root for a bounded interval, then fail closed.
remove_exact_tree() {
  local path="$1"
  local attempt
  for attempt in 1 2 3 4 5; do
    if rm -rf "$path" && [[ ! -e "$path" && ! -L "$path" ]]; then
      return 0
    fi
    [[ "$attempt" -lt 5 ]] || break
    sleep 0.2
  done
  return 1
}

recover_prior_cleanup() {
  [[ "$resume" == true && -f "$output/cleanup.json" && ! -L "$output/cleanup.json" ]] || return 0
  [[ "$(jq -r '.state // empty' "$output/cleanup.json" 2>/dev/null)" == failed ]] || return 0

  local target_path target_parent target_basename target_identity parent_real root_real
  target_path="$(jq -er '.target.path | select(type == "string" and length > 0)' "$output/cleanup.json" 2>/dev/null)" || {
    prior_cleanup_unresolved=true
    prior_cleanup_recovery_state=blocked
    prior_cleanup_recovery_reason='prior cleanup receipt has no exact target path'
    return 1
  }
  target_parent="$(jq -er '.target.parent | select(type == "string" and length > 0)' "$output/cleanup.json" 2>/dev/null)" || {
    prior_cleanup_unresolved=true
    prior_cleanup_recovery_state=blocked
    prior_cleanup_recovery_reason='prior cleanup receipt has no target parent'
    return 1
  }
  target_basename="$(jq -er '.target.basename | select(type == "string" and length > 0)' "$output/cleanup.json" 2>/dev/null)" || {
    prior_cleanup_unresolved=true
    prior_cleanup_recovery_state=blocked
    prior_cleanup_recovery_reason='prior cleanup receipt has no target basename'
    return 1
  }
  target_identity="$(jq -er '.target.deviceInode | select(type == "string" and length > 0)' "$output/cleanup.json" 2>/dev/null)" || {
    prior_cleanup_unresolved=true
    prior_cleanup_recovery_state=blocked
    prior_cleanup_recovery_reason='prior cleanup receipt has no device/inode identity'
    return 1
  }
  cleanup_target_path="$target_path"
  cleanup_target_parent="$target_parent"
  cleanup_target_basename="$target_basename"
  cleanup_target_identity="$target_identity"
  parent_real="$(cd "$target_parent" 2>/dev/null && pwd -P)" || parent_real=''
  root_real="$(cd "$target_path" 2>/dev/null && pwd -P)" || root_real=''
  if [[ -z "$parent_real" || -z "$root_real" || "$parent_real" == / || "$root_real" != "$parent_real"/* \
    || "$target_basename" != ai-cockpit-adopter-acceptance.* \
    || "${root_real##*/}" != "$target_basename" \
    || ! -d "$target_path" || -L "$target_path" \
    || "$(path_identity "$target_path" 2>/dev/null)" != "$target_identity" ]]; then
    prior_cleanup_unresolved=true
    prior_cleanup_recovery_state=blocked
    prior_cleanup_recovery_reason='prior cleanup target is absent or no longer matches its recorded safety identity'
    return 1
  fi
  if remove_exact_tree "$target_path"; then
    prior_cleanup_recovery_state=passed
    prior_cleanup_recovery_reason='prior validated run_root was removed before resuming acceptance'
    return 0
  fi
  prior_cleanup_unresolved=true
  prior_cleanup_recovery_state=blocked
  prior_cleanup_recovery_reason='prior validated run_root could not be removed'
  return 1
}

write_cleanup_receipt() {
  jq -n \
    --arg state "$cleanup_state" \
    --arg reason "$cleanup_reason" \
    --arg targetPath "$cleanup_target_path" \
    --arg targetParent "$cleanup_target_parent" \
    --arg targetBasename "$cleanup_target_basename" \
    --arg targetIdentity "$cleanup_target_identity" \
    --arg priorState "$prior_cleanup_recovery_state" \
    --arg priorReason "$prior_cleanup_recovery_reason" \
    --argjson removed "$cleanup_removed" \
    --argjson validated "$cleanup_validated" \
    '{schemaVersion:1,kind:"run_root_cleanup",state:$state,removed:$removed,validated:$validated,reason:(if $reason == "" then null else $reason end),target:(if $targetPath == "" then null else {path:$targetPath,parent:$targetParent,basename:$targetBasename,deviceInode:$targetIdentity} end),priorTargetRecovery:{state:$priorState,reason:(if $priorReason == "" then null else $priorReason end)}}' \
    > "$output/cleanup.json"
}

write_unbound_failure_receipt() {
  local phase="${1:-${current_phase:-unknown}}"
  local kind="${2:-${AI_COCKPIT_ACCEPTANCE_FAILURE_KIND:-runner}}"
  local code="${3:-acceptance_phase_failed_before_identity}"
  local diagnostic="${4:-${failure_reason:-command exited with status ${exit_code:-1}}}"
  local identity_state=unavailable
  [[ -f "${phase_identity:-}" && ! -L "${phase_identity:-}" ]] && identity_state=available
  local recovery_strategy=retry_current_phase
  case "$kind" in
    input_changed|identity_mismatch|validation|scope_changed|authority_changed|base_changed|already_published)
      recovery_strategy=blocked_until_inputs_repaired
      ;;
  esac
  jq -n \
    --arg phase "$phase" \
    --arg kind "$kind" \
    --arg code "$code" \
    --arg diagnostic "$diagnostic" \
    --arg strategy "$recovery_strategy" \
    --arg identityState "$identity_state" \
    --argjson attempt "${ACCEPTANCE_ATTEMPT:-1}" \
    '{schemaVersion:1,kind:"release_phase_failure",status:"failed",phase:$phase,failureKind:$kind,failureCode:$code,diagnostic:$diagnostic,attempt:$attempt,identityState:$identityState,recovery:{strategy:$strategy,phase:$phase},persistedPhaseReceipt:false}' \
    > "$output/phase-failure.json"
}

validate_close_decision() {
  local work_item_id="$1"
  local decision_path="$adopter_root/.ai/decisions/$work_item_id.close.json"
  local artifact_path="$output/work-items/$work_item_id.close.json"
  local binding_path="$output/work-items/$work_item_id.close.binding.json"

  mkdir -p "$output/work-items"
  [[ "$adopter_repository_id" =~ ^sha256:[0-9a-f]{64}$ ]] || die 'adopter repository identity is missing or malformed for close decision binding'
  [[ -f "$decision_path" && ! -L "$decision_path" ]] || die "missing or symlinked close decision: $decision_path"
  jq -e \
    --arg workItemId "$work_item_id" \
    '(
      .workItemId == $workItemId
      and .state == "closed"
      and .decisionState == "confirmed"
      and .humanDecision == "approved"
      and (.structuredDecision | type == "object")
      and (.structuredDecision.decision == "approved")
      and (.structuredDecision.actor | type == "string" and length > 0)
      and (.structuredDecision.authoritySource | type == "string" and length > 0)
      and (.structuredDecision.reason | type == "string" and length > 0)
      and (.structuredDecision.decidedAt | type == "string" and length > 0)
      and (.structuredDecision.resumeCondition | type == "string" and length > 0)
      and (.structuredDecision.evidenceRefs | type == "array" and length > 0)
      and (.structuredDecision.policyRefs | type == "array" and length > 0)
    )' "$decision_path" >/dev/null || die "close decision is incomplete for Work Item $work_item_id"

  cp "$decision_path" "$artifact_path"
  close_decision_work_item="$work_item_id"
  close_decision_repository_id="$adopter_repository_id"
  close_decision_digest="sha256:$(sha256_file "$decision_path")"
  close_decision_path="work-items/$work_item_id.close.json"
  close_decision_validated=true
  jq -n \
    --arg workItemId "$work_item_id" \
    --arg repositoryId "$adopter_repository_id" \
    --arg decisionPath ".ai/decisions/$work_item_id.close.json" \
    --arg artifactPath "$close_decision_path" \
    --arg decisionDigest "$close_decision_digest" \
    '{schemaVersion:1,validated:true,workItemId:$workItemId,repositoryId:$repositoryId,decisionPath:$decisionPath,artifactPath:$artifactPath,decisionDigest:$decisionDigest}' \
    > "$binding_path"
}

update_acceptance_cleanup() {
  local updated="$output/.acceptance.json.cleanup.tmp"
  if jq \
    --arg state "$cleanup_state" \
    --arg reason "$cleanup_reason" \
    '.cleanupState = $state
     | .cleanupError = (if $state == "failed" then $reason else null end)
     | if $state == "failed" then
         .adopterAcceptance = "failed"
         | .failureReason = (if (.failureReason == null or .failureReason == "") then ("cleanup failed: " + $reason) else .failureReason end)
       else . end' \
    "$output/acceptance.json" > "$updated" && mv -f -- "$updated" "$output/acceptance.json"; then
    return 0
  fi
  printf 'adopter acceptance cleanup warning: acceptance cleanup metadata could not be updated\n' >&2
  return 1
}

finalize() {
  local exit_code=$?
  set +e
  local acceptance_write_result=0
  local initial_sums_result=0
  local cleanup_result=0
  local acceptance_update_result=0
  local cleanup_receipt_result=0
  local close_result=0
  local final_sums_result=0
  finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [[ "$exit_code" -eq 0 ]]; then
    overall_state=passed
  else
    overall_state=failed
    [[ -n "$failure_reason" ]] || failure_reason="command exited with status $exit_code"
  fi
  if [[ "$exit_code" -ne 0 ]]; then
    record_phase_failure_from_plan || exit_code=1
  fi
  local steps='[]'
  if [[ -n "$steps_jsonl" && -s "$steps_jsonl" ]]; then
    steps="$(jq -s '.' "$steps_jsonl")"
  fi
  jq -n \
    --arg startedAt "$started_at" \
    --arg finishedAt "$finished_at" \
    --arg state "$overall_state" \
    --arg releasePublished "$release_published" \
    --arg stagedCandidate "$staged_candidate" \
    --arg repository "$repository" \
    --arg tag "$tag" \
    --arg target "$target" \
    --arg runtimeVersion "$runtime_version" \
    --arg runtimeDigest "$runtime_digest" \
    --arg rustToolchain "$rustup_toolchain" \
    --arg repositoryId "$repository_id" \
    --arg sourceRepositoryId "$source_repository_id" \
    --arg closeDecisionWorkItem "$close_decision_work_item" \
    --arg closeDecisionRepositoryId "$close_decision_repository_id" \
    --arg closeDecisionDigest "$close_decision_digest" \
    --arg closeDecisionPath "$close_decision_path" \
    --argjson closeDecisionValidated "$close_decision_validated" \
    --arg failureReason "$failure_reason" \
    --argjson steps "$steps" \
    '{
      schemaVersion: 1,
      startedAt: $startedAt,
      finishedAt: $finishedAt,
      releasePublished: ($releasePublished == "true"),
      stagedCandidate: ($stagedCandidate == "true"),
      adopterAcceptance: $state,
      repository: $repository,
      tag: $tag,
      target: $target,
      rustToolchain: (if $rustToolchain == "" then null else $rustToolchain end),
      runtimeVersion: (if $runtimeVersion == "" then null else $runtimeVersion end),
      runtimeDigest: (if $runtimeDigest == "" then null else $runtimeDigest end),
      repositoryId: (if $repositoryId == "" then null else $repositoryId end),
      sourceRepositoryId: (if $sourceRepositoryId == "" then null else $sourceRepositoryId end),
      closeDecision: {
        validated: $closeDecisionValidated,
        workItemId: (if $closeDecisionWorkItem == "" then null else $closeDecisionWorkItem end),
        repositoryId: (if $closeDecisionRepositoryId == "" then null else $closeDecisionRepositoryId end),
        artifactPath: (if $closeDecisionPath == "" then null else $closeDecisionPath end),
        digest: (if $closeDecisionDigest == "" then null else $closeDecisionDigest end)
      },
      cleanupState: "pending",
      cleanupError: null,
      steps: $steps,
      failureReason: (if $failureReason == "" then null else $failureReason end)
    }' > "$output/acceptance.json" || acceptance_write_result=$?
  if [[ "$acceptance_write_result" -ne 0 ]]; then
    exit_code=1
    failure_reason="acceptance receipt write failed (status $acceptance_write_result)"
  fi
  write_sums || initial_sums_result=$?
  if [[ "$initial_sums_result" -ne 0 ]]; then
    exit_code=1
    [[ -n "$failure_reason" ]] || failure_reason="initial checksum receipt write failed (status $initial_sums_result)"
  fi
  cleanup_run_root
  cleanup_result=$?
  if [[ "$cleanup_result" -ne 0 ]]; then
    failure_reason="cleanup failed: $cleanup_reason"
    record_phase_failure_from_plan close cleanup || exit_code=1
  fi
  update_acceptance_cleanup
  acceptance_update_result=$?
  write_cleanup_receipt
  cleanup_receipt_result=$?
  record_close_after_cleanup || close_result=$?
  if [[ "$close_result" -ne 0 ]]; then
    exit_code="$close_result"
    [[ -n "$failure_reason" ]] || failure_reason='close phase receipt could not be persisted'
    if ! mark_acceptance_failed "$failure_reason"; then
      printf 'adopter acceptance failure metadata could not be updated\n' >&2
    fi
  fi
  write_sums
  final_sums_result=$?
  if [[ "$cleanup_state" == failed ]]; then
    printf 'adopter acceptance cleanup failed: %s\n' "$cleanup_reason" >&2
    [[ "$exit_code" -ne 0 ]] || exit_code=1
  fi
  [[ "$acceptance_update_result" -eq 0 ]] || exit_code=1
  [[ "$cleanup_receipt_result" -eq 0 ]] || exit_code=1
  [[ "$close_result" -eq 0 ]] || exit_code=1
  [[ "$final_sums_result" -eq 0 ]] || exit_code=1
  if [[ "$cleanup_result" -ne 0 ]]; then exit_code=1; fi
  exit "$exit_code"
}
trap finalize EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

if [[ "$resume" == true ]]; then
  if [[ ! -f "$phase_identity" || -L "$phase_identity" ]]; then
    failure_reason='resume phase identity is missing or symlinked; refusing to create an empty recovery plan'
    write_unbound_failure_receipt prepare validation resume_identity_missing "$failure_reason"
    exit 1
  fi
  if [[ ! -f "$phase_receipts" || -L "$phase_receipts" ]]; then
    failure_reason='resume phase receipt is missing or symlinked; refusing to create an empty recovery plan'
    write_unbound_failure_receipt prepare validation resume_receipt_missing "$failure_reason"
    exit 1
  fi
fi

if ! recover_prior_cleanup; then
  failure_reason="$prior_cleanup_recovery_reason"
  if ! write_unbound_failure_receipt close cleanup cleanup_target_unrecoverable "$failure_reason"; then
    printf 'adopter acceptance failure receipt could not be persisted: %s\n' "$failure_reason" >&2
  fi
  exit 1
fi
run_root="$(mktemp -d "$run_parent/ai-cockpit-adopter-acceptance.XXXXXX")"
run_root_identity="$(path_identity "$run_root")"
cleanup_target_path="$run_root"
cleanup_target_parent="$run_parent"
cleanup_target_basename="${run_root##*/}"
cleanup_target_identity="$run_root_identity"
runtime_root="$run_root/runtime"
adopter_root="$run_root/adopter"
isolated_home="$run_root/home"
isolated_xdg="$run_root/xdg-config"
isolated_tmp="$run_root/tmp"
isolated_cargo="$run_root/cargo-home"
download_root="$run_root/downloads"
mkdir -p "$runtime_root" "$isolated_home" "$isolated_xdg" "$isolated_tmp" "$isolated_cargo" "$download_root"
steps_jsonl="$run_root/steps.jsonl"
: > "$steps_jsonl"

if rustup_home="$(printenv RUSTUP_HOME)"; then :; fi
if [[ -z "$rustup_home" ]] && command -v rustup >/dev/null 2>&1; then
  if rustup_home="$(rustup show home 2>/dev/null)"; then :; fi
fi
if command -v rustup >/dev/null 2>&1; then
  set +e
  rustup_toolchain="$(rustup show active-toolchain 2>/dev/null | awk 'NR == 1 {print $1}')"
  rustup_toolchain_status=$?
  set -e
  if [[ "$rustup_toolchain_status" -ne 0 ]]; then rustup_toolchain=''; fi
fi
[[ -n "$rustup_home" && -d "$rustup_home" ]] || die 'RUSTUP_HOME could not be resolved; refusing implicit toolchain download'
[[ -n "$rustup_toolchain" ]] || die 'active Rust toolchain could not be resolved; refusing implicit toolchain download'

capture_runtime() {
  local evidence_name=$1
  shift
  local evidence_stem
  evidence_stem="$(printf '%s' "$evidence_name" | sed 's/\.json$//')"
  local stderr_path="$run_root/$evidence_stem.stderr"
  set +e
  env -i \
    HOME="$isolated_home" \
    XDG_CONFIG_HOME="$isolated_xdg" \
    TMPDIR="$isolated_tmp" \
    CARGO_HOME="$isolated_cargo" \
    RUSTUP_HOME="$rustup_home" \
    RUSTUP_TOOLCHAIN="$rustup_toolchain" \
    PATH="$PATH" \
    LANG=C \
    LC_ALL=C \
    GIT_CONFIG_NOSYSTEM=1 \
    GIT_CONFIG_GLOBAL=/dev/null \
    "$runtime_bin" "$@" > "$output/$evidence_name" 2> "$stderr_path"
  local result=$?
  set -e
  if [[ "$result" -ne 0 ]]; then
    if cp "$stderr_path" "$output/$evidence_stem.stderr" 2>/dev/null; then :; fi
    record_step "$evidence_name" failed "runtime command exited with status $result"
    failure_reason="$evidence_name failed"
    return "$result"
  fi
  mark_passed "$evidence_name"
}

phase_action() {
  local phase="$1"
  jq -er --arg phase "$phase" '.actions[] | select(.phase == $phase) | .action' "$phase_plan"
}

phase_action_strategy() {
  local phase="$1"
  jq -er --arg phase "$phase" '.actions[] | select(.phase == $phase) | (.strategy // "")' "$phase_plan"
}

phase_action_should_run() {
  local phase="$1"
  local action strategy
  action="$(phase_action "$phase")" || die "acceptance plan has no action for phase $phase"
  case "$action" in
    run)
      return 0
      ;;
    retry)
      strategy="$(phase_action_strategy "$phase")" || die "acceptance plan has no retry strategy for phase $phase"
      case "$strategy" in
        retry_current_phase|restart_from_phase) return 0 ;;
        retry_cleanup_only) die "acceptance phase $phase requires cleanup-only recovery" ;;
        require_successor|block) die "acceptance phase $phase is blocked until its recovery boundary changes" ;;
        *) die "acceptance phase $phase has unknown retry strategy: $strategy" ;;
      esac
      ;;
    reuse)
      return 1
      ;;
    not_applicable)
      return 1
      ;;
    blocked)
      die "acceptance phase $phase is blocked by an invalid prerequisite"
      ;;
    *)
      die "acceptance phase $phase has unknown action: $action"
      ;;
  esac
}

refresh_phase_plan() {
  [[ -n "${COCKPIT_RELEASE_BIN:-}" && -x "$COCKPIT_RELEASE_BIN" ]] || die 'COCKPIT_RELEASE_BIN must point to the prebuilt cockpit-release helper'
  "$COCKPIT_RELEASE_BIN" acceptance-plan \
    --scope "$acceptance_scope" \
    --identity "$phase_identity" \
    --receipts "$phase_receipts" \
    --output "$phase_plan" || die 'identity-bound acceptance plan rejected the current receipt store'
}

record_phase_failure_from_plan() {
  local phase="${1:-}" kind="${2:-${AI_COCKPIT_ACCEPTANCE_FAILURE_KIND:-runner}}"
  if [[ ! -f "$phase_plan" || ! -f "$phase_identity" ]]; then
    write_unbound_failure_receipt "${phase:-${current_phase:-unknown}}" "$kind" \
      acceptance_phase_failed_before_identity \
      "${failure_reason:-failure occurred before an identity-bound phase plan was persisted}"
    return $?
  fi
  if [[ -z "$phase" ]]; then
    phase="$(jq -er '.actions[] | select(.action == "run" or .action == "retry") | .phase' "$phase_plan" 2>/dev/null | head -n 1)" || {
      failure_reason='identity-bound phase plan is malformed and could not identify the failed phase'
      if ! write_unbound_failure_receipt "${current_phase:-unknown}" validation acceptance_phase_plan_invalid "$failure_reason"; then
        printf 'adopter acceptance failure receipt could not be persisted: %s\n' "$failure_reason" >&2
      fi
      return 1
    }
  fi
  if [[ -z "$phase" ]]; then
    failure_reason='identity-bound phase plan contains no runnable phase'
    if ! write_unbound_failure_receipt "${current_phase:-unknown}" validation acceptance_phase_plan_empty "$failure_reason"; then
      printf 'adopter acceptance failure receipt could not be persisted: %s\n' "$failure_reason" >&2
    fi
    return 1
  fi
  case "$exit_code" in
    130|143) kind=interruption ;;
  esac
  if ! "$COCKPIT_RELEASE_BIN" acceptance-record-failure \
    --scope "$acceptance_scope" \
    --identity "$phase_identity" \
    --receipts "$phase_receipts" \
    --phase "$phase" \
    --failure-kind "$kind" \
    --failure-code acceptance_phase_failed \
    --diagnostic "${failure_reason:-command exited with status $exit_code}" \
    --attempt "${ACCEPTANCE_ATTEMPT:-1}" >/dev/null; then
    failure_reason="could not persist identity-bound failure receipt for $phase"
    if ! write_unbound_failure_receipt "$phase" "$kind" acceptance_phase_failure_persist_failed "$failure_reason"; then
      printf 'adopter acceptance failure receipt could not be persisted: %s\n' "$failure_reason" >&2
    fi
    return 1
  fi
  return 0
}

record_phase_success() {
  local phase="$1"
  local evidence="$2"
  current_phase="$phase"
  if ! phase_action_should_run "$phase"; then
      mark_passed "phase-$phase" 'reused identity-bound phase receipt'
      return 0
  fi
  if [[ "${AI_COCKPIT_ACCEPTANCE_FAIL_BEFORE_PHASE:-}" == "$phase" ]]; then
    failure_reason="injected failure before $phase execution"
    exit 96
  fi
  "$COCKPIT_RELEASE_BIN" acceptance-record \
    --scope "$acceptance_scope" \
    --identity "$phase_identity" \
    --receipts "$phase_receipts" \
    --phase "$phase" \
    --evidence "$evidence" \
    --attempt "${ACCEPTANCE_ATTEMPT:-1}" >/dev/null || die "could not persist identity-bound phase receipt: $phase"
  refresh_phase_plan
  if [[ "${AI_COCKPIT_ACCEPTANCE_FAIL_AFTER_PHASE:-}" == "$phase" ]]; then
    failure_reason="injected failure after $phase receipt"
    exit 97
  fi
}

validate_persisted_acceptance() {
  [[ -n "$acceptance_phase" && -n "$acceptance_evidence" ]] || die 'acceptance resume scope is not initialized'
  [[ -f "$acceptance_evidence" && ! -L "$acceptance_evidence" ]] || die "persisted acceptance evidence is missing or symlinked: $acceptance_evidence"
  jq -e \
    --arg phase "$acceptance_phase" \
    --arg evidence "$acceptance_evidence" \
    '[.results[] | select(.phase == $phase and .status == "succeeded" and (.evidence.path == $evidence or ((.evidence.path | split("/") | last) == ($evidence | split("/") | last))))] | length == 1' \
    "$phase_receipts" >/dev/null || die "persisted $acceptance_phase receipt does not bind its acceptance evidence"
  jq -e '.schemaVersion == 2 and .sourceUnchanged == true and .roots.HOME.unchanged == true and .roots.XDG_CONFIG_HOME.unchanged == true and .repositoryIsolation == true' \
    "$acceptance_evidence" >/dev/null || die "persisted $acceptance_phase evidence failed its isolation/source checks"
}

mark_acceptance_failed() {
  local reason="$1" updated="$output/.acceptance.json.failure.tmp"
  [[ -f "$output/acceptance.json" ]] || return 1
  jq --arg reason "$reason" '.adopterAcceptance = "failed" | .failureReason = $reason' \
    "$output/acceptance.json" > "$updated" && mv -f -- "$updated" "$output/acceptance.json"
}

record_close_after_cleanup() {
  [[ "$close_ready" == true && "$cleanup_state" == passed ]] || return 0
  [[ -f "$output/cleanup.json" ]] || {
    failure_reason='close receipt prerequisites are missing'
    return 1
  }
  local action strategy
  local close_plan="$output/close-phase-plan.json"
  "$COCKPIT_RELEASE_BIN" acceptance-plan \
    --scope "$acceptance_scope" \
    --identity "$phase_identity" \
    --receipts "$phase_receipts" \
    --output "$close_plan" >/dev/null || {
      failure_reason='close phase plan rejected the persisted receipt store'
      return 1
    }
  action="$(jq -er '.actions[] | select(.phase == "close") | .action' "$close_plan" 2>/dev/null)" || {
    failure_reason='close phase is absent from the scoped acceptance plan'
    return 1
  }
  case "$action" in
    reuse|not_applicable) return 0 ;;
    blocked)
      failure_reason='close phase is blocked by an unrecovered prerequisite'
      return 1
      ;;
    retry)
      strategy="$(jq -er '.actions[] | select(.phase == "close") | .strategy' "$close_plan" 2>/dev/null)" || {
        failure_reason='close retry strategy is missing'
        return 1
      }
      case "$strategy" in
        retry_cleanup_only) ;;
        retry_current_phase|restart_from_phase)
          failure_reason="close phase requires $strategy before it can be recorded"
          return 1
          ;;
        require_successor|block|*)
          failure_reason="close phase recovery strategy is not executable: $strategy"
          return 1
          ;;
      esac
      ;;
    run) ;;
    *)
      failure_reason="close phase has unsupported action: $action"
      return 1
      ;;
  esac
  "$COCKPIT_RELEASE_BIN" acceptance-record \
    --scope "$acceptance_scope" \
    --identity "$phase_identity" \
    --receipts "$phase_receipts" \
    --phase close \
    --evidence "$output/cleanup.json" \
    --attempt "${ACCEPTANCE_ATTEMPT:-1}" >/dev/null || {
      failure_reason='could not persist identity-bound close phase receipt'
      return 1
    }
  if [[ "${AI_COCKPIT_ACCEPTANCE_FAIL_AFTER_PHASE:-}" == close ]]; then
    failure_reason='injected failure after close receipt'
    return 97
  fi
}

count_acceptance_command() {
  local command_name="$1"
  [[ -n "${AI_COCKPIT_ACCEPTANCE_COMMAND_COUNTER:-}" ]] || return 0
  printf '%s\n' "$command_name" >> "$AI_COCKPIT_ACCEPTANCE_COMMAND_COUNTER"
}

version="$(printf '%s' "$tag" | sed 's/^v//')"
archive_name="ai-cockpit-$tag-$target.$archive_extension"
manifest_name=release-manifest.json
sums_name=SHA256SUMS
archive_path="$download_root/$archive_name"
manifest_path="$download_root/$manifest_name"
sums_path="$download_root/$sums_name"
release_url=''
release_api=''
formula_url=''
publish_formula_digest=''
publish_handoff_digest=''
release_source_commit=''
cache_archive="$resume_cache/$archive_name"
cache_manifest="$resume_cache/$manifest_name"
cache_sums="$resume_cache/$sums_name"
if [[ "$resume" == true ]]; then
  [[ -f "$cache_archive" && ! -L "$cache_archive" ]] || die "resume cache is missing archive: $cache_archive"
  [[ -f "$cache_manifest" && ! -L "$cache_manifest" ]] || die "resume cache is missing manifest: $cache_manifest"
  [[ -f "$cache_sums" && ! -L "$cache_sums" ]] || die "resume cache is missing checksums: $cache_sums"
  cp "$cache_archive" "$archive_path"
  cp "$cache_manifest" "$manifest_path"
  cp "$cache_sums" "$sums_path"
fi
if [[ -n "$candidate_dir" ]]; then
  staged_candidate=true
  for candidate_file in "$archive_name" "$manifest_name" "$sums_name"; do
    [[ -f "$candidate_dir/$candidate_file" && ! -L "$candidate_dir/$candidate_file" ]] || die "staged candidate file is missing or symlinked: $candidate_file"
  done
  if [[ "$resume" != true ]]; then
    count_acceptance_command candidate-artifact-copy
    cp "$candidate_dir/$archive_name" "$archive_path"
    cp "$candidate_dir/$manifest_name" "$manifest_path"
    cp "$candidate_dir/$sums_name" "$sums_path"
  fi
  source_revision="$(git -C "$source_repo" rev-parse 'HEAD^{commit}')"
  [[ "$(jq -er '.commit' "$manifest_path")" == "$source_revision" ]] || die 'staged candidate commit does not match source checkout HEAD'
  archive_url="workflow-artifact:ai-cockpit-candidate/$archive_name"
  mark_passed candidate-fetch
else
  release_url="https://github.com/$repository/releases/tag/$tag"
  api_url="https://api.github.com/repos/$repository/releases/tags/$tag"
  release_api="$output/release.json"
  if [[ "$resume" == true && -f "$release_api" && ! -L "$release_api" ]]; then
    release_published=true
    mark_passed release-fetch 'reused cached public Release metadata'
  elif ! github_api_get "$api_url" "$release_api"; then
    record_step release-fetch failed 'public Release API request failed'
    failure_reason='public Release API request failed'
    exit 1
  fi
  if ! jq -e --arg tag "$tag" '.tag_name == $tag and (.draft == false) and (.prerelease == false)' "$release_api" >/dev/null; then
    record_step release-fetch failed 'Release is missing, draft, prerelease, or tag-mismatched'
    failure_reason='public Release is not a published immutable tag'
    exit 1
  fi
  release_published=true
  mark_passed release-fetch
  archive_url="$(jq -er --arg name "$archive_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
  manifest_url="$(jq -er --arg name "$manifest_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
  sums_url="$(jq -er --arg name "$sums_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
  [[ "$archive_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die 'archive URL is outside the requested public Release'
  [[ "$manifest_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die 'manifest URL is outside the requested public Release'
  [[ "$sums_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die 'checksum URL is outside the requested public Release'
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$archive_url" -o "$archive_path"
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$manifest_url" -o "$manifest_path"
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$sums_url" -o "$sums_path"
fi
release_source_commit="$(jq -er '.commit' "$manifest_path")"
source_revision="$(git -C "$source_repo" rev-parse 'HEAD^{commit}')"
[[ "$release_source_commit" == "$source_revision" ]] || die 'release manifest commit does not match source checkout HEAD'
if [[ -n "$release_api" && -z "$archive_url" ]]; then
  archive_url="$(jq -er --arg name "$archive_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
  manifest_url="$(jq -er --arg name "$manifest_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
  sums_url="$(jq -er --arg name "$sums_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
fi
if [[ -n "$publish_handoff" ]]; then
  [[ -n "$release_api" ]] || die 'publish handoff requires public Release metadata'
  formula_url="$(jq -er '.assets[] | select(.name == "ai-cockpit.rb") | .browser_download_url' "$release_api")"
  [[ "$formula_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die 'Formula URL is outside the requested public Release'
  formula_path="$output/ai-cockpit.rb"
  formula_cache="$resume_cache/ai-cockpit.rb"
  if [[ "$resume" == true && -f "$formula_cache" && ! -L "$formula_cache" ]]; then
    cp "$formula_cache" "$formula_path"
  else
    curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$formula_url" -o "$formula_path"
    mkdir -p "$resume_cache"
    cp "$formula_path" "$formula_cache"
  fi
  publish_formula_digest="$(sha256_file "$formula_path")"
  publish_handoff_digest="sha256:$(sha256_file "$publish_handoff")"
  "$COCKPIT_RELEASE_BIN" validate-handoff \
    --handoff "$publish_handoff" \
    --tag "$tag" \
    --commit "$(jq -er '.commit' "$manifest_path")" \
    --provider-release-id "$(jq -er '.id' "$release_api")" \
    --manifest-sha256 "$(sha256_file "$manifest_path")" \
    --formula-sha256 "$publish_formula_digest" >/dev/null || die 'published handoff does not bind the public Release assets'
  mark_passed publish-handoff
fi
cp "$manifest_path" "$output/release-manifest.json"
cp "$sums_path" "$output/SHA256SUMS.release"
manifest_archive_digest="$(jq -er --arg target "$target" '.artifacts[] | select(.target == $target) | .archive.sha256' "$manifest_path")"
sums_archive_digest="$(awk -v name="$archive_name" '$2 == name {print $1}' "$sums_path")"
actual_archive_digest="$(sha256_file "$archive_path")"
[[ "$manifest_archive_digest" == "$actual_archive_digest" ]] || die 'archive digest does not match release manifest'
[[ "$sums_archive_digest" == "$actual_archive_digest" ]] || die 'archive digest does not match SHA256SUMS'
[[ "$(jq -er '.version' "$manifest_path")" == "$version" ]] || die 'manifest version does not match tag'
[[ "$(jq -er '.tag' "$manifest_path")" == "$tag" ]] || die 'manifest tag does not match requested tag'
mkdir -p "$resume_cache"
cp "$archive_path" "$cache_archive"
cp "$manifest_path" "$cache_manifest"
cp "$sums_path" "$cache_sums"
mark_passed release-download

if [[ "$archive_extension" == tar.gz ]]; then
  tar -xzf "$archive_path" -C "$runtime_root"
else
  unzip -q "$archive_path" -d "$runtime_root"
fi
runtime_bin="$runtime_root/ai-cockpit"
if [[ "$archive_extension" == zip ]]; then runtime_bin="$runtime_root/ai-cockpit.exe"; fi
[[ -f "$runtime_bin" && -x "$runtime_bin" ]] || die 'accepted archive did not contain an executable Runtime'
runtime_version="$("$runtime_bin" --version | awk '{print $2}')"
runtime_digest="sha256:$(sha256_file "$runtime_bin")"
export AI_COCKPIT_ISOLATION_BIN="$runtime_bin"
[[ "$runtime_version" == "$version" ]] || die 'accepted Runtime version does not match candidate tag'
runtime_platform="$(uname -s)-$(uname -m)"
jq -n \
  --arg tag "$tag" \
  --arg version "$version" \
  --arg target "$target" \
  --arg platform "$runtime_platform" \
  --arg archive "$archive_name" \
  --arg archiveDigest "sha256:$actual_archive_digest" \
  --arg binaryDigest "$runtime_digest" \
  --arg downloadSource "$archive_url" \
  --arg releaseUrl "$release_url" \
  --arg manifestDigest "sha256:$(sha256_file "$manifest_path")" \
  --argjson releasePublished "$release_published" \
  --argjson stagedCandidate "$staged_candidate" \
  '{schemaVersion:1,tag:$tag,version:$version,target:$target,platform:$platform,archive:$archive,archiveDigest:$archiveDigest,binaryDigest:$binaryDigest,downloadSource:$downloadSource,releaseUrl:(if $releaseUrl == "" then null else $releaseUrl end),manifestDigest:$manifestDigest,releasePublished:$releasePublished,stagedCandidate:$stagedCandidate}' > "$output/runtime.json"
mark_passed runtime-pin

jq -n \
  --arg repository "$source_repository_id" \
  --arg commit "$release_source_commit" \
  --arg lock "sha256:$(sha256_file "$source_repo/Cargo.lock")" \
  --arg version "$version" \
  --arg tag "$tag" \
  --arg manifest "sha256:$(sha256_file "$manifest_path")" \
  --arg archive "$archive_name" \
  --arg archiveDigest "$manifest_archive_digest" \
  --arg formulaDigest "$publish_formula_digest" \
  --arg handoffDigest "$publish_handoff_digest" \
  --arg runtimeVersion "$runtime_version" \
  --arg runtimeDigest "$runtime_digest" \
  --arg target "$target" \
  --arg output "$output" \
  '{source:{repository:$repository,commit:$commit,cargoLockDigest:$lock},candidate:{version:$version,tag:$tag,manifestDigest:$manifest,assets:({($archive):$archiveDigest} + (if $formulaDigest == "" then {} else {"ai-cockpit.rb":$formulaDigest} end) + (if $handoffDigest == "" then {} else {"homebrew-handoff.json":$handoffDigest} end))},previous:null,runtime:{version:$runtimeVersion,digest:$runtimeDigest},target:$target,isolation:{home:($output+"/.resume-scope/home"),xdgConfigHome:($output+"/.resume-scope/xdg"),tmp:($output+"/.resume-scope/tmp"),cargoHome:($output+"/.resume-scope/cargo")}}' \
  > "$phase_identity"
if [[ -n "$candidate_dir" ]]; then
  acceptance_scope=candidate
  acceptance_phase=candidate_acceptance
else
  acceptance_scope=public
  acceptance_phase=public_acceptance
fi
acceptance_evidence="$output/isolation.json"
phase_plan="$run_root/phase-plan.json"
refresh_phase_plan
record_phase_success prepare "$output/release-manifest.json"
source_before_status="$(git -C "$source_repo" status --porcelain=v1)"
manifest_source_checkout "$source_repo" "$output" "$run_root/source-before.manifest"
if [[ -e "$source_repo/.ai" ]]; then source_ai_state=present; else source_ai_state=absent; fi
jq -n \
  --arg sourceRepository "$source_repo" \
  --arg sourceRepositoryId "$source_repository_id" \
  --arg sourceCommit "$release_source_commit" \
  --arg sourceStatus "$source_before_status" \
  --arg sourceManifestDigest "sha256:$(sha256_file "$run_root/source-before.manifest")" \
  --arg runtimeVersion "$runtime_version" \
  --arg runtimeDigest "$runtime_digest" \
  '{schemaVersion:1,phase:"source_verification_build",source:{repository:$sourceRepository,repositoryId:$sourceRepositoryId,commit:$sourceCommit,status:$sourceStatus,manifestDigest:$sourceManifestDigest},runtime:{version:$runtimeVersion,digest:$runtimeDigest}}' \
  > "$output/source-verification.json"
record_phase_success source_verification_build "$output/source-verification.json"
if [[ "$acceptance_scope" == public ]]; then
  record_phase_success publish "$output/runtime.json"
fi
if ! phase_action_should_run "$acceptance_phase"; then
  record_phase_success "$acceptance_phase" "$acceptance_evidence"
  validate_persisted_acceptance
  close_ready=true
  overall_state=passed
  failure_reason=''
  exit 0
fi
env -i HOME="$isolated_home" XDG_CONFIG_HOME="$isolated_xdg" TMPDIR="$isolated_tmp" CARGO_HOME="$isolated_cargo" RUSTUP_HOME="$rustup_home" RUSTUP_TOOLCHAIN="$rustup_toolchain" PATH="$PATH" LANG=C LC_ALL=C cargo new --lib --vcs none "$adopter_root" >/dev/null
printf 'target/\n' > "$adopter_root/.gitignore"
env -i HOME="$isolated_home" XDG_CONFIG_HOME="$isolated_xdg" TMPDIR="$isolated_tmp" CARGO_HOME="$isolated_cargo" RUSTUP_HOME="$rustup_home" RUSTUP_TOOLCHAIN="$rustup_toolchain" PATH="$PATH" LANG=C LC_ALL=C cargo generate-lockfile --manifest-path "$adopter_root/Cargo.toml" >/dev/null
git -C "$adopter_root" init -q
git -C "$adopter_root" config user.name 'AI Cockpit Release Acceptance'
git -C "$adopter_root" config user.email 'ai-cockpit-release-acceptance@example.invalid'
# Prevent detached background gc/maintenance from racing exact checkout
# removal below. This is repository-local and does not touch user config.
git -C "$adopter_root" config gc.auto 0
git -C "$adopter_root" config maintenance.auto false
git -C "$adopter_root" add .
git -C "$adopter_root" commit -qm 'initial adopter scaffold'
mark_passed adopter-scaffold

# Cargo scaffolding is allowed to warm the isolated dependency cache. Capture
# all isolated roots immediately before Runtime operations, after scaffolding.
manifest_tree "$isolated_home" "$run_root/home-before.manifest"
manifest_tree "$isolated_xdg" "$run_root/xdg-before.manifest"
manifest_tree "$isolated_tmp" "$run_root/tmp-before.manifest"
manifest_tree "$isolated_cargo" "$run_root/cargo-before.manifest"

capture_runtime attach.json attach --repo "$adopter_root"
capture_runtime inspect.json inspect --repo "$adopter_root"
inspect_runtime_version="$(jq -er '.runtimeVersion' "$output/inspect.json")"
inspect_runtime_digest="$(jq -er '.runtimeDigest' "$output/inspect.json")"
[[ "$inspect_runtime_version" == "$runtime_version" && "$inspect_runtime_digest" == "$runtime_digest" ]] || die 'inspect Runtime identity does not match downloaded binary'
mark_passed inspect-runtime-identity
# Agent adapter state is created only after attach, then committed before any
# Work Item is created.  This keeps the adopter repository clean at the
# lifecycle boundary while preserving the explicit adapter-install contract.
: > "$adopter_root/AGENTS.md"
capture_runtime profile-confirm.json profile confirm --repo "$adopter_root" --program cargo --args test,--workspace
capture_runtime agent-list.json agent list --repo "$adopter_root"
capture_runtime agent-install.json agent install --repo "$adopter_root" --provider auto
capture_runtime agent-doctor.json agent doctor --repo "$adopter_root" --json
jq -e '.state == "VERIFIED" and .repositoryId != null and (.problems | length == 0)' "$output/agent-doctor.json" >/dev/null || die 'Agent doctor did not verify the fresh adopter'
adopter_repository_id="$(jq -er '.repositoryId' "$output/agent-doctor.json")"
repository_id="$adopter_repository_id"
mark_passed agent-doctor-assertion

git -C "$adopter_root" add .
git -C "$adopter_root" commit -qm 'attach adopter governance state'

capture_runtime first-adopter-smoke.json work-item new --repo "$adopter_root" --id first-adopter-smoke --mode code
first_smoke_contract="$adopter_root/.ai/work-items/active/first-adopter-smoke.contract.json"
[[ -f "$first_smoke_contract" ]] || die 'first-adopter-smoke contract was not created'
mkdir -p "$output/work-items"
cp "$first_smoke_contract" "$output/work-items/first-adopter-smoke.contract.json"
jq -e '.state == "not_ready" and .intent == "" and (.scope | length == 0) and (.acceptanceCriteria | length == 0) and .authority == "unknown"' "$first_smoke_contract" >/dev/null || die 'first-adopter-smoke was not preserved as not_ready'
mark_passed first-adopter-smoke-assertion

git -C "$adopter_root" add .
git -C "$adopter_root" commit -qm 'create first adopter Work Item scaffold'

capture_runtime verify-first.json verify --repo "$adopter_root" --workers 1
jq -e '.passed == true and .nodesExecuted >= 1 and .nodesReused == 0 and .processesSpawned >= 1' "$output/verify-first.json" >/dev/null || die 'first verification did not execute a process'
capture_runtime verify-reuse.json verify --repo "$adopter_root" --workers 1
jq -e '.passed == true and .nodesReused >= 1 and .nodesExecuted == 0 and .processesSpawned == 0' "$output/verify-reuse.json" >/dev/null || die 'second verification did not reuse evidence with zero spawns'
mark_passed reuse-assertion

lifecycle_control_root="$adopter_root"
lifecycle_control_branch="$(git -C "$lifecycle_control_root" branch --show-current)"
lifecycle_branch=release-adopter-lifecycle
lifecycle_worktree="$adopter_root"
git -C "$adopter_root" switch -q -c "$lifecycle_branch"

lifecycle_id=release-adopter-lifecycle
capture_runtime lifecycle-start.json start --repo "$adopter_root" --id "$lifecycle_id" \
  --intent 'Validate the published Runtime against a real adopter change.' \
  --goal 'Demonstrate an auditable Work Item lifecycle using only the pinned public Release binary.' \
  --scope src/lib.rs --out-of-scope target --risk normal --authority authorized \
  --acceptance 'cargo test passes for the adopter change' --required-evidence verification
lifecycle_contract="$adopter_root/.ai/work-items/active/$lifecycle_id.contract.json"
[[ -f "$lifecycle_contract" ]] || die 'lifecycle contract was not created'
lifecycle_base_revision="$(jq -er '.baseRevision | select(type == "string" and test("^[0-9a-f]{40}$"))' "$lifecycle_contract")" || die 'lifecycle Contract base revision is missing or malformed'
lifecycle_head="$(git -C "$adopter_root" rev-parse HEAD)"
lifecycle_pr="acceptance://$tag/$lifecycle_id"
lifecycle_context="$run_root/$lifecycle_id.finalize-context.json"
jq -n \
  --arg branch "$lifecycle_branch" \
  --arg worktree "$adopter_root" \
  --arg provider release-adopter-harness \
  --arg pullRequest "$lifecycle_pr" \
  '{branch:$branch,worktree:$worktree,baseBranch:$branch,baseRemote:"local",provider:$provider,pullRequest:$pullRequest}' \
  > "$lifecycle_context"
cp "$lifecycle_context" "$output/work-items/$lifecycle_id.finalize-context.json"
capture_runtime lifecycle-finalize-plan.json work-item finalize-plan --repo "$adopter_root" --id "$lifecycle_id" --input "$lifecycle_context"
jq -e '.state == "planned" and .workItemId == $id' --arg id "$lifecycle_id" "$output/lifecycle-finalize-plan.json" >/dev/null || die 'lifecycle finalize-plan did not bind the Work Item'
printf '\n// release adopter acceptance mutation\n' >> "$adopter_root/src/lib.rs"
git -C "$adopter_root" add src/lib.rs
git -C "$adopter_root" commit -qm 'make deterministic adopter change'
lifecycle_head="$(git -C "$adopter_root" rev-parse HEAD)"
capture_runtime lifecycle-preflight.json preflight --repo "$adopter_root" --contract "$lifecycle_contract"
capture_runtime lifecycle-checkpoint.json checkpoint --repo "$adopter_root" --id "$lifecycle_id"
capture_runtime lifecycle-verify.json verify --repo "$adopter_root" --work-item "$lifecycle_id" --workers 1
cp "$adopter_root/.ai/evidence/$lifecycle_id.verification.json" "$output/work-items/lifecycle.evidence.json"
jq -e --arg version "$runtime_version" --arg digest "$runtime_digest" '.runtimeVersion == $version and .runtimeDigest == $digest and .passed == true' "$output/work-items/lifecycle.evidence.json" >/dev/null || die 'Work Item verification evidence is not bound to the downloaded Runtime'
mark_passed lifecycle-runtime-identity
capture_runtime lifecycle-finish.json finish --repo "$adopter_root" --id "$lifecycle_id"
capture_runtime lifecycle-archive.json archive --repo "$adopter_root" --id "$lifecycle_id"
archived_lifecycle_contract="$adopter_root/.ai/work-items/archive/$lifecycle_id.contract.json"
archived_lifecycle_contract_digest="sha256:$(sha256_file "$archived_lifecycle_contract")"
git -C "$adopter_root" add .
git -C "$adopter_root" commit -qm 'commit adopter lifecycle archive'
lifecycle_head="$(git -C "$adopter_root" rev-parse HEAD)"
control_clone="$run_root/adopter-control"
git clone -q "$adopter_root" "$control_clone"
git -C "$control_clone" switch -q -c release-adopter-control
remove_exact_tree "$lifecycle_worktree" || die 'lifecycle worktree removal failed after bounded retries'
git -C "$control_clone" worktree prune
git -C "$control_clone" branch -D "$lifecycle_branch" >/dev/null
[[ ! -e "$lifecycle_worktree" && ! -L "$lifecycle_worktree" ]] || die 'lifecycle worktree was not removed before close'
adopter_root="$control_clone"
lifecycle_receipt="$run_root/$lifecycle_id.finalize-receipt.json"
jq -n \
  --arg workItemId "$lifecycle_id" \
  --arg repositoryId "$adopter_repository_id" \
  --arg runtimeVersion "$runtime_version" \
  --arg runtimeDigest "$runtime_digest" \
  --arg provider release-adopter-harness \
  --arg pullRequest "$lifecycle_pr" \
  --arg branch "$lifecycle_branch" \
  --arg worktree "$lifecycle_worktree" \
  --arg headRevision "$lifecycle_head" \
  --arg lifecycleBaseRevision "$lifecycle_base_revision" \
  --arg contractDigest "$archived_lifecycle_contract_digest" \
  --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{
    schemaVersion:1,
    receiptId:("release-adopter-" + $workItemId),
    operationId:("delete-" + $workItemId),
    repositoryId:$repositoryId,
    workItemId:$workItemId,
    runtimeVersion:$runtimeVersion,
    runtimeDigest:$runtimeDigest,
    provider:$provider,
    pullRequest:{number:1,url:$pullRequest,headRevision:$headRevision,baseBranch:$branch,baseRemote:"local",baseRevision:$lifecycleBaseRevision,mergeCommit:$headRevision},
    branch:{name:$branch,remote:"local",headRevision:$headRevision},
    worktree:{worktreeId:$workItemId,path:$worktree,branch:$branch,headRevision:$headRevision},
    before:{pullRequest:"merged",branch:"present",worktree:"clean"},
    after:{pullRequest:"merged",branch:"deleted",worktree:"removed"},
    result:{disposition:"deleted",failureCodes:[],unknownCodes:[]},
    actor:"harness:release-adopter",
    authoritySource:"release-adopter-acceptance",
    reason:"The isolated adopter lifecycle branch and worktree were removed before close.",
    timestamp:$timestamp,
    contractDigest:$contractDigest,
    resourceContext:{branch:$branch,worktree:$worktree,baseBranch:$branch,baseRemote:"local",provider:$provider,pullRequest:$pullRequest}
  }' > "$lifecycle_receipt"
cp "$lifecycle_receipt" "$output/work-items/$lifecycle_id.finalize-receipt.json"
capture_runtime lifecycle-finalize.json work-item finalize --repo "$adopter_root" --id "$lifecycle_id" --input "$lifecycle_receipt"
jq -e '.state == "recorded" and .disposition == "deleted"' "$output/lifecycle-finalize.json" >/dev/null || die 'lifecycle finalize receipt was not recorded as deleted'
capture_runtime lifecycle-finalize-verify.json work-item finalize-verify --repo "$adopter_root" --id "$lifecycle_id"
jq -e '.state == "verified" and .disposition == "deleted"' "$output/lifecycle-finalize-verify.json" >/dev/null || die 'lifecycle finalize verification did not pass'
capture_runtime lifecycle-close.json close --repo "$adopter_root" --id "$lifecycle_id" \
  --human-decision approved \
  --actor human:release-acceptance \
  --authority-source release-adopter-acceptance \
  --reason 'Confirm the published Runtime adopter lifecycle after the pinned Release evidence passed.' \
  --evidence-ref ".ai/evidence/$lifecycle_id.verification.json" \
  --policy-ref "release-adopter-acceptance:$tag" \
  --decided-at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --resume-condition none
validate_close_decision "$lifecycle_id"
for lifecycle_file in \
  "$adopter_root/.ai/work-items/archive/$lifecycle_id.contract.json" \
  "$adopter_root/.ai/work-items/archive/$lifecycle_id.outcome.json" \
  "$adopter_root/.ai/work-items/archive/$lifecycle_id.summary.json"; do
  [[ -f "$lifecycle_file" ]] || die "missing archived lifecycle evidence: $lifecycle_file"
  cp "$lifecycle_file" "$output/work-items/$(basename "$lifecycle_file")"
done
mark_passed lifecycle-assertion

source_after_status="$(git -C "$source_repo" status --porcelain=v1)"
manifest_source_checkout "$source_repo" "$output" "$run_root/source-after.manifest"
manifest_tree "$isolated_home" "$run_root/home-after.manifest"
manifest_tree "$isolated_xdg" "$run_root/xdg-after.manifest"
manifest_tree "$isolated_tmp" "$run_root/tmp-after.manifest"
manifest_tree "$isolated_cargo" "$run_root/cargo-after.manifest"
home_unchanged=true
xdg_unchanged=true
cmp -s "$run_root/home-before.manifest" "$run_root/home-after.manifest" || home_unchanged=false
cmp -s "$run_root/xdg-before.manifest" "$run_root/xdg-after.manifest" || xdg_unchanged=false
[[ "$source_before_status" == "$source_after_status" ]] || die 'acceptance modified the source checkout'
cmp -s "$run_root/source-before.manifest" "$run_root/source-after.manifest" || die 'acceptance modified tracked source or source .ai contents'
validate_manifest_symlink_containment "$isolated_tmp" "$run_root/tmp-before.manifest" || die 'isolated TMPDIR contains an escaping symlink target before Runtime execution'
validate_manifest_symlink_containment "$isolated_tmp" "$run_root/tmp-after.manifest" || die 'isolated TMPDIR contains an escaping symlink target after Runtime execution'
validate_manifest_symlink_containment "$isolated_cargo" "$run_root/cargo-before.manifest" || die 'isolated CARGO_HOME contains an escaping symlink target before Runtime execution'
validate_manifest_symlink_containment "$isolated_cargo" "$run_root/cargo-after.manifest" || die 'isolated CARGO_HOME contains an escaping symlink target after Runtime execution'
if [[ "$source_ai_state" == present ]]; then
  [[ -d "$source_repo/.ai" ]] || die 'acceptance changed source repository .ai state'
else
  [[ ! -e "$source_repo/.ai" ]] || die 'acceptance created .ai in an initially unattached source checkout'
fi
mkdir -p "$output/isolation-manifests"
for name in source-before source-after home-before home-after xdg-before xdg-after tmp-before tmp-after cargo-before cargo-after; do
  cp "$run_root/$name.manifest" "$output/isolation-manifests/$name.manifest"
done
jq -n \
  --arg sourceRepository "$source_repo" \
  --arg sourceAiState "$source_ai_state" \
  --arg sourceBeforeStatus "$source_before_status" \
  --arg sourceAfterStatus "$source_after_status" \
  --arg sourceBeforeDigest "sha256:$(sha256_file "$run_root/source-before.manifest")" \
  --arg sourceAfterDigest "sha256:$(sha256_file "$run_root/source-after.manifest")" \
  --arg adopterRepository "$adopter_root" \
  --argjson homeUnchanged "$home_unchanged" \
  --argjson xdgUnchanged "$xdg_unchanged" \
  --arg homeBeforeDigest "sha256:$(sha256_file "$run_root/home-before.manifest")" \
  --arg homeAfterDigest "sha256:$(sha256_file "$run_root/home-after.manifest")" \
  --arg xdgBeforeDigest "sha256:$(sha256_file "$run_root/xdg-before.manifest")" \
  --arg xdgAfterDigest "sha256:$(sha256_file "$run_root/xdg-after.manifest")" \
  --arg tmpBeforeDigest "sha256:$(sha256_file "$run_root/tmp-before.manifest")" \
  --arg tmpAfterDigest "sha256:$(sha256_file "$run_root/tmp-after.manifest")" \
  --arg cargoBeforeDigest "sha256:$(sha256_file "$run_root/cargo-before.manifest")" \
  --arg cargoAfterDigest "sha256:$(sha256_file "$run_root/cargo-after.manifest")" \
  '{schemaVersion:2,sourceRepository:$sourceRepository,sourceAiState:$sourceAiState,sourceBeforeStatus:$sourceBeforeStatus,sourceAfterStatus:$sourceAfterStatus,sourceManifest:{format:"typed-jsonl",beforeDigest:$sourceBeforeDigest,afterDigest:$sourceAfterDigest},sourceUnchanged:($sourceBeforeStatus == $sourceAfterStatus and $sourceBeforeDigest == $sourceAfterDigest),adopterRepository:$adopterRepository,repositoryIsolation:($sourceRepository != $adopterRepository),roots:{HOME:{classification:"global-config",allowedWrites:false,allowedPrefixes:[],beforeDigest:$homeBeforeDigest,afterDigest:$homeAfterDigest,unchanged:$homeUnchanged},XDG_CONFIG_HOME:{classification:"global-config",allowedWrites:false,allowedPrefixes:[],beforeDigest:$xdgBeforeDigest,afterDigest:$xdgAfterDigest,unchanged:$xdgUnchanged},TMPDIR:{classification:"runtime-temporary",allowedWrites:true,allowedPrefixes:["<TMPDIR>/**"],beforeDigest:$tmpBeforeDigest,afterDigest:$tmpAfterDigest,symlinkTargetsContained:true},CARGO_HOME:{classification:"dependency-cache",allowedWrites:true,allowedPrefixes:["<CARGO_HOME>/**"],beforeDigest:$cargoBeforeDigest,afterDigest:$cargoAfterDigest,symlinkTargetsContained:true}}}' > "$output/isolation.json"
jq -e '.schemaVersion == 2 and .sourceUnchanged and .roots.HOME.unchanged and .roots.XDG_CONFIG_HOME.unchanged and (.roots.HOME.allowedPrefixes | length == 0) and (.roots.XDG_CONFIG_HOME.allowedPrefixes | length == 0) and .roots.TMPDIR.allowedWrites and .roots.TMPDIR.allowedPrefixes == ["<TMPDIR>/**"] and .roots.CARGO_HOME.allowedWrites and .roots.CARGO_HOME.allowedPrefixes == ["<CARGO_HOME>/**"] and .repositoryIsolation' "$output/isolation.json" >/dev/null || die 'isolation proof failed'
mark_passed isolation-assertion

repository_id="$(jq -er '.repositoryId' "$output/agent-doctor.json")"
adopter_head="$(git -C "$adopter_root" rev-parse HEAD)"
initial_head="$(git -C "$adopter_root" rev-list --max-parents=0 HEAD | tail -n 1)"
jq -n \
  --arg repositoryId "$repository_id" \
  --arg sourceRepositoryId "$source_repository_id" \
  --arg adopterPath "$adopter_root" \
  --arg initialHead "$initial_head" \
  --arg head "$adopter_head" \
  --arg runtimeVersion "$runtime_version" \
  --arg runtimeDigest "$runtime_digest" \
  '{schemaVersion:1,repositoryId:$repositoryId,sourceRepositoryId:(if $sourceRepositoryId == "" then null else $sourceRepositoryId end),adopterPath:$adopterPath,initialHead:$initialHead,head:$head,runtimeVersion:$runtimeVersion,runtimeDigest:$runtimeDigest,distinctFromSource:($repositoryId != $sourceRepositoryId)}' > "$output/repository.json"
jq -e '.repositoryId != null and .distinctFromSource == true' "$output/repository.json" >/dev/null || die 'adopter repository identity is not distinct from source'
mark_passed repository-identity

if [[ "$acceptance_scope" == candidate ]]; then
  record_phase_success candidate_acceptance "$acceptance_evidence"
else
  record_phase_success public_acceptance "$acceptance_evidence"
fi
close_ready=true
overall_state=passed
failure_reason=''
