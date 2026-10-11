#!/usr/bin/env bash
set -euo pipefail

repo="$(git rev-parse --show-toplevel)"
helper="$repo/.github/scripts/restore_release_attempt_artifact.sh"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-release-attempt-restore.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT
export RUNNER_TEMP="$tmp/runner-temp"
mkdir -p "$RUNNER_TEMP"

fail() { printf 'restore regression failure: %s\n' "$*" >&2; exit 1; }
expect_restore_failure() {
  local label=$1
  if restore >/dev/null 2>&1; then fail "$label unexpectedly passed"; fi
}
expect_restore_failure_reason() {
  local label=$1 reason=$2 log="$tmp/restore-failure.log"
  if restore >"$log" 2>&1; then fail "$label unexpectedly passed"; fi
  grep -Fq "$reason" "$log" || {
    cat "$log" >&2
    fail "$label failed for a different reason than: $reason"
  }
}
sha() { sha256sum "$1" | awk '{print $1}'; }

export GITHUB_REPOSITORY=xinglun/ai-cockpit
export GITHUB_WORKSPACE="$tmp"
export GITHUB_OUTPUT="$tmp/default-output"
export GITHUB_RUN_ID=173
export GITHUB_RUN_ATTEMPT=2
export GITHUB_SHA=0123456789abcdef0123456789abcdef01234567
export RELEASE_PLAN_DIGEST=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
export RELEASE_SOURCE_REVISION=89abcdef0123456789abcdef0123456789abcdef
export TO_TAG=v1.0.1
export FROM_TAG=v1.0.0
export CANDIDATE_DIR="$tmp/candidate"
export GH_TOKEN=test-token
export PATH="$tmp/bin:$PATH"
mkdir -p "$tmp/bin" "$tmp/candidate" "$tmp/artifacts"

archive=ai-cockpit-v1.0.1-x86_64-unknown-linux-gnu.tar.gz
printf 'current candidate bytes\n' > "$tmp/candidate/$archive"
candidate_archive_sha="$(sha "$tmp/candidate/$archive")"
jq -n --arg tag "$TO_TAG" --arg commit "$RELEASE_SOURCE_REVISION" --arg archive "$archive" --arg digest "$candidate_archive_sha" \
  '{version:"1.0.1",tag:$tag,commit:$commit,artifacts:[{target:"x86_64-unknown-linux-gnu",archive:{filename:$archive,sha256:$digest}}]}' \
  > "$tmp/candidate/release-manifest.json"
printf '%s  %s\n' "$candidate_archive_sha" "$archive" > "$tmp/candidate/SHA256SUMS"

old_archive=ai-cockpit-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
old_archive_sha="$(printf 'previous release bytes\n' | sha256sum | awk '{print $1}')"

export GH_FIXTURE="$tmp/gh"
mkdir -p "$GH_FIXTURE"
cat > "$tmp/bin/gh" <<'GH'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  api)
    endpoint=''
    while (($#)); do
      case "$1" in
        repos/*) endpoint=$1; shift ;;
        *) shift ;;
      esac
    done
    if [[ "$endpoint" == repos/$GITHUB_REPOSITORY/actions/artifacts/*/zip ]]; then
      [[ "$endpoint" == "repos/$GITHUB_REPOSITORY/actions/artifacts/$GH_EXPECTED_ARTIFACT_ID/zip" ]] || exit 76
      [[ "${GH_FAIL:-}" != download ]] || exit 75
      cat "$GH_FIXTURE/artifact.zip"
      exit 0
    fi
    [[ -n "$endpoint" ]] || exit 70
    if [[ "${GH_FAIL:-}" == jobs && "$endpoint" == */jobs\?* ]]; then exit 71; fi
    if [[ "${GH_FAIL:-}" == artifacts && "$endpoint" == */artifacts\?* ]]; then exit 72; fi
    if [[ "$endpoint" == */jobs\?* ]]; then
      expected="repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/attempts/$((GITHUB_RUN_ATTEMPT - 1))/jobs?per_page=100"
      [[ "$endpoint" == "$expected" ]] || exit 78
      if [[ "${GH_MULTI_PAGE:-}" == 1 ]]; then
        jq -c '{total_count,items:(.jobs[0:1])}' "$GH_FIXTURE/jobs.json"
        [[ "${GH_FAIL:-}" != jobs-page2 ]] || exit 71
        jq -c '{total_count,items:(.jobs[1:])}' "$GH_FIXTURE/jobs.json"
      else
        jq -c '{total_count,items:.jobs}' "$GH_FIXTURE/jobs.json"
      fi
      exit
    fi
    if [[ "$endpoint" == */artifacts\?* ]]; then
      if [[ "${GH_MULTI_PAGE:-}" == 1 ]]; then
        jq -c '{total_count,items:(.artifacts[0:1])}' "$GH_FIXTURE/artifacts.json"
        [[ "${GH_FAIL:-}" != artifacts-page2 ]] || exit 72
        jq -c '{total_count,items:(.artifacts[1:])}' "$GH_FIXTURE/artifacts.json"
      else
        jq -c '{total_count,items:.artifacts}' "$GH_FIXTURE/artifacts.json"
      fi
      exit
    fi
    exit 73
    ;;
  run) exit 74 ;;
  *) exit 77 ;;
esac
GH
chmod +x "$tmp/bin/gh"
cat > "$tmp/bin/cockpit-release" <<'RELEASE_HELPER'
#!/usr/bin/env bash
set -euo pipefail
[[ "${1:-}" == acceptance-plan ]] || exit 81
output=''
while (($#)); do
  case "$1" in --output) output=$2; shift 2 ;; *) shift ;; esac
done
[[ -n "$output" ]] || exit 82
printf '{"actions":[]}\n' > "$output"
RELEASE_HELPER
chmod +x "$tmp/bin/cockpit-release"
export COCKPIT_RELEASE_BIN="$tmp/bin/cockpit-release"

write_jobs_skipped() {
  local stage=$1
  jq -n --arg stage "$stage" --arg runId "$GITHUB_RUN_ID" --arg head "$GITHUB_SHA" \
    '{total_count:1,jobs:[{id:501,run_id:($runId|tonumber),head_sha:$head,name:$stage,status:"completed",conclusion:"skipped",steps:[]}]}' \
    > "$GH_FIXTURE/jobs.json"
}
write_jobs_executed() {
  local stage=$1
  jq -n --arg stage "$stage" --arg runId "$GITHUB_RUN_ID" --arg head "$GITHUB_SHA" \
    '{total_count:1,jobs:[{id:501,run_id:($runId|tonumber),head_sha:$head,name:$stage,status:"completed",conclusion:"failure",steps:[{name:"Run acceptance",status:"completed",conclusion:"failure"}]}]}' \
    > "$GH_FIXTURE/jobs.json"
}
write_jobs_page2_skipped() {
  local stage=$1
  jq -n --arg stage "$stage" --arg runId "$GITHUB_RUN_ID" --arg head "$GITHUB_SHA" \
    '{total_count:2,jobs:[{id:499,run_id:($runId|tonumber),head_sha:$head,name:"unrelated job",status:"completed",conclusion:"success",steps:[]},{id:501,run_id:($runId|tonumber),head_sha:$head,name:$stage,status:"completed",conclusion:"skipped",steps:[]}]}' \
    > "$GH_FIXTURE/jobs.json"
}
write_jobs_duplicates_on_page2() {
  local stage=$1
  jq -n --arg stage "$stage" --arg runId "$GITHUB_RUN_ID" --arg head "$GITHUB_SHA" \
    '{total_count:3,jobs:[{id:499,run_id:($runId|tonumber),head_sha:$head,name:"unrelated job",status:"completed",conclusion:"success",steps:[]},{id:501,run_id:($runId|tonumber),head_sha:$head,name:$stage,status:"completed",conclusion:"skipped",steps:[]},{id:502,run_id:($runId|tonumber),head_sha:$head,name:$stage,status:"completed",conclusion:"skipped",steps:[]}]}' \
    > "$GH_FIXTURE/jobs.json"
}
write_no_artifacts() { printf '{"total_count":0,"artifacts":[]}\n' > "$GH_FIXTURE/artifacts.json"; }
write_artifact_zip() {
  python3 - "$GH_FIXTURE/artifact" "$GH_FIXTURE/artifact.zip" <<'PY'
import pathlib
import sys
import zipfile

root = pathlib.Path(sys.argv[1])
with zipfile.ZipFile(sys.argv[2], "w", compression=zipfile.ZIP_DEFLATED) as archive:
    for path in sorted(root.rglob("*")):
        if path.is_file():
            archive.write(path, path.relative_to(root).as_posix())
PY
}

stage=''
output=''
restore() { "$helper" restore --stage "$stage" --output "$output"; }
seal_artifact() {
  local target=$1
  mkdir -p "$target/.resume-cache"
  cp "$tmp/candidate/$archive" "$target/.resume-cache/$archive"
  cp "$tmp/candidate/release-manifest.json" "$target/.resume-cache/release-manifest.json"
  cp "$tmp/candidate/SHA256SUMS" "$target/.resume-cache/SHA256SUMS"
  cache_prefix=''
  old_manifest_sha=''
  if [[ "$stage" == staged_adopter_upgrade_acceptance ]]; then
    cache_prefix=to-
    printf 'previous release bytes\n' > "$target/.resume-cache/from-$old_archive"
    jq -n --arg tag "$FROM_TAG" --arg archive "$old_archive" --arg digest "$old_archive_sha" \
      '{version:"1.0.0",tag:$tag,artifacts:[{target:"x86_64-unknown-linux-gnu",archive:{filename:$archive,sha256:$digest}}]}' \
      > "$target/.resume-cache/from-release-manifest.json"
    old_manifest_sha="$(sha "$target/.resume-cache/from-release-manifest.json")"
    printf '%s  %s\n' "$old_archive_sha" "$old_archive" > "$target/.resume-cache/from-SHA256SUMS"
  fi
  if [[ -n "$cache_prefix" ]]; then
    mv "$target/.resume-cache/$archive" "$target/.resume-cache/$cache_prefix$archive"
    mv "$target/.resume-cache/release-manifest.json" "$target/.resume-cache/${cache_prefix}release-manifest.json"
    mv "$target/.resume-cache/SHA256SUMS" "$target/.resume-cache/${cache_prefix}SHA256SUMS"
  fi
  jq -n --arg source "$RELEASE_SOURCE_REVISION" --arg tag "$TO_TAG" \
    --arg manifest "sha256:$(sha "$tmp/candidate/release-manifest.json")" \
    --arg archive "$archive" --arg digest "sha256:$candidate_archive_sha" \
    --arg from "${FROM_TAG:-}" --arg oldArchive "$old_archive" --arg oldDigest "sha256:$old_archive_sha" --arg oldManifest "sha256:$old_manifest_sha" \
    --arg output "$target" \
    '{source:{commit:$source},candidate:{tag:$tag,version:"1.0.1",manifestDigest:$manifest,assets:{($archive):$digest}},previous:(if $from == "" then null else {tag:$from,version:"1.0.0",manifestDigest:$oldManifest,assets:{($oldArchive):$oldDigest}} end),target:"x86_64-unknown-linux-gnu",isolation:{home:($output+"/.resume-scope/home")}}' \
    > "$target/phase-identity.json"
  jq -n --slurpfile identity "$target/phase-identity.json" \
    '{schemaVersion:1,identity:$identity[0],identityDigest:("sha256:" + ("a" * 64)),results:[{phase:"prepare",status:"succeeded"}]}' \
    > "$target/phase-receipts.json"
  printf '{"adopterAcceptance":"failed"}\n' > "$target/acceptance.json"
  "$helper" seal --stage "$stage" --output "$target"
  write_artifact_zip
}
write_valid_artifact() {
  local name
  if [[ "$stage" == staged_adopter_acceptance ]]; then
    name="staged-adopter-acceptance-$GITHUB_RUN_ID-1"
  else
    name="staged-adopter-upgrade-$GITHUB_RUN_ID-1"
  fi
  export GH_EXPECTED_ARTIFACT="$name"
  rm -rf "$GH_FIXTURE/artifact"
  mkdir -p "$GH_FIXTURE/artifact"
  local current_attempt=$GITHUB_RUN_ATTEMPT
  GITHUB_RUN_ATTEMPT=1
  seal_artifact "$GH_FIXTURE/artifact"
  GITHUB_RUN_ATTEMPT=$current_attempt
  export GH_EXPECTED_ARTIFACT_ID=44
  jq -n --arg name "$name" --arg head "$GITHUB_SHA" \
    '{total_count:2,artifacts:[{id:43,name:"unrelated-artifact",expired:false,size_in_bytes:128,workflow_run:{id:173,head_sha:$head}},{id:44,name:$name,expired:false,size_in_bytes:128,workflow_run:{id:173,head_sha:$head}}]}' \
    > "$GH_FIXTURE/artifacts.json"
  printf '{"total_count":0,"jobs":[]}\n' > "$GH_FIXTURE/jobs.json"
}

for stage in staged_adopter_acceptance staged_adopter_upgrade_acceptance; do
  if [[ "$stage" == staged_adopter_upgrade_acceptance ]]; then export FROM_TAG=v1.0.0; else unset FROM_TAG; fi
  output="$tmp/output-$stage"
  rm -rf "$output"
  write_jobs_page2_skipped "$stage"
  write_no_artifacts
  unset GH_FAIL || true
  export GH_MULTI_PAGE=1
  GITHUB_OUTPUT="$tmp/fresh-output"
  : > "$GITHUB_OUTPUT"
  restore >/dev/null
  unset GH_MULTI_PAGE
  grep -Fxq 'resume=false' "$GITHUB_OUTPUT" || fail "$stage did not select a fresh run after an exact skipped/no-artifact attempt"

  printf '{"total_count":0,"jobs":[]}\n' > "$GH_FIXTURE/jobs.json"
  output="$tmp/never-ran-$stage"
  GITHUB_OUTPUT="$tmp/never-ran-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage empty exact-attempt job list" 'no stage job record; cannot establish that the stage was skipped'

  write_jobs_page2_skipped "$stage"
  jq '.jobs = .jobs[0:1] | .total_count = 1' "$GH_FIXTURE/jobs.json" > "$GH_FIXTURE/jobs.tmp"
  mv "$GH_FIXTURE/jobs.tmp" "$GH_FIXTURE/jobs.json"
  output="$tmp/missing-stage-job-$stage"
  GITHUB_OUTPUT="$tmp/missing-stage-job-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage missing stage job among other jobs" 'no stage job record; cannot establish that the stage was skipped'

  write_jobs_skipped "$stage"
  jq 'del(.jobs[0].run_id)' "$GH_FIXTURE/jobs.json" > "$GH_FIXTURE/jobs.tmp"
  mv "$GH_FIXTURE/jobs.tmp" "$GH_FIXTURE/jobs.json"
  output="$tmp/missing-run-id-$stage"
  GITHUB_OUTPUT="$tmp/missing-run-id-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage missing job run_id" 'stage job identity is missing or mismatched'

  write_jobs_skipped "$stage"
  jq '.jobs[0].run_id = 174' "$GH_FIXTURE/jobs.json" > "$GH_FIXTURE/jobs.tmp"
  mv "$GH_FIXTURE/jobs.tmp" "$GH_FIXTURE/jobs.json"
  output="$tmp/mismatched-run-id-$stage"
  GITHUB_OUTPUT="$tmp/mismatched-run-id-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage mismatched job run_id" 'stage job identity is missing or mismatched'

  write_jobs_skipped "$stage"
  jq 'del(.jobs[0].head_sha)' "$GH_FIXTURE/jobs.json" > "$GH_FIXTURE/jobs.tmp"
  mv "$GH_FIXTURE/jobs.tmp" "$GH_FIXTURE/jobs.json"
  output="$tmp/missing-head-$stage"
  GITHUB_OUTPUT="$tmp/missing-head-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage missing job head_sha" 'stage job identity is missing or mismatched'

  write_jobs_skipped "$stage"
  jq '.jobs[0].head_sha = "ffffffffffffffffffffffffffffffffffffffff"' "$GH_FIXTURE/jobs.json" > "$GH_FIXTURE/jobs.tmp"
  mv "$GH_FIXTURE/jobs.tmp" "$GH_FIXTURE/jobs.json"
  output="$tmp/mismatched-head-$stage"
  GITHUB_OUTPUT="$tmp/mismatched-head-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage mismatched job head_sha" 'stage job identity is missing or mismatched'

  write_jobs_skipped "$stage"
  # The attempt number is carried by the exact attempt-specific API endpoint.
  # If GitHub ever includes run_attempt in a job object, a contradiction fails closed.
  jq '.jobs[0].run_attempt = 2' "$GH_FIXTURE/jobs.json" > "$GH_FIXTURE/jobs.tmp"
  mv "$GH_FIXTURE/jobs.tmp" "$GH_FIXTURE/jobs.json"
  output="$tmp/mismatched-run-attempt-$stage"
  GITHUB_OUTPUT="$tmp/mismatched-run-attempt-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage mismatched optional job run_attempt" 'stage job identity is missing or mismatched'

  write_jobs_duplicates_on_page2 "$stage"
  write_no_artifacts
  export GH_MULTI_PAGE=1
  output="$tmp/duplicate-jobs-$stage"
  GITHUB_OUTPUT="$tmp/duplicate-jobs-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage duplicate stage jobs on page 2" 'duplicate stage jobs'
  unset GH_MULTI_PAGE

  write_jobs_page2_skipped "$stage"
  write_no_artifacts
  export GH_MULTI_PAGE=1 GH_FAIL=jobs-page2
  output="$tmp/jobs-page2-failed-$stage"
  GITHUB_OUTPUT="$tmp/jobs-page2-failed-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage jobs page 2 API failure" 'exact prior-attempt jobs API query failed or returned incomplete pagination'
  unset GH_MULTI_PAGE GH_FAIL

  write_jobs_skipped "$stage"
  write_no_artifacts
  export GH_MULTI_PAGE=1 GH_FAIL=artifacts-page2
  output="$tmp/artifacts-page2-failed-$stage"
  GITHUB_OUTPUT="$tmp/artifacts-page2-failed-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage artifacts page 2 API failure" 'prior-run artifact API query failed or returned incomplete pagination'
  unset GH_MULTI_PAGE GH_FAIL

  write_valid_artifact
  output="$tmp/resumed-$stage"
  rm -rf "$output"
  GITHUB_OUTPUT="$tmp/resume-output"
  : > "$GITHUB_OUTPUT"
  export GH_MULTI_PAGE=1
  restore >/dev/null
  unset GH_MULTI_PAGE
  grep -Fxq 'resume=true' "$GITHUB_OUTPUT" || fail "$stage did not resume a valid exact artifact"
  [[ -f "$output/phase-receipts.json" ]] || fail "$stage valid receipt was not restored"

  write_valid_artifact
  python3 - "$GH_FIXTURE/artifact.zip" <<'PY'
import sys
import zipfile

with zipfile.ZipFile(sys.argv[1], "w", compression=zipfile.ZIP_DEFLATED) as archive:
    archive.writestr("../escaped", b"must not leave the extraction root")
PY
  output="$tmp/unsafe-archive-$stage"
  GITHUB_OUTPUT="$tmp/unsafe-archive-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage unsafe artifact archive path" 'artifact archive is invalid or unsafe to extract'
  [[ ! -e "$RUNNER_TEMP/escaped" ]] || fail "$stage extracted an archive path outside its temporary root"

  # A later skipped attempt cannot establish that earlier attempts had no
  # receipt, so absence must fail closed after attempt 2.
  GITHUB_RUN_ATTEMPT=3
  write_jobs_skipped "$stage"
  write_no_artifacts
  output="$tmp/attempt3-$stage"
  GITHUB_OUTPUT="$tmp/attempt3-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure "$stage attempt 3 without exact artifact"
  GITHUB_RUN_ATTEMPT=2

  write_jobs_executed "$stage"
  write_no_artifacts
  output="$tmp/missing-after-run-$stage"
  GITHUB_OUTPUT="$tmp/missing-after-run-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure "$stage missing artifact after execution"

  write_jobs_skipped "$stage"
  write_valid_artifact
  output="$tmp/expired-$stage"
  GITHUB_OUTPUT="$tmp/expired-output"
  : > "$GITHUB_OUTPUT"
  jq '.artifacts[1].expired = true' "$GH_FIXTURE/artifacts.json" > "$GH_FIXTURE/artifacts.tmp"
  mv "$GH_FIXTURE/artifacts.tmp" "$GH_FIXTURE/artifacts.json"
  expect_restore_failure "$stage expired artifact"

  write_valid_artifact
  jq '.artifacts += [.artifacts[1]] | .total_count = (.artifacts | length)' "$GH_FIXTURE/artifacts.json" > "$GH_FIXTURE/artifacts.tmp"
  mv "$GH_FIXTURE/artifacts.tmp" "$GH_FIXTURE/artifacts.json"
  export GH_MULTI_PAGE=1
  output="$tmp/duplicate-$stage"
  GITHUB_OUTPUT="$tmp/duplicate-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage duplicate exact artifacts across pages" 'more than one artifact matches the exact prior-attempt name'
  unset GH_MULTI_PAGE

  RELEASE_PLAN_DIGEST=sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
  write_valid_artifact
  RELEASE_PLAN_DIGEST=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
  output="$tmp/wrong-plan-$stage"
  GITHUB_OUTPUT="$tmp/wrong-plan-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure_reason "$stage plan mismatch" 'artifact phase binding does not match this run, stage, workflow head, plan, source, or prior attempt'

  write_valid_artifact
  cache_archive_path="$GH_FIXTURE/artifact/.resume-cache/$archive"
  [[ "$stage" != staged_adopter_upgrade_acceptance ]] || cache_archive_path="$GH_FIXTURE/artifact/.resume-cache/to-$archive"
  printf 'tampered cache\n' >> "$cache_archive_path"
  write_artifact_zip
  output="$tmp/tampered-cache-$stage"
  GITHUB_OUTPUT="$tmp/tampered-cache-output"
  : > "$GITHUB_OUTPUT"
  expect_restore_failure "$stage cached candidate tampering"

  write_valid_artifact
  output="$tmp/download-failed-$stage"
  GITHUB_OUTPUT="$tmp/download-failed-output"
  : > "$GITHUB_OUTPUT"
  export GH_FAIL=download
  expect_restore_failure_reason "$stage artifact download failure" 'exact prior-attempt artifact download by ID failed'
  unset GH_FAIL

  output="$tmp/api-failed-$stage"
  GITHUB_OUTPUT="$tmp/api-failed-output"
  : > "$GITHUB_OUTPUT"
  export GH_FAIL=jobs
  expect_restore_failure "$stage exact-attempt API failure"
  unset GH_FAIL

  if [[ "$stage" == staged_adopter_upgrade_acceptance ]]; then
    unset FROM_TAG
    output="$tmp/not-applicable-output"
    rm -rf "$output" "$GH_FIXTURE/artifact"
    mkdir -p "$output" "$GH_FIXTURE/artifact"
    jq -n --arg tag "$TO_TAG" \
      '{schemaVersion:1,releasePublished:false,stagedCandidate:true,adopterAcceptance:"not_applicable",fromTag:null,toTag:$tag,target:"x86_64-unknown-linux-gnu",reason:"no previous published Release"}' \
      > "$output/acceptance.json"
    local_attempt=$GITHUB_RUN_ATTEMPT
    GITHUB_RUN_ATTEMPT=1
    "$helper" seal --stage "$stage" --output "$output" >/dev/null
    GITHUB_RUN_ATTEMPT=$local_attempt
    cp -a "$output/." "$GH_FIXTURE/artifact/"
    write_artifact_zip
    write_jobs_skipped "$stage"
    jq -n --arg name "$GH_EXPECTED_ARTIFACT" --arg head "$GITHUB_SHA" \
      '{total_count:2,artifacts:[{id:43,name:"unrelated-artifact",expired:false,size_in_bytes:128,workflow_run:{id:173,head_sha:$head}},{id:44,name:$name,expired:false,size_in_bytes:128,workflow_run:{id:173,head_sha:$head}}]}' \
      > "$GH_FIXTURE/artifacts.json"
    output="$tmp/not-applicable-restored"
    rm -rf "$output"
    GITHUB_OUTPUT="$tmp/not-applicable-resume-output"
    : > "$GITHUB_OUTPUT"
    export GH_MULTI_PAGE=1
    restore >/dev/null
    unset GH_MULTI_PAGE
    grep -Fxq 'resume=true' "$GITHUB_OUTPUT" || fail 'N-1 not-applicable evidence did not restore on a valid rerun'
    jq -e '.adopterAcceptance == "not_applicable"' "$output/acceptance.json" >/dev/null || fail 'N-1 not-applicable receipt was not restored'
  fi
done

echo 'staged release attempt restore regression passed'
