#!/usr/bin/env bash
set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/adopter_upgrade_acceptance.sh"
workflow="$(cd "$(dirname "$0")/../.." && pwd)/.github/workflows/release.yml"
bash -n "$script"
assert_release_identity_checkouts() {
  local job="$1"
  local block
  block="$(awk -v job="$job" '
    $0 == "  " job ":" { found = 1; next }
    found && $0 ~ /^  [[:alnum:]_]+:/ { exit }
    found { print }
  ' "$workflow")"
  [[ -n "$block" ]] || { printf 'release workflow job is missing: %s\n' "$job" >&2; exit 1; }
  grep -Fq 'ref: ${{ github.sha }}' <<<"$block" || {
    printf '%s must execute the harness from the workflow commit\n' "$job" >&2
    exit 1
  }
  grep -Fq 'path: release-source' <<<"$block" || {
    printf '%s must checkout the immutable release source separately\n' "$job" >&2
    exit 1
  }
  grep -Fq 'ref: ${{ (github.event_name == '\''workflow_dispatch'\'' && github.event.inputs.to_tag) || github.ref }}' <<<"$block" || {
    printf '%s must bind its source checkout to the requested Release tag\n' "$job" >&2
    exit 1
  }
  grep -Fq -- '--source-repo "$GITHUB_WORKSPACE/release-source"' <<<"$block" || {
    printf '%s must pass the separate immutable source checkout\n' "$job" >&2
    exit 1
  }
}
for release_job in staged_adopter_acceptance staged_adopter_upgrade_acceptance adopter_acceptance adopter_upgrade_acceptance; do
  assert_release_identity_checkouts "$release_job"
done
grep -F -A8 -- 'name: Run staged candidate adopter acceptance' "$workflow" | grep -q -- 'GH_TOKEN:'
grep -F -A8 -- 'name: Run public-to-staged N-1 acceptance' "$workflow" | grep -q -- 'GH_TOKEN:'
grep -F -A8 -- 'name: Run public Release adopter acceptance' "$workflow" | grep -q -- 'GH_TOKEN:'
grep -F -A8 -- 'name: Run public-artifact N-1 upgrade acceptance' "$workflow" | grep -q -- 'GH_TOKEN:'
grep -q -- '--from-tag' "$script"
grep -q -- '--to-tag' "$script"
grep -q -- '--to-candidate-dir' "$script"
grep -q -- 'profile confirm --repo "$adopter" --program cargo --args test,--locked,--package,adopter' "$script"
grep -q -- '--publish-handoff FILE' "$script"
grep -q -- 'releasePublished' "$script"
grep -q -- 'stagedCandidate' "$script"
grep -q -- 'platform' "$script"
grep -q -- 'runtimeVersion' "$script"
grep -q -- 'runtimeDigest' "$script"
grep -q -- 'github_api_get' "$script"
grep -q -- 'Authorization: Bearer' "$script"
grep -q -- 'GITHUB_TOKEN' "$script"
grep -q -- 'github_api_get "https://api.github.com/repos/$repository/releases/tags/$tag" "$api"' "$script"
grep -q -- '--resume' "$script"
grep -q -- 'acceptance-plan' "$script"
grep -q -- 'acceptance-record' "$script"
grep -q -- 'phase_action_should_run' "$script"
grep -q -- 'phase_action_strategy' "$script"
grep -q -- 'retry_cleanup_only' "$script"
grep -q -- 'resume phase receipt is missing' "$script"
grep -q -- 'record_phase_success candidate_acceptance' "$script"
grep -q -- 'record_phase_success public_acceptance' "$script"
grep -q -- 'target release manifest commit does not match source checkout HEAD' "$script"
grep -q -- '--arg commit "$release_source_commit"' "$script"
grep -q -- '--arg sourceCommit "$release_source_commit"' "$script"
grep -q -- 'write_unbound_failure_receipt' "$script"
grep -q -- 'acceptance_phase_failed_before_identity' "$script"
grep -q -- 'record_close_after_cleanup' "$script"
grep -q -- 'close_ready' "$script"
grep -q -- 'validate_persisted_acceptance' "$script"
grep -Fq -- "--scope '.ai/**'" "$script"
grep -Fq -- "--scope 'Cargo.lock'" "$script"
grep -q -- 'AI_COCKPIT_ACCEPTANCE_FAIL_AFTER_PHASE' "$script"
grep -q -- 'COCKPIT_RELEASE_BIN' "$script"
if grep -q -- 'auth_args' "$script"; then
  printf 'adopter upgrade acceptance must not expand an empty auth array under set -u\n' >&2
  exit 1
fi
if grep -n -- 'curl --fail.*api.github.com' "$script" >/dev/null; then
  printf 'adopter upgrade acceptance must route GitHub API requests through the authenticated helper\n' >&2
  exit 1
fi
grep -q -- 'historical Runtime predates verify identity fields' "$script"
grep -q -- 'MIGRATION_REQUIRED' "$script"
grep -q -- 'migrate plan' "$script"
grep -q -- 'migrate apply' "$script"
grep -q -- '2:2' "$script"
grep -q -- 'not_required' "$script"
grep -q -- 'chainLength' "$script"
grep -q -- 'oldEvidenceDigest' "$script"
grep -q -- 'byte-identical' "$script"
grep -q -- 'SHA256SUMS' "$script"
grep -q -- 'cleanup_run_root' "$script"
grep -q -- 'cleanupState' "$script"
grep -q -- 'cleanup.json' "$script"
grep -q -- 'deviceInode' "$script"
grep -q -- 'recover_prior_cleanup' "$script"
grep -q -- 'remove_exact_tree' "$script"
grep -q -- 'git -C "$adopter" config gc.auto 0' "$script"
grep -q -- 'git -C "$adopter" config maintenance.auto false' "$script"
grep -q -- 'rustup show active-toolchain' "$script"
grep -q -- 'RUSTUP_TOOLCHAIN' "$script"
grep -q -- 'rustToolchain' "$script"
grep -q -- 'adopterAcceptance = "failed"' "$script"
grep -q -- 'exit "$exit_code"' "$script"
grep -q -- 'local exit_code=\$?' "$script"
grep -q -- 'isolation_manifest.sh' "$script"
grep -q -- 'manifest_tree' "$script"
grep -q -- '--actor human:release-acceptance' "$script"
grep -q -- '--authority-source release-adopter-upgrade-acceptance' "$script"
grep -q -- '--evidence-ref' "$script"
grep -q -- '--policy-ref' "$script"
grep -q -- '--decided-at' "$script"
grep -q -- '--resume-condition' "$script"
grep -q -- 'validate_close_decision' "$script"
grep -q -- 'work-item finalize-plan' "$script"
grep -q -- 'work-item finalize-verify' "$script"
grep -q -- 'new-finalize.json' "$script"
grep -q -- 'new-finalize-verify.json' "$script"
grep -q -- 'old-finalize.json' "$script"
grep -q -- 'old-finalize-verify.json' "$script"
grep -q -- 'close.binding.json' "$script"
grep -q -- 'closeDecisionValidated' "$script"
grep -q -- 'adopter repository identity is missing or malformed' "$script"
grep -q -- 'decisionState == "confirmed"' "$script"
grep -q -- 'structuredDecision.evidenceRefs' "$script"
grep -q -- 'structuredDecision.policyRefs' "$script"
grep -q -- 'result:{disposition:"deleted"' "$script"
grep -q -- 'configure_git_identity()' "$script"
grep -q -- 'config --local user.name' "$script"
grep -q -- 'config --local user.email' "$script"
initial_clone_line=$(grep -n -- 'git clone -q "\$adopter" "\$old_control_root"' "$script" | head -1 | cut -d: -f1)
initial_identity_line=$(grep -n -- 'configure_git_identity "\$old_control_root"' "$script" | head -1 | cut -d: -f1)
new_clone_line=$(grep -n -- 'git clone -q "\$adopter" "\$new_control_root"' "$script" | head -1 | cut -d: -f1)
new_identity_line=$(grep -n -- 'configure_git_identity "\$new_control_root"' "$script" | head -1 | cut -d: -f1)
[[ -n "$initial_clone_line" && -n "$initial_identity_line" && "$initial_clone_line" -lt "$initial_identity_line" && -n "$new_clone_line" && -n "$new_identity_line" && "$new_clone_line" -lt "$new_identity_line" ]] || {
  printf 'every cloned acceptance repository must receive a local Git identity before use\n' >&2
  exit 1
}
grep -q -- 'remove_exact_tree "$old_worktree"' "$script"
grep -q -- 'remove_exact_tree "$new_worktree"' "$script"
grep -q -- 'worktree prune' "$script"
grep -q -- 'branch -D' "$script"
if grep -q -- 'result:{disposition:"retained"' "$script"; then
  echo 'staged upgrade lifecycle must not close with retained resources' >&2
  exit 1
fi
if grep -Eq -- 'close --repo [^[:space:]]+ --id [^[:space:]]+ --human-decision approved$' "$script"; then
  echo 'adopter upgrade acceptance must not close with an unstructured decision' >&2
  exit 1
fi
grep -q -- 'schemaVersion:2' "$script"
grep -Fq -- 'allowedPrefixes' "$script"
grep -Fq -- '<CARGO_HOME>/**' "$script"
grep -Fq -- 'CARGO_TARGET_DIR="$isolated_cargo/target"' "$script"
if grep -Fq -- "--scope '**'" "$script"; then
  printf 'generic N-1 lifecycle fixtures must not use a repository-wide scope\n' >&2
  exit 1
fi
grep -Fq -- "--scope 'src/**'" "$script" || {
  printf 'generic N-1 lifecycle fixtures must declare their source scope\n' >&2
  exit 1
}
preflight_line=$(grep -n -- 'old-preflight.json preflight' "$script" | head -1 | cut -d: -f1)
checkpoint_line=$(grep -n -- 'old-checkpoint.json checkpoint' "$script" | head -1 | cut -d: -f1)
[[ -n "$preflight_line" && -n "$checkpoint_line" && "$preflight_line" -lt "$checkpoint_line" ]] || {
  printf 'adopter upgrade acceptance must record preflight before checkpoint\n' >&2
  exit 1
}
new_preflight_line=$(grep -n -- 'new-preflight.json preflight' "$script" | head -1 | cut -d: -f1)
new_checkpoint_line=$(grep -n -- 'new-checkpoint.json checkpoint' "$script" | head -1 | cut -d: -f1)
new_verify_line=$(grep -n -- 'new-verify.json verify' "$script" | head -1 | cut -d: -f1)
new_finalize_line=$(grep -n -- 'new-finalize.json work-item finalize' "$script" | head -1 | cut -d: -f1)
new_finalize_verify_line=$(grep -n -- 'new-finalize-verify.json work-item finalize-verify' "$script" | head -1 | cut -d: -f1)
new_close_line=$(grep -n -- 'new-close.json close' "$script" | head -1 | cut -d: -f1)
[[ -n "$new_preflight_line" && -n "$new_checkpoint_line" && -n "$new_verify_line" && -n "$new_finalize_line" && -n "$new_finalize_verify_line" && -n "$new_close_line" && "$new_preflight_line" -lt "$new_checkpoint_line" && "$new_checkpoint_line" -lt "$new_verify_line" && "$new_verify_line" -lt "$new_finalize_line" && "$new_finalize_line" -lt "$new_finalize_verify_line" && "$new_finalize_verify_line" -lt "$new_close_line" ]] || {
  printf 'adopter upgrade acceptance must run new preflight, checkpoint, verify, finalize, finalize-verify, then close\n' >&2
  exit 1
}
old_base_revision_line=$(grep -n -- 'old_base_revision' "$script" | head -1 | cut -d: -f1)
new_base_revision_line=$(grep -n -- 'new_base_revision' "$script" | head -1 | cut -d: -f1)
[[ -n "$old_base_revision_line" && -n "$new_base_revision_line" ]] || {
  printf 'adopter upgrade acceptance must preserve both archived Contract base revisions\n' >&2
  exit 1
}
grep -q -- '--arg oldBaseRevision "$old_base_revision"' "$script" || {
  printf 'adopter upgrade old finalization must bind its Contract base revision\n' >&2
  exit 1
}
grep -q -- 'baseRevision:$oldBaseRevision' "$script" || {
  printf 'adopter upgrade old finalization must emit its preserved base revision\n' >&2
  exit 1
}
grep -q -- '--arg newBaseRevision "$new_base_revision"' "$script" || {
  printf 'adopter upgrade new finalization must bind its Contract base revision\n' >&2
  exit 1
}
grep -q -- 'baseRevision:$newBaseRevision' "$script" || {
  printf 'adopter upgrade new finalization must emit its preserved base revision\n' >&2
  exit 1
}
if grep -q -- 'baseRevision:$old_head\|baseRevision:$new_head' "$script"; then
  printf 'adopter upgrade acceptance must not bind post-mutation HEAD as baseRevision\n' >&2
  exit 1
fi
old_verify_line=$(grep -n -- 'old-verify.json verify' "$script" | head -1 | cut -d: -f1)
old_post_verify_status_line=$(grep -n -- 'old-status-after-verify.json work-item status --repo "$adopter" --id "$work_item" --json' "$script" | head -1 | cut -d: -f1)
old_finish_line=$(grep -n -- 'old-finish.json finish --repo "$adopter" --id "$work_item"' "$script" | head -1 | cut -d: -f1)
old_archive_line=$(grep -n -- 'old-archive.json archive --repo "$adopter" --id "$work_item"' "$script" | head -1 | cut -d: -f1)
old_finalize_line=$(grep -n -- 'old-finalize.json work-item finalize' "$script" | head -1 | cut -d: -f1)
old_finalize_verify_line=$(grep -n -- 'old-finalize-verify.json work-item finalize-verify' "$script" | head -1 | cut -d: -f1)
old_close_line=$(grep -n -- 'old-close.json close' "$script" | head -1 | cut -d: -f1)
[[ -n "$old_verify_line" && -n "$old_post_verify_status_line" && -n "$old_finish_line" && -n "$old_archive_line" && -n "$old_finalize_line" && -n "$old_finalize_verify_line" && -n "$old_close_line" && "$old_verify_line" -lt "$old_post_verify_status_line" && "$old_post_verify_status_line" -lt "$old_finish_line" && "$old_finish_line" -lt "$old_archive_line" && "$old_archive_line" -lt "$old_finalize_line" && "$old_finalize_line" -lt "$old_finalize_verify_line" && "$old_finalize_verify_line" -lt "$old_close_line" ]] || {
  printf 'adopter upgrade acceptance must verify, read and validate old Runtime status, finish, archive, then finalize, finalize-verify, and close\n' >&2
  exit 1
}
if grep -Fq -- 'old-preflight-after-verify.json preflight' "$script"; then
  printf 'adopter upgrade acceptance must not rerun old preflight after verification\n' >&2
  exit 1
fi
if grep -Eq 'cargo (build|run)|target/debug/ai-cockpit|workspace binary' "$script"; then
  echo 'upgrade acceptance must not fall back to source builds or workspace binaries' >&2
  exit 1
fi
if grep -Fq -- '--command' "$script"; then
  echo 'upgrade acceptance must use only the canonical Runtime verification command' >&2
  exit 1
fi
test_parent="${TMPDIR:-/tmp}"
regression_root="$(mktemp -d "$test_parent/ai-cockpit-n-minus-one-regression.XXXXXX")"
cleanup_regression_root() { find "$regression_root" -depth -mindepth 0 -delete; }
trap cleanup_regression_root EXIT

# Exercise the same fail-closed validator used before old-Runtime finish. A
# harmless unknown is retained because finish admission is based on fresh
# verification and the Runtime's current safeActions, not empty unknowns.
status_fixture="$regression_root/old-finish-status.json"
status_work_item='WI-N1-STATUS-TEST'
status_repository_id='sha256:1111111111111111111111111111111111111111111111111111111111111111'
jq -n --arg id "$status_work_item" --arg repo "$status_repository_id" \
  '{workItemId:$id,repositoryId:$repo,verification:"verified",evidenceFreshness:{state:"fresh"},blockers:[],safeActions:["finish"],actionExplanation:{admissionState:"allowed",recommendedAction:"finish",humanDecisionRequired:false},humanDecisionRequired:false,unknowns:["user_visible_benefit_not_declared"]}' \
  > "$status_fixture"
"$script" --validate-old-finish-status "$status_fixture" "$status_work_item" "$status_repository_id" >/dev/null

assert_old_finish_status_rejected() {
  local label="$1" path="$2"
  if "$script" --validate-old-finish-status "$path" "$status_work_item" "$status_repository_id" >/dev/null 2>&1; then
    printf 'old finish status validator accepted unsafe fixture: %s\n' "$label" >&2
    exit 1
  fi
}
assert_old_finish_status_rejected missing-file "$regression_root/missing-status.json"
printf '{broken json\n' > "$regression_root/malformed-status.json"
assert_old_finish_status_rejected malformed-json "$regression_root/malformed-status.json"
for mutation in \
  '.workItemId = "other-work-item"' \
  '.repositoryId = "sha256:2222222222222222222222222222222222222222222222222222222222222222"' \
  '.verification = "not_ready"' \
  '.evidenceFreshness.state = "stale_or_invalid"' \
  '.blockers = ["verification_required"]' \
  '.safeActions = []' \
  '.actionExplanation.admissionState = "blocked"' \
  '.actionExplanation.recommendedAction = "run_preflight"' \
  '.actionExplanation.humanDecisionRequired = true' \
  '.humanDecisionRequired = true'; do
  jq "$mutation" "$status_fixture" > "$regression_root/mutated-status.json"
  assert_old_finish_status_rejected "$mutation" "$regression_root/mutated-status.json"
done

# Regression: a clean CI-like environment must be able to commit both the
# initial repository and a freshly cloned control repository without global
# Git configuration.  This mirrors the exact failure seen in the staged N-1
# release acceptance path.
git_identity_root="$regression_root/git-identity"
git_identity_home="$git_identity_root/home"
git_identity_origin="$git_identity_root/origin"
git_identity_clone="$git_identity_root/clone"
mkdir -p "$git_identity_home"
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git init -q "$git_identity_origin"
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git -C "$git_identity_origin" config --local user.name 'AI Cockpit N-1 Acceptance'
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git -C "$git_identity_origin" config --local user.email 'ai-cockpit-n-minus-one@example.invalid'
touch "$git_identity_origin/initial"
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git -C "$git_identity_origin" add .
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git -C "$git_identity_origin" commit -qm initial
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git clone -q "$git_identity_origin" "$git_identity_clone"
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git -C "$git_identity_clone" config --local user.name 'AI Cockpit N-1 Acceptance'
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git -C "$git_identity_clone" config --local user.email 'ai-cockpit-n-minus-one@example.invalid'
touch "$git_identity_clone/cloned"
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git -C "$git_identity_clone" add .
env -i HOME="$git_identity_home" XDG_CONFIG_HOME="$git_identity_root/xdg" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null PATH="$PATH" git -C "$git_identity_clone" commit -qm cloned
same_output="$regression_root/same-output"
mkdir -p "$same_output"
if "$script" --repository xinglun/ai-cockpit --from-tag v0.1.1 --to-tag v0.1.1 --target aarch64-apple-darwin --output "$same_output" --source-repo "$(git rev-parse --show-toplevel)" >/dev/null 2>&1; then
  echo 'same Release tags must be rejected' >&2
  exit 1
fi

toolchain_tmp="$regression_root/toolchain-tmp"
toolchain_output="$regression_root/toolchain-output"
mkdir -p "$toolchain_tmp" "$toolchain_output"
printf 'not a rustup directory\n' > "$regression_root/invalid-rustup-home"
set +e
RUSTUP_HOME="$regression_root/invalid-rustup-home" TMPDIR="$toolchain_tmp" "$script" \
  --repository xinglun/ai-cockpit --from-tag v0.2.22 --to-tag v0.2.23 \
  --target aarch64-apple-darwin --output "$toolchain_output" \
  --source-repo "$(git rev-parse --show-toplevel)" >/dev/null 2>&1
toolchain_exit=$?
set -e
[[ "$toolchain_exit" -eq 1 ]] || { printf 'invalid RUSTUP_HOME must fail closed\n' >&2; exit 1; }
[[ -z "$(find "$toolchain_tmp" -mindepth 1 -maxdepth 1 -type d -name 'ai-cockpit-n-minus-one.*' -print -quit)" ]] || {
  printf 'upgrade pre-toolchain failure left a run_root behind\n' >&2
  exit 1
}
jq -e '.adopterAcceptance == "failed" and .cleanupState == "passed"' "$toolchain_output/acceptance.json" >/dev/null
jq -e '.state == "passed" and .removed == true and .validated == true' "$toolchain_output/cleanup.json" >/dev/null
(cd "$toolchain_output" && shasum -a 256 -c SHA256SUMS >/dev/null)

fake_bin="$regression_root/fake-bin"
mkdir -p "$fake_bin"
printf '#!/bin/sh\nexit 97\n' > "$fake_bin/curl"
chmod +x "$fake_bin/curl"
failure_tmp="$regression_root/failure-tmp"
failure_output="$regression_root/failure-output"
mkdir -p "$failure_tmp" "$failure_output"
set +e
PATH="$fake_bin:$PATH" TMPDIR="$failure_tmp" "$script" \
  --repository xinglun/ai-cockpit --from-tag v0.2.5 --to-tag v0.2.6 \
  --target x86_64-unknown-linux-gnu --output "$failure_output" \
  --source-repo "$(git rev-parse --show-toplevel)" >/dev/null 2>&1
failure_exit=$?
set -e
[[ "$failure_exit" -eq 1 ]] || { printf 'upgrade failure path must preserve the original exit code\n' >&2; exit 1; }
[[ -z "$(find "$failure_tmp" -mindepth 1 -maxdepth 1 -type d -name 'ai-cockpit-n-minus-one.*' -print -quit)" ]] || {
  printf 'upgrade failure path left a run_root behind\n' >&2
  exit 1
}
jq -e '.adopterAcceptance == "failed" and .releasePublished == false and .cleanupState == "passed" and .cleanupError == null' "$failure_output/acceptance.json" >/dev/null
jq -e '.state == "passed" and .removed == true and .validated == true' "$failure_output/cleanup.json" >/dev/null
(cd "$failure_output" && shasum -a 256 -c SHA256SUMS >/dev/null)

blocked_rm_bin="$regression_root/blocked-rm-bin"
mkdir -p "$blocked_rm_bin"
printf '#!/bin/sh\nexit 71\n' > "$blocked_rm_bin/rm"
chmod +x "$blocked_rm_bin/rm"
blocked_tmp="$regression_root/blocked-tmp"
blocked_output="$regression_root/blocked-output"
mkdir -p "$blocked_tmp" "$blocked_output"
set +e
PATH="$blocked_rm_bin:$fake_bin:$PATH" TMPDIR="$blocked_tmp" "$script" \
  --repository xinglun/ai-cockpit --from-tag v0.2.5 --to-tag v0.2.6 \
  --target x86_64-unknown-linux-gnu --output "$blocked_output" \
  --source-repo "$(git rev-parse --show-toplevel)" >/dev/null 2>&1
blocked_exit=$?
set -e
[[ "$blocked_exit" -eq 1 ]] || { printf 'upgrade cleanup-failure path must preserve the acceptance exit code\n' >&2; exit 1; }
jq -e '.adopterAcceptance == "failed" and .releasePublished == false and .cleanupState == "failed" and (.cleanupError | length) > 0' "$blocked_output/acceptance.json" >/dev/null
jq -e '.state == "failed" and .removed == false and .validated == true' "$blocked_output/cleanup.json" >/dev/null
(cd "$blocked_output" && shasum -a 256 -c SHA256SUMS >/dev/null)
[[ -n "$(find "$blocked_tmp" -mindepth 1 -maxdepth 1 -type d -name 'ai-cockpit-n-minus-one.*' -print -quit)" ]] || {
  printf 'upgrade cleanup-failure regression did not leave the forced failure root for inspection\n' >&2
  exit 1
}
find "$blocked_tmp" -depth -mindepth 0 -delete
echo 'adopter upgrade acceptance static checks passed'
