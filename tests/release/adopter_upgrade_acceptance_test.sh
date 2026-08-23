#!/usr/bin/env bash
set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/adopter_upgrade_acceptance.sh"
bash -n "$script"
grep -q -- '--from-tag' "$script"
grep -q -- '--to-tag' "$script"
grep -q -- '--to-candidate-dir' "$script"
grep -q -- 'releasePublished' "$script"
grep -q -- 'stagedCandidate' "$script"
grep -q -- 'platform' "$script"
grep -q -- 'runtimeVersion' "$script"
grep -q -- 'runtimeDigest' "$script"
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
grep -q -- 'rm -rf --' "$script"
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
if grep -Eq -- 'close --repo [^[:space:]]+ --id [^[:space:]]+ --human-decision approved$' "$script"; then
  echo 'adopter upgrade acceptance must not close with an unstructured decision' >&2
  exit 1
fi
grep -q -- 'schemaVersion:2' "$script"
grep -Fq -- 'allowedPrefixes' "$script"
grep -Fq -- '<CARGO_HOME>/**' "$script"
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
old_verify_line=$(grep -n -- 'old-verify.json verify' "$script" | head -1 | cut -d: -f1)
old_finalize_line=$(grep -n -- 'old-finalize.json work-item finalize' "$script" | head -1 | cut -d: -f1)
old_finalize_verify_line=$(grep -n -- 'old-finalize-verify.json work-item finalize-verify' "$script" | head -1 | cut -d: -f1)
old_close_line=$(grep -n -- 'old-close.json close' "$script" | head -1 | cut -d: -f1)
[[ -n "$old_verify_line" && -n "$old_finalize_line" && -n "$old_finalize_verify_line" && -n "$old_close_line" && "$old_verify_line" -lt "$old_finalize_line" && "$old_finalize_line" -lt "$old_finalize_verify_line" && "$old_finalize_verify_line" -lt "$old_close_line" ]] || {
  printf 'adopter upgrade acceptance must run old verify, finalize, finalize-verify, then close\n' >&2
  exit 1
}
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
