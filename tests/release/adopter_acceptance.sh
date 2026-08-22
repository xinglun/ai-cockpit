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
  [--source-repo DIRECTORY]
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

repository=''
tag=''
target=''
output=''
source_repo=''

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
source_repo="$(cd "$source_repo" && pwd)"
git -C "$source_repo" rev-parse --show-toplevel >/dev/null 2>&1 || die 'source repository is not a Git checkout'

mkdir -p "$output"
output="$(cd "$output" && pwd)"
if [[ -n "$(find "$output" -mindepth 1 -print -quit 2>/dev/null)" ]]; then
  die "output directory must be empty: $output"
fi

tmpdir=''
if tmpdir="$(printenv TMPDIR)"; then :; fi
[[ -n "$tmpdir" ]] || tmpdir=/tmp
run_parent="$(cd "$tmpdir" 2>/dev/null && pwd -P)" || die "TMPDIR is not a directory: $tmpdir"
run_root="$(mktemp -d "$run_parent/ai-cockpit-adopter-acceptance.XXXXXX")"
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
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
finished_at=''
overall_state=failed
failure_reason=''
release_published=false
runtime_version=''
runtime_digest=''
repository_id=''
source_repository_id=''
source_ai_state=unknown
source_before_status=''
source_after_status=''
runtime_bin=''
rustup_home=''
cleanup_state=not_started
cleanup_removed=false
cleanup_validated=false
cleanup_reason=''
if rustup_home="$(printenv RUSTUP_HOME)"; then :; fi
if [[ -z "$rustup_home" ]] && command -v rustup >/dev/null 2>&1; then
  if rustup_home="$(rustup show home 2>/dev/null)"; then :; fi
fi

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

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}

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

  [[ -n "${run_root:-}" && -n "${run_parent:-}" ]] || {
    cleanup_reason='run_root cleanup path was not initialized'
    return 1
  }
  [[ -d "$run_parent" ]] || {
    cleanup_reason='run_root parent directory is missing'
    return 1
  }
  [[ -d "$run_root" ]] || {
    cleanup_state=passed
    cleanup_removed=true
    cleanup_validated=true
    cleanup_reason='run_root was already absent'
    return 0
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
  if rm -rf -- "$root_real" && [[ ! -e "$root_real" ]]; then
    cleanup_state=passed
    cleanup_removed=true
    cleanup_reason='validated run_root removed'
    return 0
  fi
  cleanup_reason='validated run_root removal failed'
  return 1
}

write_cleanup_receipt() {
  jq -n \
    --arg state "$cleanup_state" \
    --arg reason "$cleanup_reason" \
    --argjson removed "$cleanup_removed" \
    --argjson validated "$cleanup_validated" \
    '{schemaVersion:1,kind:"run_root_cleanup",state:$state,removed:$removed,validated:$validated,reason:(if $reason == "" then null else $reason end)}' \
    > "$output/cleanup.json"
}

update_acceptance_cleanup() {
  local updated="$output/.acceptance.json.cleanup.tmp"
  if jq \
    --arg state "$cleanup_state" \
    --arg reason "$cleanup_reason" \
    '.cleanupState = $state | .cleanupError = (if $state == "failed" then $reason else null end)' \
    "$output/acceptance.json" > "$updated" && mv -f -- "$updated" "$output/acceptance.json"; then
    return 0
  fi
  printf 'adopter acceptance cleanup warning: acceptance cleanup metadata could not be updated\n' >&2
  return 1
}

finalize() {
  local exit_code=$?
  set +e
  finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [[ "$exit_code" -eq 0 ]]; then
    overall_state=passed
  else
    overall_state=failed
    [[ -n "$failure_reason" ]] || failure_reason="command exited with status $exit_code"
  fi
  local steps='[]'
  if [[ -s "$steps_jsonl" ]]; then
    steps="$(jq -s '.' "$steps_jsonl")"
  fi
  jq -n \
    --arg startedAt "$started_at" \
    --arg finishedAt "$finished_at" \
    --arg state "$overall_state" \
    --arg releasePublished "$release_published" \
    --arg repository "$repository" \
    --arg tag "$tag" \
    --arg target "$target" \
    --arg runtimeVersion "$runtime_version" \
    --arg runtimeDigest "$runtime_digest" \
    --arg repositoryId "$repository_id" \
    --arg sourceRepositoryId "$source_repository_id" \
    --arg failureReason "$failure_reason" \
    --argjson steps "$steps" \
    '{
      schemaVersion: 1,
      startedAt: $startedAt,
      finishedAt: $finishedAt,
      releasePublished: ($releasePublished == "true"),
      adopterAcceptance: $state,
      repository: $repository,
      tag: $tag,
      target: $target,
      runtimeVersion: (if $runtimeVersion == "" then null else $runtimeVersion end),
      runtimeDigest: (if $runtimeDigest == "" then null else $runtimeDigest end),
      repositoryId: (if $repositoryId == "" then null else $repositoryId end),
      sourceRepositoryId: (if $sourceRepositoryId == "" then null else $sourceRepositoryId end),
      cleanupState: "pending",
      cleanupError: null,
      steps: $steps,
      failureReason: (if $failureReason == "" then null else $failureReason end)
    }' > "$output/acceptance.json"
  write_sums
  cleanup_run_root
  update_acceptance_cleanup
  write_cleanup_receipt
  write_sums
  if [[ "$cleanup_state" == failed ]]; then
    printf 'adopter acceptance cleanup warning: %s\n' "$cleanup_reason" >&2
  fi
  exit "$exit_code"
}
trap finalize EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

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

release_url="https://github.com/$repository/releases/tag/$tag"
api_url="https://api.github.com/repos/$repository/releases/tags/$tag"
release_api="$output/release.json"
if ! curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$api_url" > "$release_api"; then
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

version="$(printf '%s' "$tag" | sed 's/^v//')"
archive_name="ai-cockpit-$tag-$target.$archive_extension"
manifest_name=release-manifest.json
sums_name=SHA256SUMS
archive_url="$(jq -er --arg name "$archive_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
manifest_url="$(jq -er --arg name "$manifest_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
sums_url="$(jq -er --arg name "$sums_name" '.assets[] | select(.name == $name) | .browser_download_url' "$release_api")"
[[ "$archive_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die 'archive URL is outside the requested public Release'
[[ "$manifest_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die 'manifest URL is outside the requested public Release'
[[ "$sums_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die 'checksum URL is outside the requested public Release'

archive_path="$download_root/$archive_name"
manifest_path="$download_root/$manifest_name"
sums_path="$download_root/$sums_name"
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$archive_url" -o "$archive_path"
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$manifest_url" -o "$manifest_path"
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$sums_url" -o "$sums_path"
cp "$manifest_path" "$output/release-manifest.json"
cp "$sums_path" "$output/SHA256SUMS.release"
manifest_archive_digest="$(jq -er --arg target "$target" '.artifacts[] | select(.target == $target) | .archive.sha256' "$manifest_path")"
sums_archive_digest="$(awk -v name="$archive_name" '$2 == name {print $1}' "$sums_path")"
actual_archive_digest="$(sha256_file "$archive_path")"
[[ "$manifest_archive_digest" == "$actual_archive_digest" ]] || die 'archive digest does not match release manifest'
[[ "$sums_archive_digest" == "$actual_archive_digest" ]] || die 'archive digest does not match SHA256SUMS'
[[ "$(jq -er '.version' "$manifest_path")" == "$version" ]] || die 'manifest version does not match tag'
[[ "$(jq -er '.tag' "$manifest_path")" == "$tag" ]] || die 'manifest tag does not match requested tag'
mark_passed release-download

if [[ "$archive_extension" == tar.gz ]]; then
  tar -xzf "$archive_path" -C "$runtime_root"
else
  unzip -q "$archive_path" -d "$runtime_root"
fi
runtime_bin="$runtime_root/ai-cockpit"
if [[ "$archive_extension" == zip ]]; then runtime_bin="$runtime_root/ai-cockpit.exe"; fi
[[ -f "$runtime_bin" && -x "$runtime_bin" ]] || die 'public Release archive did not contain an executable Runtime'
runtime_version="$("$runtime_bin" --version | awk '{print $2}')"
runtime_digest="sha256:$(sha256_file "$runtime_bin")"
[[ "$runtime_version" == "$version" ]] || die 'downloaded Runtime version does not match Release tag'
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
  '{schemaVersion:1,tag:$tag,version:$version,target:$target,platform:$platform,archive:$archive,archiveDigest:$archiveDigest,binaryDigest:$binaryDigest,downloadSource:$downloadSource,releaseUrl:$releaseUrl,manifestDigest:$manifestDigest,releasePublished:true}' > "$output/runtime.json"
mark_passed runtime-pin

source_before_status="$(git -C "$source_repo" status --porcelain=v1)"
if [[ -e "$source_repo/.ai" ]]; then source_ai_state=present; else source_ai_state=absent; fi
source_repository_id=''
if source_repository_id="$(jq -er '.repositoryId' "$source_repo/.ai/project.json" 2>/dev/null)"; then :; fi
find "$isolated_home" -type f -print | LC_ALL=C sort > "$run_root/home-before.manifest"
find "$isolated_xdg" -type f -print | LC_ALL=C sort > "$run_root/xdg-before.manifest"

env -i HOME="$isolated_home" XDG_CONFIG_HOME="$isolated_xdg" TMPDIR="$isolated_tmp" CARGO_HOME="$isolated_cargo" RUSTUP_HOME="$rustup_home" PATH="$PATH" LANG=C LC_ALL=C cargo new --lib --vcs none "$adopter_root" >/dev/null
printf 'target/\n' > "$adopter_root/.gitignore"
env -i HOME="$isolated_home" XDG_CONFIG_HOME="$isolated_xdg" TMPDIR="$isolated_tmp" CARGO_HOME="$isolated_cargo" RUSTUP_HOME="$rustup_home" PATH="$PATH" LANG=C LC_ALL=C cargo generate-lockfile --manifest-path "$adopter_root/Cargo.toml" >/dev/null
git -C "$adopter_root" init -q
git -C "$adopter_root" config user.name 'AI Cockpit Release Acceptance'
git -C "$adopter_root" config user.email 'ai-cockpit-release-acceptance@example.invalid'
git -C "$adopter_root" add .
git -C "$adopter_root" commit -qm 'initial adopter scaffold'
mark_passed adopter-scaffold

: > "$adopter_root/AGENTS.md"
capture_runtime attach.json attach --repo "$adopter_root"
capture_runtime inspect.json inspect --repo "$adopter_root"
inspect_runtime_version="$(jq -er '.runtimeVersion' "$output/inspect.json")"
inspect_runtime_digest="$(jq -er '.runtimeDigest' "$output/inspect.json")"
[[ "$inspect_runtime_version" == "$runtime_version" && "$inspect_runtime_digest" == "$runtime_digest" ]] || die 'inspect Runtime identity does not match downloaded binary'
mark_passed inspect-runtime-identity
capture_runtime profile-confirm.json profile confirm --repo "$adopter_root" --program cargo --args test,--workspace
capture_runtime agent-list.json agent list --repo "$adopter_root"
capture_runtime agent-install.json agent install --repo "$adopter_root" --provider auto
capture_runtime agent-doctor.json agent doctor --repo "$adopter_root" --json
jq -e '.state == "VERIFIED" and .repositoryId != null and (.problems | length == 0)' "$output/agent-doctor.json" >/dev/null || die 'Agent doctor did not verify the fresh adopter'
mark_passed agent-doctor-assertion

capture_runtime first-adopter-smoke.json work-item new --repo "$adopter_root" --id first-adopter-smoke --mode code
first_smoke_contract="$adopter_root/.ai/work-items/active/first-adopter-smoke.contract.json"
[[ -f "$first_smoke_contract" ]] || die 'first-adopter-smoke contract was not created'
mkdir -p "$output/work-items"
cp "$first_smoke_contract" "$output/work-items/first-adopter-smoke.contract.json"
jq -e '.state == "not_ready" and .intent == "" and (.scope | length == 0) and (.acceptanceCriteria | length == 0) and .authority == "unknown"' "$first_smoke_contract" >/dev/null || die 'first-adopter-smoke was not preserved as not_ready'
mark_passed first-adopter-smoke-assertion

git -C "$adopter_root" add .
git -C "$adopter_root" commit -qm 'attach adopter governance state'

capture_runtime verify-first.json verify --repo "$adopter_root" --workers 1
jq -e '.passed == true and .nodesExecuted >= 1 and .nodesReused == 0 and .processesSpawned >= 1' "$output/verify-first.json" >/dev/null || die 'first verification did not execute a process'
capture_runtime verify-reuse.json verify --repo "$adopter_root" --workers 1
jq -e '.passed == true and .nodesReused >= 1 and .nodesExecuted == 0 and .processesSpawned == 0' "$output/verify-reuse.json" >/dev/null || die 'second verification did not reuse evidence with zero spawns'
mark_passed reuse-assertion

lifecycle_id=release-adopter-lifecycle
capture_runtime lifecycle-start.json start --repo "$adopter_root" --id "$lifecycle_id" \
  --intent 'Validate the published Runtime against a real adopter change.' \
  --goal 'Demonstrate an auditable Work Item lifecycle using only the pinned public Release binary.' \
  --scope src/lib.rs --out-of-scope target --risk normal --authority authorized \
  --acceptance 'cargo test passes for the adopter change' --required-evidence verification
lifecycle_contract="$adopter_root/.ai/work-items/active/$lifecycle_id.contract.json"
[[ -f "$lifecycle_contract" ]] || die 'lifecycle contract was not created'
printf '\n// release adopter acceptance mutation\n' >> "$adopter_root/src/lib.rs"
git -C "$adopter_root" add src/lib.rs
git -C "$adopter_root" commit -qm 'make deterministic adopter change'
capture_runtime lifecycle-checkpoint.json checkpoint --repo "$adopter_root" --id "$lifecycle_id"
capture_runtime lifecycle-preflight.json preflight --repo "$adopter_root" --contract "$lifecycle_contract"
capture_runtime lifecycle-verify.json verify --repo "$adopter_root" --work-item "$lifecycle_id" --workers 1
cp "$adopter_root/.ai/evidence/$lifecycle_id.verification.json" "$output/work-items/lifecycle.evidence.json"
jq -e --arg version "$runtime_version" --arg digest "$runtime_digest" '.runtimeVersion == $version and .runtimeDigest == $digest and .passed == true' "$output/work-items/lifecycle.evidence.json" >/dev/null || die 'Work Item verification evidence is not bound to the downloaded Runtime'
mark_passed lifecycle-runtime-identity
capture_runtime lifecycle-finish.json finish --repo "$adopter_root" --id "$lifecycle_id"
capture_runtime lifecycle-archive.json archive --repo "$adopter_root" --id "$lifecycle_id"
capture_runtime lifecycle-close.json close --repo "$adopter_root" --id "$lifecycle_id" --human-decision approved
for lifecycle_file in \
  "$adopter_root/.ai/work-items/archive/$lifecycle_id.contract.json" \
  "$adopter_root/.ai/work-items/archive/$lifecycle_id.outcome.json" \
  "$adopter_root/.ai/work-items/archive/$lifecycle_id.summary.json"; do
  [[ -f "$lifecycle_file" ]] || die "missing archived lifecycle evidence: $lifecycle_file"
  cp "$lifecycle_file" "$output/work-items/$(basename "$lifecycle_file")"
done
mark_passed lifecycle-assertion

source_after_status="$(git -C "$source_repo" status --porcelain=v1)"
find "$isolated_home" -type f -print | LC_ALL=C sort > "$run_root/home-after.manifest"
find "$isolated_xdg" -type f -print | LC_ALL=C sort > "$run_root/xdg-after.manifest"
home_unchanged=true
xdg_unchanged=true
cmp -s "$run_root/home-before.manifest" "$run_root/home-after.manifest" || home_unchanged=false
cmp -s "$run_root/xdg-before.manifest" "$run_root/xdg-after.manifest" || xdg_unchanged=false
[[ "$source_before_status" == "$source_after_status" ]] || die 'acceptance modified the source checkout'
if [[ "$source_ai_state" == present ]]; then
  [[ -d "$source_repo/.ai" ]] || die 'acceptance changed source repository .ai state'
else
  [[ ! -e "$source_repo/.ai" ]] || die 'acceptance created .ai in an initially unattached source checkout'
fi
jq -n \
  --arg sourceRepository "$source_repo" \
  --arg sourceAiState "$source_ai_state" \
  --arg sourceBeforeStatus "$source_before_status" \
  --arg sourceAfterStatus "$source_after_status" \
  --arg adopterRepository "$adopter_root" \
  --argjson homeUnchanged "$home_unchanged" \
  --argjson xdgUnchanged "$xdg_unchanged" \
  '{schemaVersion:1,sourceRepository:$sourceRepository,sourceAiState:$sourceAiState,sourceBeforeStatus:$sourceBeforeStatus,sourceAfterStatus:$sourceAfterStatus,adopterRepository:$adopterRepository,homeManifest:{unchanged:$homeUnchanged},xdgConfigManifest:{unchanged:$xdgUnchanged},sourceUnchanged:($sourceBeforeStatus == $sourceAfterStatus),repositoryIsolation:($sourceRepository != $adopterRepository)}' > "$output/isolation.json"
[[ "$home_unchanged" == true && "$xdg_unchanged" == true ]] || die 'isolated HOME/XDG changed during acceptance'
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

overall_state=passed
failure_reason=''
