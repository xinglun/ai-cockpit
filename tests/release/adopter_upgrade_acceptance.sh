#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: adopter_upgrade_acceptance.sh --repository OWNER/REPOSITORY --from-tag vX.Y.Z --to-tag vX.Y.Z --target TARGET --output DIRECTORY [--source-repo DIRECTORY]
USAGE
}
die() { failure_reason="$*"; printf 'adopter upgrade acceptance failed: %s\n' "$failure_reason" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "required command is unavailable: $1"; }
source "$(cd "$(dirname "$0")" && pwd)/isolation_manifest.sh"

write_sums() {
  : > "$output/SHA256SUMS"
  find "$output" -type f ! -name SHA256SUMS -print | LC_ALL=C sort | while IFS= read -r path; do
    printf '%s  %s\n' "$(sha256_file "$path")" "$(printf '%s' "$path" | sed "s#^$output/##")" >> "$output/SHA256SUMS"
  done
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
    "$parent_real"/ai-cockpit-n-minus-one.*) ;;
    *)
      cleanup_reason='run_root name or parent did not match the N-1 temp boundary'
      return 1
      ;;
  esac
  [[ "$root_name" == ai-cockpit-n-minus-one.* ]] || {
    cleanup_reason='run_root basename did not match the N-1 temp boundary'
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
  printf 'adopter upgrade acceptance cleanup warning: acceptance cleanup metadata could not be updated\n' >&2
  return 1
}

repository='' from_tag='' to_tag='' target='' output='' source_repo=''
while (($# > 0)); do
  case "$1" in
    --repository) [[ $# -ge 2 ]] || die "--repository requires a value"; repository="$2"; shift 2 ;;
    --from-tag) [[ $# -ge 2 ]] || die "--from-tag requires a value"; from_tag="$2"; shift 2 ;;
    --to-tag) [[ $# -ge 2 ]] || die "--to-tag requires a value"; to_tag="$2"; shift 2 ;;
    --target) [[ $# -ge 2 ]] || die "--target requires a value"; target="$2"; shift 2 ;;
    --output) [[ $# -ge 2 ]] || die "--output requires a value"; output="$2"; shift 2 ;;
    --source-repo) [[ $# -ge 2 ]] || die "--source-repo requires a value"; source_repo="$2"; shift 2 ;;
    --help|-h) usage; exit 0 ;;
    *) usage >&2; die "unknown argument: $1" ;;
  esac
done
[[ "$repository" =~ ^[^/]+/[^/]+$ ]] || die 'repository must be OWNER/REPOSITORY'
[[ "$from_tag" =~ ^v[0-9]+[.][0-9]+[.][0-9]+$ ]] || die 'from-tag must be vX.Y.Z'
[[ "$to_tag" =~ ^v[0-9]+[.][0-9]+[.][0-9]+$ ]] || die 'to-tag must be vX.Y.Z'
[[ "$from_tag" != "$to_tag" ]] || die 'from-tag and to-tag must be distinct immutable Releases'
[[ "$target" =~ ^(aarch64-apple-darwin|aarch64-unknown-linux-gnu|x86_64-apple-darwin|x86_64-pc-windows-msvc|x86_64-unknown-linux-gnu)$ ]] || die "unsupported target: $target"
[[ -n "$output" ]] || die '--output is required'
for command_name in bash curl jq git cargo tar; do need "$command_name"; done
if [[ "$target" == x86_64-pc-windows-msvc ]]; then need unzip; archive_ext=zip; else
  if ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then die 'SHA-256 implementation is unavailable'; fi
  archive_ext=tar.gz
fi
if [[ -z "$source_repo" ]]; then source_repo="$(git rev-parse --show-toplevel 2>/dev/null || true)"; fi
[[ -n "$source_repo" && -d "$source_repo" ]] || die 'source repository is unavailable'
source_repo="$(cd "$source_repo" && pwd)"
git -C "$source_repo" rev-parse --show-toplevel >/dev/null 2>&1 || die 'source repository is not a Git checkout'
mkdir -p "$output"; output="$(cd "$output" && pwd)"
[[ -z "$(find "$output" -mindepth 1 -print -quit 2>/dev/null)" ]] || die "output directory must be empty: $output"

tmp_parent="$(printenv TMPDIR 2>/dev/null || printf '/tmp')"; [[ -d "$tmp_parent" ]] || tmp_parent=/tmp
run_parent="$(cd "$tmp_parent" 2>/dev/null && pwd -P)" || die "TMPDIR is not a directory: $tmp_parent"
run_root="$(mktemp -d "$run_parent/ai-cockpit-n-minus-one.XXXXXX")"
download_root="$run_root/downloads"; from_root="$run_root/from"; to_root="$run_root/to"; adopter="$run_root/adopter"
isolated_home="$run_root/home"; isolated_xdg="$run_root/xdg"; isolated_tmp="$run_root/tmp"; isolated_cargo="$run_root/cargo"
mkdir -p "$download_root" "$from_root" "$to_root" "$isolated_home" "$isolated_xdg" "$isolated_tmp" "$isolated_cargo"
steps="$run_root/steps.jsonl"; : > "$steps"
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"; failure_reason=''; release_published=false
from_bin='' to_bin='' from_version='' to_version='' from_digest='' to_digest='' repository_id=''
cleanup_state=not_started cleanup_removed=false cleanup_validated=false cleanup_reason=''
if rustup_home="$(printenv RUSTUP_HOME 2>/dev/null)"; then :; else rustup_home=''; fi

record() {
  local name="$1" state="$2" reason=''
  if [[ $# -ge 3 ]]; then reason="$3"; fi
  jq -cn --arg name "$name" --arg state "$state" --arg reason "$reason" \
    '{name:$name,state:$state} + (if $reason == "" then {} else {reason:$reason} end)' >> "$steps"
}
pass() {
  local reason=''
  if [[ $# -ge 2 ]]; then reason="$2"; fi
  record "$1" passed "$reason"
}
finish() {
  local code=$? state=failed
  set +e
  if [[ "$code" -eq 0 ]]; then state=passed; else [[ -n "$failure_reason" ]] || failure_reason="command exited with status $code"; fi
  local step_json='[]'
  if [[ -s "$steps" ]]; then step_json="$(jq -s '.' "$steps")"; fi
  jq -n --arg startedAt "$started_at" --arg finishedAt "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg state "$state" --arg published "$release_published" --arg repository "$repository" \
    --arg fromTag "$from_tag" --arg toTag "$to_tag" --arg target "$target" \
    --arg fromVersion "$from_version" --arg toVersion "$to_version" --arg fromDigest "$from_digest" \
    --arg toDigest "$to_digest" --arg repositoryId "$repository_id" --arg reason "$failure_reason" \
    --argjson steps "$step_json" \
    '{schemaVersion:1,startedAt:$startedAt,finishedAt:$finishedAt,releasePublished:($published=="true"),adopterAcceptance:$state,repository:$repository,fromTag:$fromTag,toTag:$toTag,target:$target,fromRuntimeVersion:(if $fromVersion=="" then null else $fromVersion end),toRuntimeVersion:(if $toVersion=="" then null else $toVersion end),fromRuntimeDigest:(if $fromDigest=="" then null else $fromDigest end),toRuntimeDigest:(if $toDigest=="" then null else $toDigest end),repositoryId:(if $repositoryId=="" then null else $repositoryId end),cleanupState:"pending",cleanupError:null,steps:$steps,failureReason:(if $reason=="" then null else $reason end)}' > "$output/acceptance.json"
  write_sums
  cleanup_run_root
  update_acceptance_cleanup
  write_cleanup_receipt
  write_sums
  if [[ "$cleanup_state" == failed ]]; then
    printf 'adopter upgrade acceptance cleanup warning: %s\n' "$cleanup_reason" >&2
  fi
  exit "$code"
}
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

download() {
  local tag="$1" label="$2" root="$3" version archive api manifest sums actual expected url
  version="$(printf '%s' "$tag" | sed 's/^v//')"
  archive="ai-cockpit-$tag-$target.$archive_ext"
  api="$download_root/$label-release.json"; manifest="$download_root/$label-manifest.json"; sums="$download_root/$label-SHA256SUMS"
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "https://api.github.com/repos/$repository/releases/tags/$tag" > "$api" || die "$label Release API request failed"
  jq -e --arg tag "$tag" '.tag_name==$tag and (.draft==false) and (.prerelease==false)' "$api" >/dev/null || die "$label Release is not published"
  release_published=true
  url="$(jq -er --arg name "$archive" '.assets[]|select(.name==$name)|.browser_download_url' "$api")"
  local manifest_url sums_url
  manifest_url="$(jq -er '.assets[]|select(.name=="release-manifest.json")|.browser_download_url' "$api")"
  sums_url="$(jq -er '.assets[]|select(.name=="SHA256SUMS")|.browser_download_url' "$api")"
  [[ "$url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die "$label archive URL is not Release-bound"
  [[ "$manifest_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die "$label manifest URL is not Release-bound"
  [[ "$sums_url" == "https://github.com/$repository/releases/download/$tag/"* ]] || die "$label checksum URL is not Release-bound"
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$url" -o "$download_root/$archive"
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$manifest_url" -o "$manifest"
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$sums_url" -o "$sums"
  actual="$(sha256_file "$download_root/$archive")"; expected="$(jq -er --arg target "$target" '.artifacts[]|select(.target==$target)|.archive.sha256' "$manifest")"
  [[ "$actual" == "$expected" ]] || die "$label archive manifest digest mismatch"
  [[ "$(awk -v name="$archive" '$2==name {print $1}' "$sums")" == "$actual" ]] || die "$label archive SHA256SUMS mismatch"
  [[ "$(jq -er '.version' "$manifest")" == "$version" && "$(jq -er '.tag' "$manifest")" == "$tag" ]] || die "$label manifest identity mismatch"
  cp "$manifest" "$output/$label-release-manifest.json"; cp "$sums" "$output/$label-SHA256SUMS.release"
  if [[ "$archive_ext" == tar.gz ]]; then tar -xzf "$download_root/$archive" -C "$root"; else unzip -q "$download_root/$archive" -d "$root"; fi
  local binary="$root/ai-cockpit"; [[ "$archive_ext" == zip ]] && binary="$root/ai-cockpit.exe"
  [[ -f "$binary" && -x "$binary" ]] || die "$label archive has no executable"
  local binary_version binary_digest
  binary_version="$("$binary" --version | awk '{print $2}')"; binary_digest="sha256:$(sha256_file "$binary")"
  [[ "$binary_version" == "$version" ]] || die "$label binary version mismatch"
  jq -n --arg tag "$tag" --arg version "$version" --arg target "$target" --arg platform "$target" --arg archive "$archive" \
    --arg archiveDigest "sha256:$actual" --arg binaryDigest "$binary_digest" --arg downloadSource "$url" \
    '{schemaVersion:1,tag:$tag,version:$version,target:$target,platform:$platform,archive:$archive,archiveDigest:$archiveDigest,binaryDigest:$binaryDigest,downloadSource:$downloadSource,releasePublished:true}' > "$output/$label-runtime.json"
  if [[ "$label" == from ]]; then from_bin="$binary"; from_version="$binary_version"; from_digest="$binary_digest"; else to_bin="$binary"; to_version="$binary_version"; to_digest="$binary_digest"; fi
  pass "$label-release-pin"
}

run() {
  local binary="$1" name="$2"; shift 2
  local stem stderr result
  stem="$(printf '%s' "$name" | sed 's/\.json$//')"; stderr="$run_root/$stem.stderr"
  set +e
  env -i HOME="$isolated_home" XDG_CONFIG_HOME="$isolated_xdg" TMPDIR="$isolated_tmp" CARGO_HOME="$isolated_cargo" RUSTUP_HOME="$rustup_home" PATH="$PATH" LANG=C LC_ALL=C GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null "$binary" "$@" > "$output/$name" 2> "$stderr"
  result=$?; set -e
  [[ "$result" -eq 0 ]] || { cp "$stderr" "$output/$stem.stderr" 2>/dev/null || true; failure_reason="$name failed"; record "$name" failed "exit $result"; return "$result"; }
  pass "$name"
}
expected_fail() {
  local binary="$1" name="$2"; shift 2
  local stem stderr result
  stem="$(printf '%s' "$name" | sed 's/\.json$//')"; stderr="$run_root/$stem.stderr"
  set +e
  env -i HOME="$isolated_home" XDG_CONFIG_HOME="$isolated_xdg" TMPDIR="$isolated_tmp" CARGO_HOME="$isolated_cargo" RUSTUP_HOME="$rustup_home" PATH="$PATH" LANG=C LC_ALL=C "$binary" "$@" > "$output/$name" 2> "$stderr"
  result=$?; set -e
  [[ "$result" -ne 0 ]] || die "$name unexpectedly succeeded"
  cp "$stderr" "$output/$stem.stderr" 2>/dev/null || true
  pass "$name" expected-fail
}

download "$from_tag" from "$from_root"
download "$to_tag" to "$to_root"
source_before_status="$(git -C "$source_repo" status --porcelain=v1)"
source_ai_digest=''; [[ -f "$source_repo/.ai/project.json" ]] && source_ai_digest="sha256:$(sha256_file "$source_repo/.ai/project.json")"
pass public-release-pins

env -i HOME="$isolated_home" XDG_CONFIG_HOME="$isolated_xdg" TMPDIR="$isolated_tmp" CARGO_HOME="$isolated_cargo" RUSTUP_HOME="$rustup_home" PATH="$PATH" LANG=C LC_ALL=C cargo new --lib --vcs none "$adopter" >/dev/null
printf 'target/\n' > "$adopter/.gitignore"; : > "$adopter/AGENTS.md"
git -C "$adopter" init -q; git -C "$adopter" config user.name 'AI Cockpit N-1 Acceptance'; git -C "$adopter" config user.email 'ai-cockpit-n-minus-one@example.invalid'
git -C "$adopter" add .; git -C "$adopter" commit -qm 'initial adopter'
# Cargo/rustup may populate the intentionally isolated HOME while scaffolding
# the fixture.  Capture the baseline after that setup and before any Runtime
# operation, so the proof detects Runtime escape into global directories.
manifest_tree "$isolated_home" "$run_root/home-before.manifest"
manifest_tree "$isolated_xdg" "$run_root/xdg-before.manifest"
manifest_tree "$isolated_tmp" "$run_root/tmp-before.manifest"
manifest_tree "$isolated_cargo" "$run_root/cargo-before.manifest"
run "$from_bin" from-attach.json attach --repo "$adopter"
run "$from_bin" from-profile.json profile confirm --repo "$adopter" --program cargo --args test,--workspace
run "$from_bin" from-agent-install.json agent install --repo "$adopter" --provider auto
run "$from_bin" from-agent-doctor.json agent doctor --repo "$adopter" --json
jq -e '.state=="VERIFIED" and (.problems|length==0)' "$output/from-agent-doctor.json" >/dev/null || die 'old Agent doctor did not verify'
# Older public Runtimes may create protocol directories lazily.  Preparing the
# repository-owned evidence directory is fixture scaffolding, not a Runtime
# fallback or a source checkout; it lets the old binary record its first
# verification receipt deterministically.
mkdir -p "$adopter/.ai/evidence"
pass old-protocol-directories
old_schema=1
if grep -q '^repository_schema_version[[:space:]]*=' "$adopter/.ai/cockpit.toml"; then
  old_schema="$(awk -F '=' '/^repository_schema_version[[:space:]]*=/{gsub(/[[:space:]]/,"",$2); print $2; exit}' "$adopter/.ai/cockpit.toml")"
fi
[[ "$old_schema" =~ ^[0-9]+$ ]] || die 'old Runtime wrote an invalid repository schema'
git -C "$adopter" add .
git -C "$adopter" commit -qm 'attach adopter governance state'
pass old-schema-assertion "old Runtime repository schema $old_schema"
work_item=n-minus-one-lifecycle
run "$from_bin" old-start.json start --repo "$adopter" --id "$work_item" --intent 'Validate upgrade without losing governed history.' --goal 'Prove N-1 compatibility and explicit migration.' --scope '**' --out-of-scope target --risk normal --authority authorized --acceptance 'cargo test passes' --required-evidence verification
printf '\n// N-1 acceptance mutation\n' >> "$adopter/src/lib.rs"; git -C "$adopter" add src/lib.rs; git -C "$adopter" commit -qm 'adopter change before upgrade'
run "$from_bin" old-preflight.json preflight --repo "$adopter" --contract "$adopter/.ai/work-items/active/$work_item.contract.json"
run "$from_bin" old-checkpoint.json checkpoint --repo "$adopter" --id "$work_item"
run "$from_bin" old-verify.json verify --repo "$adopter" --work-item "$work_item" --command true --workers 1
if jq -e --arg version "$from_version" --arg digest "$from_digest" '.runtimeVersion==$version and .runtimeDigest==$digest' "$output/old-verify.json" >/dev/null; then
  pass old-verify-runtime-identity
else
  case "$from_version" in
    0.1.*|0.2.0)
      # These historical public binaries predate Runtime identity fields in
      # verify output. Their immutable archive identity remains bound by the
      # from-runtime.json record; current Releases are never allowed this
      # exception.
      record old-verify-runtime-identity not_applicable 'historical Runtime predates verify identity fields'
      ;;
    *)
      die 'old verification output lacks old Runtime identity'
      ;;
  esac
fi
run "$from_bin" old-finish.json finish --repo "$adopter" --id "$work_item"
run "$from_bin" old-archive.json archive --repo "$adopter" --id "$work_item"
run "$from_bin" old-close.json close --repo "$adopter" --id "$work_item" --human-decision approved
pass old-lifecycle-closed
find "$adopter/.ai/evidence" -type f -print | LC_ALL=C sort | while IFS= read -r path; do printf '%s  %s\n' "$(sha256_file "$path")" "$(printf '%s' "$path" | sed "s#^$adopter/##")"; done > "$run_root/evidence-before"
cp "$run_root/evidence-before" "$output/evidence-before.sha256"; pass historical-evidence-captured

run "$to_bin" new-compatibility.json compatibility --repo "$adopter"
new_required_schema="$(jq -er '.requiredRepositorySchemaVersion' "$output/new-compatibility.json")"
case "$old_schema:$new_required_schema" in
  1:2)
    jq -e '.state=="MIGRATION_REQUIRED" and .repositorySchemaVersion==1' "$output/new-compatibility.json" >/dev/null || die 'new Runtime did not require migration'
    run "$to_bin" migration-plan.json migrate plan --repo "$adopter"
    jq -e '.state=="MIGRATION_REQUIRED" and .humanApprovalRequired==true and .currentSchema==1 and (.steps|length)>=1' "$output/migration-plan.json" >/dev/null || die 'migration plan was not approval-gated or adjacent'
    expected_fail "$to_bin" migration-apply-without-approval.json migrate apply --repo "$adopter"
    grep -Eiq 'approved|approval|human' "$output/migration-apply-without-approval.stderr" || die 'unapproved migration did not explain approval'
    run "$to_bin" migration-apply-approved.json migrate apply --repo "$adopter" --approved
    jq -e --arg version "$to_version" --arg digest "$to_digest" '.fromSchema==1 and .toSchema==2 and .result=="completed" and .runtimeVersion==$version and .runtimeDigest==$digest and .step.fromSchema==1 and .step.toSchema==2 and .chainLength>=1' "$output/migration-apply-approved.json" >/dev/null || die 'migration receipt lacks new Runtime identity or chain metadata'
    jq -n --arg state migrated --argjson from "$old_schema" --argjson to "$new_required_schema" '{schemaVersion:1,state:$state,fromSchema:$from,toSchema:$to}' > "$output/migration-state.json"
    ;;
  2:2)
    jq -e '.state=="COMPATIBLE" and .repositorySchemaVersion==2' "$output/new-compatibility.json" >/dev/null || die 'same-schema N-1 Runtime was not compatible'
    jq -n --arg state not_required --argjson from "$old_schema" --argjson to "$new_required_schema" '{schemaVersion:1,state:$state,fromSchema:$from,toSchema:$to}' > "$output/migration-state.json"
    ;;
  *)
    die "unsupported N-1 schema transition: $old_schema -> $new_required_schema"
    ;;
esac
find "$adopter/.ai/evidence" -type f -print | LC_ALL=C sort | while IFS= read -r path; do printf '%s  %s\n' "$(sha256_file "$path")" "$(printf '%s' "$path" | sed "s#^$adopter/##")"; done > "$run_root/evidence-after"
cmp -s "$run_root/evidence-before" "$run_root/evidence-after" || die 'historical evidence changed during migration'
jq -n --arg before "sha256:$(sha256_file "$run_root/evidence-before")" --arg after "sha256:$(sha256_file "$run_root/evidence-after")" '{schemaVersion:1,oldEvidenceDigest:$before,newEvidenceDigest:$after,result:"byte-identical"}' > "$output/history-digest.json"
pass historical-evidence-preserved

run "$to_bin" new-compatibility-after.json compatibility --repo "$adopter"
jq -e '.state=="COMPATIBLE" and .repositorySchemaVersion==2' "$output/new-compatibility-after.json" >/dev/null || die 'migrated adopter is not compatible'
new_work_item=n-minus-one-post-migration
run "$to_bin" new-start.json start --repo "$adopter" --id "$new_work_item" --intent 'Validate operation after an approved repository migration.' --goal 'Prove the new Runtime can govern a fresh Work Item after N-1 migration.' --scope '**' --out-of-scope target --risk normal --authority authorized --acceptance 'cargo test passes' --required-evidence verification
run "$to_bin" new-preflight.json preflight --repo "$adopter" --contract "$adopter/.ai/work-items/active/$new_work_item.contract.json"
run "$to_bin" new-checkpoint.json checkpoint --repo "$adopter" --id "$new_work_item"
run "$to_bin" new-agent-doctor.json agent doctor --repo "$adopter" --json
jq -e '.state=="VERIFIED" and (.problems|length==0)' "$output/new-agent-doctor.json" >/dev/null || die 'new Agent doctor did not verify'
run "$to_bin" new-verify.json verify --repo "$adopter" --work-item "$new_work_item" --command true --workers 1
jq -e --arg version "$to_version" --arg digest "$to_digest" '.runtimeVersion==$version and .runtimeDigest==$digest and .passed==true' "$output/new-verify.json" >/dev/null || die 'new verification output lacks new Runtime identity'
jq -e '.passed==true' "$output/new-verify.json" >/dev/null || die 'new Runtime did not continue operation'
new_evidence="$adopter/.ai/evidence/$new_work_item.verification.json"
[[ -f "$new_evidence" ]] || die 'new Runtime did not record verification evidence'
jq -e --arg version "$to_version" --arg digest "$to_digest" '.runtimeVersion==$version and .runtimeDigest==$digest and .passed==true' "$new_evidence" >/dev/null || die 'new verification evidence lacks Runtime identity'
run "$to_bin" new-finish.json finish --repo "$adopter" --id "$new_work_item"
run "$to_bin" new-archive.json archive --repo "$adopter" --id "$new_work_item"
run "$to_bin" new-close.json close --repo "$adopter" --id "$new_work_item" --human-decision approved
pass continued-operation

source_after_status="$(git -C "$source_repo" status --porcelain=v1)"
manifest_tree "$isolated_home" "$run_root/home-after.manifest"
manifest_tree "$isolated_xdg" "$run_root/xdg-after.manifest"
manifest_tree "$isolated_tmp" "$run_root/tmp-after.manifest"
manifest_tree "$isolated_cargo" "$run_root/cargo-after.manifest"
home_unchanged=true; xdg_unchanged=true
cmp -s "$run_root/home-before.manifest" "$run_root/home-after.manifest" || home_unchanged=false
cmp -s "$run_root/xdg-before.manifest" "$run_root/xdg-after.manifest" || xdg_unchanged=false
[[ "$source_before_status" == "$source_after_status" ]] || die 'acceptance modified source checkout'
[[ "$home_unchanged" == true && "$xdg_unchanged" == true ]] || die 'acceptance escaped isolated HOME/XDG'
repository_id="$(jq -er '.repositoryId' "$output/from-agent-doctor.json")"
mkdir -p "$output/isolation-manifests"
for name in home-before home-after xdg-before xdg-after tmp-before tmp-after cargo-before cargo-after; do
  cp "$run_root/$name.manifest" "$output/isolation-manifests/$name.manifest"
done
jq -n \
  --arg sourceRepository "$source_repo" \
  --arg sourceAiDigest "$source_ai_digest" \
  --arg sourceBeforeStatus "$source_before_status" \
  --arg sourceAfterStatus "$source_after_status" \
  --arg adopterRepository "$adopter" \
  --arg repositoryId "$repository_id" \
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
  '{schemaVersion:2,sourceRepository:$sourceRepository,sourceAiDigest:(if $sourceAiDigest=="" then null else $sourceAiDigest end),sourceBeforeStatus:$sourceBeforeStatus,sourceAfterStatus:$sourceAfterStatus,sourceUnchanged:($sourceBeforeStatus==$sourceAfterStatus),adopterRepository:$adopterRepository,repositoryId:$repositoryId,repositoryIsolation:($adopterRepository!=$sourceRepository),roots:{HOME:{classification:"global-config",allowedWrites:false,beforeDigest:$homeBeforeDigest,afterDigest:$homeAfterDigest,unchanged:$homeUnchanged},XDG_CONFIG_HOME:{classification:"global-config",allowedWrites:false,beforeDigest:$xdgBeforeDigest,afterDigest:$xdgAfterDigest,unchanged:$xdgUnchanged},TMPDIR:{classification:"runtime-temporary",allowedWrites:true,beforeDigest:$tmpBeforeDigest,afterDigest:$tmpAfterDigest},CARGO_HOME:{classification:"dependency-cache",allowedWrites:true,beforeDigest:$cargoBeforeDigest,afterDigest:$cargoAfterDigest}},releasePublished:true}' > "$output/isolation.json"
jq -e '.schemaVersion==2 and .sourceUnchanged and .roots.HOME.unchanged and .roots.XDG_CONFIG_HOME.unchanged and .roots.TMPDIR.allowedWrites and .roots.CARGO_HOME.allowedWrites and .repositoryIsolation' "$output/isolation.json" >/dev/null || die 'isolation proof failed'
pass isolation
