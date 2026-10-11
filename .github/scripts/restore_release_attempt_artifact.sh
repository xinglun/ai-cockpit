#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'staged release attempt recovery failed: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat >&2 <<'USAGE'
Usage:
  restore_release_attempt_artifact.sh seal --stage STAGE --output DIRECTORY
  restore_release_attempt_artifact.sh restore --stage STAGE --output DIRECTORY
USAGE
  exit 2
}

sha256() { sha256sum -- "$1" | awk '{print $1}'; }

artifact_prefix() {
  case "$1" in
    staged_adopter_acceptance) printf '%s' staged-adopter-acceptance ;;
    staged_adopter_upgrade_acceptance) printf '%s' staged-adopter-upgrade ;;
    *) fail "unsupported staged acceptance job: $1" ;;
  esac
}

require_identity_environment() {
  [[ "${GITHUB_REPOSITORY:-}" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]] || fail 'GITHUB_REPOSITORY is missing or malformed'
  [[ "${GITHUB_RUN_ID:-}" =~ ^[1-9][0-9]*$ ]] || fail 'GITHUB_RUN_ID is missing or malformed'
  [[ "${GITHUB_RUN_ATTEMPT:-}" =~ ^[1-9][0-9]*$ ]] || fail 'GITHUB_RUN_ATTEMPT is missing or malformed'
  [[ "${GITHUB_SHA:-}" =~ ^[0-9a-f]{40}$ ]] || fail 'GITHUB_SHA is missing or malformed'
  [[ "${RELEASE_PLAN_DIGEST:-}" =~ ^sha256:[0-9a-f]{64}$ ]] || fail 'typed ReleasePlan digest is missing or malformed'
  [[ "${RELEASE_SOURCE_REVISION:-}" =~ ^[0-9a-f]{40}$ ]] || fail 'typed release source revision is missing or malformed'
  [[ -n "${GITHUB_OUTPUT:-}" ]] || fail 'GITHUB_OUTPUT is required'
}

load_candidate_identity() {
  local candidate_dir=$1
  [[ -d "$candidate_dir" && ! -L "$candidate_dir" ]] || fail 'candidate artifact directory is missing, symlinked, or not a directory'
  candidate_dir="$(cd "$candidate_dir" && pwd -P)"
  CANDIDATE_DIR_REAL=$candidate_dir
  CANDIDATE_MANIFEST="$candidate_dir/release-manifest.json"
  CANDIDATE_SUMS="$candidate_dir/SHA256SUMS"
  [[ -f "$CANDIDATE_MANIFEST" && ! -L "$CANDIDATE_MANIFEST" ]] || fail 'candidate release manifest is missing or symlinked'
  [[ -f "$CANDIDATE_SUMS" && ! -L "$CANDIDATE_SUMS" ]] || fail 'candidate checksum file is missing or symlinked'
  CANDIDATE_TAG="$(jq -er '.tag | select(type == "string" and test("^v[0-9]+[.][0-9]+[.][0-9]+$"))' "$CANDIDATE_MANIFEST")" || fail 'candidate tag is missing or malformed'
  CANDIDATE_VERSION="$(jq -er '.version | select(type == "string" and test("^[0-9]+[.][0-9]+[.][0-9]+$"))' "$CANDIDATE_MANIFEST")" || fail 'candidate version is missing or malformed'
  [[ "v$CANDIDATE_VERSION" == "$CANDIDATE_TAG" ]] || fail 'candidate tag and version disagree'
  CANDIDATE_SOURCE="$(jq -er '.commit | select(type == "string" and test("^[0-9a-f]{40}$"))' "$CANDIDATE_MANIFEST")" || fail 'candidate source commit is missing or malformed'
  [[ "$CANDIDATE_SOURCE" == "$RELEASE_SOURCE_REVISION" ]] || fail 'candidate source commit does not match the typed ReleasePlan source revision'
  if [[ -n "${TO_TAG:-}" && "$TO_TAG" != "$CANDIDATE_TAG" ]]; then
    fail 'candidate tag does not match the resolved release pair'
  fi
  CANDIDATE_ARCHIVE="$(jq -er --arg target x86_64-unknown-linux-gnu '.artifacts[] | select(.target == $target) | .archive.filename | select(type == "string" and length > 0)' "$CANDIDATE_MANIFEST")" || fail 'candidate target archive is missing from its manifest'
  [[ "${CANDIDATE_ARCHIVE##*/}" == "$CANDIDATE_ARCHIVE" && "$CANDIDATE_ARCHIVE" == "ai-cockpit-$CANDIDATE_TAG-x86_64-unknown-linux-gnu.tar.gz" ]] || fail 'candidate archive filename does not match its target and tag'
  CANDIDATE_ARCHIVE_SHA="$(jq -er --arg target x86_64-unknown-linux-gnu '.artifacts[] | select(.target == $target) | .archive.sha256 | select(type == "string" and test("^[0-9a-f]{64}$"))' "$CANDIDATE_MANIFEST")" || fail 'candidate target archive digest is missing or malformed'
  CANDIDATE_MANIFEST_SHA="$(sha256 "$CANDIDATE_MANIFEST")"
  CANDIDATE_ARCHIVE_PATH="$candidate_dir/$CANDIDATE_ARCHIVE"
  [[ -f "$CANDIDATE_ARCHIVE_PATH" && ! -L "$CANDIDATE_ARCHIVE_PATH" ]] || fail 'candidate target archive is missing or symlinked'
  [[ "$(sha256 "$CANDIDATE_ARCHIVE_PATH")" == "$CANDIDATE_ARCHIVE_SHA" ]] || fail 'candidate archive does not match the current manifest digest'
  [[ "$(awk -v name="$CANDIDATE_ARCHIVE" '$2 == name {print $1}' "$CANDIDATE_SUMS")" == "$CANDIDATE_ARCHIVE_SHA" ]] || fail 'candidate archive does not match the current SHA256SUMS entry'
}

validate_sums() {
  local root=$1 sums="$1/SHA256SUMS" line digest rel resolved
  [[ -f "$sums" && ! -L "$sums" ]] || return 1
  [[ -z "$(find "$root" -type l -print -quit)" ]] || return 1
  local -A seen=()
  local listed=0 actual=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ "$line" =~ ^([0-9a-f]{64})\ \ (.+)$ ]] || return 1
    digest=${BASH_REMATCH[1]}
    rel=${BASH_REMATCH[2]}
    [[ "$rel" != /* && "$rel" != . && "$rel" != .. && "$rel" != ../* && "$rel" != */../* && "$rel" != */.. ]] || return 1
    [[ "$rel" != *$'\n'* && "$rel" != *$'\r'* ]] || return 1
    [[ -z "${seen[$rel]+present}" ]] || return 1
    seen[$rel]=1
    [[ -f "$root/$rel" && ! -L "$root/$rel" ]] || return 1
    resolved="$(realpath -e "$root/$rel" 2>/dev/null)" || return 1
    case "$resolved" in "$root"/*) ;; *) return 1 ;; esac
    ((listed += 1))
  done < "$sums"
  while IFS= read -r -d '' line; do
    [[ "$line" == "$sums" ]] && continue
    ((actual += 1))
    [[ -n "${seen[${line#"$root"/}]+present}" ]] || return 1
  done < <(find "$root" -type f -print0)
  (( listed > 0 && actual == listed )) || return 1
  (cd "$root" && sha256sum --check --strict SHA256SUMS >/dev/null 2>&1) || return 1
}

write_sums() {
  local root=$1 file rel
  [[ -z "$(find "$root" -type l -print -quit)" ]] || fail 'acceptance output contains a symlink'
  : > "$root/SHA256SUMS"
  while IFS= read -r -d '' file; do
    [[ "$file" == "$root/SHA256SUMS" ]] && continue
    rel=${file#"$root"/}
    [[ "$rel" != *$'\n'* && "$rel" != *$'\r'* ]] || fail 'acceptance output contains a filename that cannot be checksummed safely'
    (cd "$root" && sha256sum -- "$rel") >> "$root/SHA256SUMS"
  done < <(find "$root" -type f -print0 | LC_ALL=C sort -z)
}

cache_paths() {
  local stage=$1 root=$2
  if [[ "$stage" == staged_adopter_acceptance ]]; then
    CACHE_ARCHIVE="$root/.resume-cache/$CANDIDATE_ARCHIVE"
    CACHE_MANIFEST="$root/.resume-cache/release-manifest.json"
    CACHE_SUMS="$root/.resume-cache/SHA256SUMS"
  else
    CACHE_ARCHIVE="$root/.resume-cache/to-$CANDIDATE_ARCHIVE"
    CACHE_MANIFEST="$root/.resume-cache/to-release-manifest.json"
    CACHE_SUMS="$root/.resume-cache/to-SHA256SUMS"
  fi
}

validate_candidate_cache() {
  local stage=$1 root=$2
  cache_paths "$stage" "$root"
  [[ -f "$CACHE_ARCHIVE" && ! -L "$CACHE_ARCHIVE" ]] || return 2
  [[ -f "$CACHE_MANIFEST" && ! -L "$CACHE_MANIFEST" ]] || return 2
  [[ -f "$CACHE_SUMS" && ! -L "$CACHE_SUMS" ]] || return 2
  [[ "$(sha256 "$CACHE_ARCHIVE")" == "$CANDIDATE_ARCHIVE_SHA" ]] || return 1
  [[ "$(sha256 "$CACHE_MANIFEST")" == "$CANDIDATE_MANIFEST_SHA" ]] || return 1
  cmp -s "$CACHE_MANIFEST" "$CANDIDATE_MANIFEST" || return 1
  cmp -s "$CACHE_ARCHIVE" "$CANDIDATE_ARCHIVE_PATH" || return 1
  cmp -s "$CACHE_SUMS" "$CANDIDATE_SUMS" || return 1
  return 0
}

validate_previous_cache() {
  local root=$1 from_tag=${FROM_TAG:-} from_version from_archive from_digest archive_path manifest_path sums_path
  [[ "$from_tag" =~ ^v[0-9]+[.][0-9]+[.][0-9]+$ && "$from_tag" != "$CANDIDATE_TAG" ]] || return 1
  from_version=${from_tag#v}
  from_archive="ai-cockpit-$from_tag-x86_64-unknown-linux-gnu.tar.gz"
  archive_path="$root/.resume-cache/from-$from_archive"
  manifest_path="$root/.resume-cache/from-release-manifest.json"
  sums_path="$root/.resume-cache/from-SHA256SUMS"
  [[ -f "$archive_path" && ! -L "$archive_path" ]] || return 2
  [[ -f "$manifest_path" && ! -L "$manifest_path" ]] || return 2
  [[ -f "$sums_path" && ! -L "$sums_path" ]] || return 2
  from_digest="$(jq -er --arg tag "$from_tag" --arg version "$from_version" --arg target x86_64-unknown-linux-gnu \
    'select(.tag == $tag and .version == $version) | .artifacts[] | select(.target == $target) | .archive.sha256 | select(type == "string" and test("^[0-9a-f]{64}$"))' "$manifest_path")" || return 1
  [[ "$(sha256 "$archive_path")" == "$from_digest" ]] || return 1
  [[ "$(awk -v name="$from_archive" '$2 == name {print $1}' "$sums_path")" == "$from_digest" ]] || return 1
  [[ "$(sha256 "$manifest_path")" == "$(jq -er '.previous.manifestDigest | select(type == "string") | sub("^sha256:"; "")' "$root/phase-identity.json")" ]] || return 1
  jq -e --arg tag "$from_tag" --arg archive "$from_archive" --arg digest "sha256:$from_digest" \
    '.previous.tag == $tag and .previous.assets[$archive] == $digest' "$root/phase-identity.json" >/dev/null || return 1
  return 0
}

validate_phase_identity() {
  local stage=$1 root=$2 identity="$2/phase-identity.json" receipts="$2/phase-receipts.json" plan_dir plan_output
  [[ -f "$identity" && ! -L "$identity" ]] || return 2
  [[ -f "$receipts" && ! -L "$receipts" ]] || return 2
  jq -e \
    --arg source "$RELEASE_SOURCE_REVISION" \
    --arg tag "$CANDIDATE_TAG" \
    --arg version "$CANDIDATE_VERSION" \
    --arg manifest "sha256:$CANDIDATE_MANIFEST_SHA" \
    --arg archive "$CANDIDATE_ARCHIVE" \
    --arg archiveDigest "sha256:$CANDIDATE_ARCHIVE_SHA" \
    '.source.commit == $source
     and .candidate.tag == $tag
     and .candidate.version == $version
     and .candidate.manifestDigest == $manifest
     and .candidate.assets[$archive] == $archiveDigest
     and .target == "x86_64-unknown-linux-gnu"' "$identity" >/dev/null || return 1
  if [[ "$stage" == staged_adopter_acceptance ]]; then
    jq -e '.previous == null' "$identity" >/dev/null || return 1
  else
    jq -e --arg tag "${FROM_TAG:-}" '.previous.tag == $tag' "$identity" >/dev/null || return 1
  fi
  jq -e --slurpfile expected "$identity" \
    'type == "object"
     and .schemaVersion == 1
     and (.identityDigest | type == "string" and test("^sha256:[0-9a-f]{64}$"))
     and .identity == $expected[0]
     and (.results | type == "array" and length > 0)' "$receipts" >/dev/null || return 1
  [[ -n "${COCKPIT_RELEASE_BIN:-}" && -x "$COCKPIT_RELEASE_BIN" ]] || return 1
  plan_dir="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/staged-acceptance-plan.XXXXXX")" || return 1
  plan_output="$plan_dir/plan.json"
  if ! "$COCKPIT_RELEASE_BIN" acceptance-plan \
    --scope candidate \
    --identity "$identity" \
    --receipts "$receipts" \
    --output "$plan_output" >/dev/null 2>&1; then
    rm -rf "$plan_dir"
    return 1
  fi
  rm -rf "$plan_dir"
  local acceptance="$root/acceptance.json"
  [[ -f "$acceptance" && ! -L "$acceptance" ]] || return 2
  jq -e '(.adopterAcceptance == "passed" or .adopterAcceptance == "failed")' "$acceptance" >/dev/null || return 1
  return 0
}

write_binding() {
  local stage=$1 root=$2 state=$3 reason=$4 phase_identity_digest='' manifest_binding='' archive_binding=''
  if [[ -f "$root/phase-identity.json" && ! -L "$root/phase-identity.json" ]]; then
    phase_identity_digest="sha256:$(sha256 "$root/phase-identity.json")"
  fi
  [[ -z "${CANDIDATE_MANIFEST_SHA:-}" ]] || manifest_binding="sha256:$CANDIDATE_MANIFEST_SHA"
  [[ -z "${CANDIDATE_ARCHIVE_SHA:-}" ]] || archive_binding="sha256:$CANDIDATE_ARCHIVE_SHA"
  jq -n \
    --arg stage "$stage" \
    --arg state "$state" \
    --arg reason "$reason" \
    --arg runId "$GITHUB_RUN_ID" \
    --arg attempt "$GITHUB_RUN_ATTEMPT" \
    --arg workflowHead "$GITHUB_SHA" \
    --arg planDigest "$RELEASE_PLAN_DIGEST" \
    --arg sourceRevision "$RELEASE_SOURCE_REVISION" \
    --arg tag "${CANDIDATE_TAG:-}" \
    --arg version "${CANDIDATE_VERSION:-}" \
    --arg manifestSha "$manifest_binding" \
    --arg archive "${CANDIDATE_ARCHIVE:-}" \
    --arg archiveSha "$archive_binding" \
    --arg fromTag "${FROM_TAG:-}" \
    --arg phaseIdentityDigest "$phase_identity_digest" \
    '{schemaVersion:1,kind:"staged_release_attempt_artifact",state:$state,reason:(if $reason == "" then null else $reason end),run:{id:($runId|tonumber),attempt:($attempt|tonumber)},stage:$stage,workflowHead:$workflowHead,planDigest:$planDigest,releaseSourceRevision:$sourceRevision,candidate:{tag:(if $tag == "" then null else $tag end),version:(if $version == "" then null else $version end),manifestSha256:(if $manifestSha == "" then null else $manifestSha end),archive:(if $archive == "" then null else $archive end),archiveSha256:(if $archiveSha == "" then null else $archiveSha end)},previousTag:(if $fromTag == "" then null else $fromTag end),phaseIdentitySha256:(if $phaseIdentityDigest == "" then null else $phaseIdentityDigest end)}' \
    > "$root/phase-binding.json"
}

seal() {
  local stage=$1 output=$2 state=ready reason='' receipt_result cache_result
  [[ -d "$output" && ! -L "$output" ]] || fail 'acceptance output directory is missing or symlinked'
  output="$(cd "$output" && pwd -P)"
  local candidate_dir="${CANDIDATE_DIR:-$GITHUB_WORKSPACE/dist}"
  load_candidate_identity "$candidate_dir"
  if [[ "$stage" == staged_adopter_upgrade_acceptance ]]; then
    if [[ -z "${FROM_TAG:-}" ]]; then
      if jq -e --arg tag "$CANDIDATE_TAG" \
        '.schemaVersion == 1 and .releasePublished == false and .stagedCandidate == true
         and .adopterAcceptance == "not_applicable" and .fromTag == null
         and .toTag == $tag and .target == "x86_64-unknown-linux-gnu"' \
        "$output/acceptance.json" >/dev/null 2>&1; then
        state=not_applicable
        write_binding "$stage" "$output" "$state" 'no earlier public Release exists for N-1 acceptance'
        write_sums "$output"
        echo "sealed not-applicable staged acceptance evidence for $stage"
        return 0
      fi
      fail 'previous Release tag is empty but the N-1 not-applicable receipt is missing or mismatched'
    fi
    [[ "$FROM_TAG" =~ ^v[0-9]+[.][0-9]+[.][0-9]+$ && "$FROM_TAG" != "$CANDIDATE_TAG" ]] || fail 'resolved previous Release tag is malformed'
  fi
  if validate_phase_identity "$stage" "$output"; then receipt_result=0; else receipt_result=$?; fi
  if (( receipt_result == 2 )); then
    state=incomplete
    reason='acceptance identity, receipt, or final result was not persisted'
  elif (( receipt_result != 0 )); then
    state=invalid
    reason='acceptance phase identity or receipt does not match the current candidate'
  else
    if validate_candidate_cache "$stage" "$output"; then cache_result=0; else cache_result=$?; fi
    if (( cache_result == 2 )); then
      state=incomplete
      reason='candidate resume cache was not persisted'
    elif (( cache_result != 0 )); then
      state=invalid
      reason='candidate resume cache differs from the current candidate artifact'
    fi
    if [[ "$state" == ready && "$stage" == staged_adopter_upgrade_acceptance ]]; then
      if validate_previous_cache "$output"; then cache_result=0; else cache_result=$?; fi
      if (( cache_result == 2 )); then
        state=incomplete
        reason='previous Release resume cache was not persisted'
      elif (( cache_result != 0 )); then
        state=invalid
        reason='previous Release resume cache does not match the recorded identity'
      fi
    fi
  fi
  write_binding "$stage" "$output" "$state" "$reason"
  write_sums "$output"
  if [[ "$state" == invalid ]]; then fail "$reason"; fi
  if [[ "$state" == ready ]]; then
    echo "sealed resumable staged acceptance evidence for $stage"
  else
    echo "sealed non-resumable staged acceptance evidence for $stage: $reason"
  fi
}

fetch_pages() {
  local kind=$1 endpoint=$2 destination=$3 jq_filter
  case "$kind" in jobs) jq_filter='{total_count,items:.jobs}' ;; artifacts) jq_filter='{total_count,items:.artifacts}' ;; *) return 1 ;; esac
  gh api --paginate "$endpoint" --jq "$jq_filter" > "$destination" || return 1
  jq -s -e '
    . as $pages
    | ($pages | length > 0)
      and all($pages[]; type == "object" and (.total_count | type == "number" and . >= 0) and (.items | type == "array"))
      and ([$pages[].total_count] | unique | length == 1)
      and ([$pages[].items[]] | length == $pages[0].total_count)
  ' "$destination" >/dev/null 2>&1 || return 1
  jq -s '[.[].items[]]' "$destination" > "${destination}.items.json"
}

extract_artifact_zip() {
  local archive=$1 destination=$2
  python3 - "$archive" "$destination" <<'PY'
import os
import shutil
import stat
import sys
import zipfile
from pathlib import PurePosixPath

archive_path, destination = sys.argv[1:]
root = os.path.realpath(destination)
os.makedirs(root, mode=0o700, exist_ok=True)

with zipfile.ZipFile(archive_path) as source:
    members = source.infolist()
    names = set()
    for member in members:
        name = member.filename
        path = PurePosixPath(name.rstrip("/"))
        if (
            not name
            or "\x00" in name
            or "\\" in name
            or path.is_absolute()
            or not path.parts
            or any(part in {"", ".", ".."} for part in path.parts)
            or (len(path.parts[0]) >= 2 and path.parts[0][1] == ":")
        ):
            raise SystemExit("unsafe artifact archive path")
        normalized = path.as_posix()
        if normalized in names:
            raise SystemExit("duplicate artifact archive path")
        names.add(normalized)
        mode = (member.external_attr >> 16) & 0o170000
        if mode == stat.S_IFLNK or mode not in {0, stat.S_IFREG, stat.S_IFDIR}:
            raise SystemExit("unsupported artifact archive file type")
        resolved = os.path.realpath(os.path.join(root, *path.parts))
        if os.path.commonpath((root, resolved)) != root:
            raise SystemExit("artifact archive path escapes its destination")

    for member in members:
        path = PurePosixPath(member.filename.rstrip("/"))
        target = os.path.join(root, *path.parts)
        if member.is_dir():
            os.makedirs(target, mode=0o700, exist_ok=True)
            continue
        os.makedirs(os.path.dirname(target), mode=0o700, exist_ok=True)
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
        if hasattr(os, "O_NOFOLLOW"):
            flags |= os.O_NOFOLLOW
        descriptor = os.open(target, flags, 0o600)
        with os.fdopen(descriptor, "wb") as output, source.open(member) as input_file:
            shutil.copyfileobj(input_file, output)
PY
}

validate_downloaded_artifact() {
  local stage=$1 root=$2 previous_attempt=$3 binding="$2/phase-binding.json"
  validate_sums "$root" || fail 'artifact checksum set is missing, unsafe, incomplete, or invalid'
  [[ -f "$binding" && ! -L "$binding" ]] || fail 'artifact phase binding is missing or symlinked'
  if ! jq -e \
    --arg stage "$stage" \
    --arg runId "$GITHUB_RUN_ID" \
    --arg attempt "$previous_attempt" \
    --arg workflowHead "$GITHUB_SHA" \
    --arg planDigest "$RELEASE_PLAN_DIGEST" \
    --arg sourceRevision "$RELEASE_SOURCE_REVISION" \
    '.schemaVersion == 1
     and .kind == "staged_release_attempt_artifact"
     and (.state == "ready" or .state == "not_applicable")
     and .run.id == ($runId|tonumber)
     and .run.attempt == ($attempt|tonumber)
     and .stage == $stage
     and .workflowHead == $workflowHead
     and .planDigest == $planDigest
     and .releaseSourceRevision == $sourceRevision' "$binding" >/dev/null; then
    jq -c '{state,run,stage,workflowHead,planDigest,releaseSourceRevision}' "$binding" >&2 || true
    fail 'artifact phase binding does not match this run, stage, workflow head, plan, source, or prior attempt'
  fi
  local output="$root" candidate_dir="${CANDIDATE_DIR:-$GITHUB_WORKSPACE/dist}"
  load_candidate_identity "$candidate_dir"
  if ! jq -e \
    --arg tag "$CANDIDATE_TAG" \
    --arg version "$CANDIDATE_VERSION" \
    --arg manifest "sha256:$CANDIDATE_MANIFEST_SHA" \
    --arg archive "$CANDIDATE_ARCHIVE" \
    --arg archiveSha "sha256:$CANDIDATE_ARCHIVE_SHA" \
    --arg fromTag "${FROM_TAG:-}" \
    '.candidate.tag == $tag
     and .candidate.version == $version
     and .candidate.manifestSha256 == $manifest
     and .candidate.archive == $archive
     and .candidate.archiveSha256 == $archiveSha
     and .previousTag == (if $fromTag == "" then null else $fromTag end)' "$binding" >/dev/null; then
    jq -c '{candidate,previousTag}' "$binding" >&2 || true
    fail 'artifact phase binding candidate identity does not match the current release inputs'
  fi
  if [[ "$(jq -r '.state' "$binding")" == not_applicable ]]; then
    [[ "$stage" == staged_adopter_upgrade_acceptance && -z "${FROM_TAG:-}" ]] || fail 'not-applicable artifact is incompatible with the resolved N-1 input'
    jq -e --arg tag "$CANDIDATE_TAG" \
      '.schemaVersion == 1 and .releasePublished == false and .stagedCandidate == true
       and .adopterAcceptance == "not_applicable" and .fromTag == null
       and .toTag == $tag and .target == "x86_64-unknown-linux-gnu"' \
      "$root/acceptance.json" >/dev/null || fail 'not-applicable artifact receipt does not match the current candidate'
    return 0
  fi
  if [[ "$stage" == staged_adopter_upgrade_acceptance ]]; then
    [[ "${FROM_TAG:-}" =~ ^v[0-9]+[.][0-9]+[.][0-9]+$ && "$FROM_TAG" != "$CANDIDATE_TAG" ]] || fail 'resolved previous Release tag is missing or malformed'
  fi
  local result
  if validate_phase_identity "$stage" "$output"; then result=0; else result=$?; fi
  (( result == 0 )) || fail 'artifact phase receipts or acceptance result are incomplete or mismatched'
  jq -e --arg digest "sha256:$(sha256 "$output/phase-identity.json")" \
    '.phaseIdentitySha256 == $digest' "$binding" >/dev/null || fail 'artifact binding does not match the stored phase identity bytes'
  if validate_candidate_cache "$stage" "$output"; then result=0; else result=$?; fi
  (( result == 0 )) || fail 'artifact cached candidate manifest or archive differs from the current candidate'
  if [[ "$stage" == staged_adopter_upgrade_acceptance ]]; then
    if validate_previous_cache "$output"; then result=0; else result=$?; fi
    (( result == 0 )) || fail 'artifact cached previous Release does not match its acceptance identity'
  fi
}

restore() {
  local stage=$1 output=$2 attempt=$GITHUB_RUN_ATTEMPT previous_attempt=$((GITHUB_RUN_ATTEMPT - 1))
  [[ -d "$output" || ! -e "$output" ]] || fail 'restore destination is not a directory'
  mkdir -p "$output"
  output="$(cd "$output" && pwd -P)"
  [[ -z "$(find "$output" -mindepth 1 -print -quit)" ]] || fail 'restore destination must be empty'
  load_candidate_identity "${CANDIDATE_DIR:-$GITHUB_WORKSPACE/dist}"
  if (( attempt == 1 )); then
    printf 'resume=false\n' >> "$GITHUB_OUTPUT"
    return 0
  fi
  local name="$(artifact_prefix "$stage")-$GITHUB_RUN_ID-$previous_attempt"
  local scratch="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/staged-release-attempt.XXXXXX")"
  trap '[[ -z "${scratch:-}" ]] || rm -rf "$scratch"' EXIT
  local jobs_file="$scratch/jobs.pages.jsonl" artifacts_file="$scratch/artifacts.pages.jsonl"
  local jobs_items="$scratch/jobs.json" artifacts_items="$scratch/artifacts.json" matched count
  local jobs_endpoint="repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/attempts/$previous_attempt/jobs?per_page=100"
  local artifacts_endpoint="repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/artifacts?per_page=100&name=$name"
  if ! fetch_pages jobs "$jobs_endpoint" "$jobs_file"; then fail 'exact prior-attempt jobs API query failed or returned incomplete pagination' ; fi
  if ! fetch_pages artifacts "$artifacts_endpoint" "$artifacts_file"; then fail 'prior-run artifact API query failed or returned incomplete pagination' ; fi
  jobs_items="$jobs_file.items.json"
  artifacts_items="$artifacts_file.items.json"
  matched="$(jq -c --arg name "$name" '[.[] | select(.name == $name)]' "$artifacts_items")" || fail 'artifact list could not be parsed'
  count="$(jq -er 'length' <<<"$matched")" || fail 'artifact match count could not be read'
  if (( count == 0 )); then
    local jobs="$(jq -c --arg stage "$stage" '[.[] | select(.name == $stage)]' "$jobs_items")" || fail 'exact attempt job list could not be parsed'
    count="$(jq -er 'length' <<<"$jobs")" || fail 'exact attempt job count could not be read'
    if (( count == 0 )); then
      fail 'exact prior attempt has no stage job record; cannot establish that the stage was skipped'
    elif (( count == 1 )); then
      # The attempt number is bound by jobs_endpoint. GitHub's job response
      # schema exposes run_id/head_sha but not run_attempt; validate the exact
      # fields the endpoint returns and check run_attempt if it is ever present.
      jq -e \
        --arg runId "$GITHUB_RUN_ID" \
        --arg attempt "$previous_attempt" \
        --arg head "$GITHUB_SHA" \
        'length == 1
         and (.[0].id | type == "number" and . > 0)
         and (.[0].run_id | type == "number" and . == ($runId|tonumber))
         and .[0].head_sha == $head
         and ((.[0] | has("run_attempt") | not) or .[0].run_attempt == ($attempt|tonumber))' \
        <<<"$jobs" >/dev/null || fail 'exact prior-attempt stage job identity is missing or mismatched'
      jq -e '.[0].status == "completed" and .[0].conclusion == "skipped" and .[0].steps == []' <<<"$jobs" >/dev/null || \
        fail 'exact prior-attempt stage ran, was not conclusively skipped, or has step evidence; artifact is missing'
    else
      fail 'exact prior attempt has duplicate stage jobs; refusing to choose one'
    fi
    (( previous_attempt == 1 )) || fail 'missing exact prior-attempt artifact after attempt 1; earlier attempts are not searched'
    printf 'resume=false\n' >> "$GITHUB_OUTPUT"
    echo "prior attempt $previous_attempt did not run $stage and has no exact artifact; starting this stage fresh"
    rm -rf "$scratch"
    return 0
  fi
  (( count == 1 )) || fail 'more than one artifact matches the exact prior-attempt name'
  local artifact_id
  artifact_id="$(jq -er '.[0].id | select(type == "number" and . > 0 and floor == .) | tostring | select(test("^[1-9][0-9]*$"))' <<<"$matched")" || \
    fail 'exact prior-attempt artifact has no valid numeric ID'
  jq -e \
    --arg runId "$GITHUB_RUN_ID" \
    --arg head "$GITHUB_SHA" \
    '.[0].expired == false
     and (.[0].id | type == "number" and . > 0)
     and (.[0].size_in_bytes | type == "number" and . > 0)
     and .[0].workflow_run.id == ($runId|tonumber)
     and .[0].workflow_run.head_sha == $head' <<<"$matched" >/dev/null || fail 'exact artifact is expired or belongs to a different workflow run/head'
  if ! gh api "repos/$GITHUB_REPOSITORY/actions/artifacts/$artifact_id/zip" > "$scratch/artifact.zip"; then
    fail 'exact prior-attempt artifact download by ID failed'
  fi
  [[ -f "$scratch/artifact.zip" && ! -L "$scratch/artifact.zip" && -s "$scratch/artifact.zip" ]] || fail 'artifact download by ID did not produce a regular archive'
  mkdir -m 700 "$scratch/download"
  if ! extract_artifact_zip "$scratch/artifact.zip" "$scratch/download"; then
    fail 'exact prior-attempt artifact archive is invalid or unsafe to extract'
  fi
  [[ -d "$scratch/download" && ! -L "$scratch/download" ]] || fail 'artifact archive extraction did not produce a regular directory'
  validate_downloaded_artifact "$stage" "$scratch/download" "$previous_attempt"
  cp -a "$scratch/download/." "$output/" || fail 'verified artifact could not be restored'
  printf 'resume=true\n' >> "$GITHUB_OUTPUT"
  echo "restored the verified $stage artifact from attempt $previous_attempt"
  rm -rf "$scratch"
}

command=${1:-}
[[ -n "$command" ]] || usage
shift
stage=''
output=''
while (($#)); do
  case "$1" in
    --stage) (($# >= 2)) || usage; stage=$2; shift 2 ;;
    --output) (($# >= 2)) || usage; output=$2; shift 2 ;;
    *) usage ;;
  esac
done
[[ -n "$stage" && -n "$output" ]] || usage
require_identity_environment
artifact_prefix "$stage" >/dev/null

case "$command" in
  seal) seal "$stage" "$output" ;;
  restore) restore "$stage" "$output" ;;
  *) usage ;;
esac
