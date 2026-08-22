#!/usr/bin/env bash
set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/adopter_acceptance.sh"
repo="$(git rev-parse --show-toplevel)"
manifest_test="$(cd "$(dirname "$0")" && pwd)/isolation_manifest_test.sh"

bash -n "$script"
bash -n "$manifest_test"
bash "$manifest_test"
grep -q -- '--repository OWNER/REPOSITORY' "$script"
grep -q -- 'releasePublished' "$script"
grep -q -- 'first-adopter-smoke' "$script"
grep -q -- 'nodesReused' "$script"
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
grep -q -- 'isolation_manifest.sh' "$script"
grep -q -- 'manifest_tree' "$script"
grep -q -- 'schemaVersion:2' "$script"
grep -Fq -- 'allowedPrefixes' "$script"
grep -Fq -- '<CARGO_HOME>/**' "$script"
preflight_line=$(grep -n -- 'lifecycle-preflight.json preflight' "$script" | head -1 | cut -d: -f1)
checkpoint_line=$(grep -n -- 'lifecycle-checkpoint.json checkpoint' "$script" | head -1 | cut -d: -f1)
[[ -n "$preflight_line" && -n "$checkpoint_line" && "$preflight_line" -lt "$checkpoint_line" ]] || {
  printf 'adopter acceptance must record preflight before checkpoint\n' >&2
  exit 1
}
if grep -Eq 'cargo[[:space:]]+(build|run)' "$script"; then
  printf 'acceptance harness must not obtain Runtime through cargo build/run\n' >&2
  exit 1
fi

test_parent="${TMPDIR:-/tmp}"
regression_root="$(mktemp -d "$test_parent/ai-cockpit-adopter-regression.XXXXXX")"
cleanup_regression_root() { find "$regression_root" -depth -mindepth 0 -delete; }
trap cleanup_regression_root EXIT
invalid_output="$regression_root/invalid-output"
mkdir -p "$invalid_output"
if "$script" --repository xinglun/ai-cockpit --tag v0.1.1 --target unsupported --output "$invalid_output" --source-repo "$repo" >/dev/null 2>&1; then
  printf 'unsupported targets must fail closed\n' >&2
  exit 1
fi

fake_bin="$regression_root/fake-bin"
mkdir -p "$fake_bin"
printf '#!/bin/sh\nexit 97\n' > "$fake_bin/curl"
chmod +x "$fake_bin/curl"
failure_tmp="$regression_root/failure-tmp"
failure_output="$regression_root/failure-output"
mkdir -p "$failure_tmp" "$failure_output"
set +e
PATH="$fake_bin:$PATH" TMPDIR="$failure_tmp" "$script" \
  --repository xinglun/ai-cockpit --tag v0.2.6 --target x86_64-unknown-linux-gnu \
  --output "$failure_output" --source-repo "$repo" >/dev/null 2>&1
failure_exit=$?
set -e
[[ "$failure_exit" -eq 1 ]] || { printf 'failure path must preserve the original exit code\n' >&2; exit 1; }
[[ -z "$(find "$failure_tmp" -mindepth 1 -maxdepth 1 -type d -name 'ai-cockpit-adopter-acceptance.*' -print -quit)" ]] || {
  printf 'failure path left a run_root behind\n' >&2
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
  --repository xinglun/ai-cockpit --tag v0.2.6 --target x86_64-unknown-linux-gnu \
  --output "$blocked_output" --source-repo "$repo" >/dev/null 2>&1
blocked_exit=$?
set -e
[[ "$blocked_exit" -eq 1 ]] || { printf 'cleanup-failure path must preserve the acceptance exit code\n' >&2; exit 1; }
jq -e '.adopterAcceptance == "failed" and .releasePublished == false and .cleanupState == "failed" and (.cleanupError | length) > 0' "$blocked_output/acceptance.json" >/dev/null
jq -e '.state == "failed" and .removed == false and .validated == true' "$blocked_output/cleanup.json" >/dev/null
(cd "$blocked_output" && shasum -a 256 -c SHA256SUMS >/dev/null)
[[ -n "$(find "$blocked_tmp" -mindepth 1 -maxdepth 1 -type d -name 'ai-cockpit-adopter-acceptance.*' -print -quit)" ]] || {
  printf 'cleanup-failure regression did not leave the forced failure root for inspection\n' >&2
  exit 1
}
find "$blocked_tmp" -depth -mindepth 0 -delete

run_public=''
if run_public="$(printenv AI_COCKPIT_RUN_PUBLIC_ACCEPTANCE)"; then :; fi
if [[ "$run_public" == 1 ]]; then
  target=''
  if target="$(printenv AI_COCKPIT_ACCEPTANCE_TARGET)"; then :; fi
  if [[ -z "$target" ]]; then
    case "$(uname -s):$(uname -m)" in
      Darwin:arm64) target=aarch64-apple-darwin ;;
      Darwin:x86_64) target=x86_64-apple-darwin ;;
      Linux:x86_64) target=x86_64-unknown-linux-gnu ;;
      *) printf 'set AI_COCKPIT_ACCEPTANCE_TARGET for this host\n' >&2; exit 1 ;;
    esac
  fi
  output="$regression_root/public-output"
  mkdir -p "$output"
  public_tmp="$regression_root/public-tmp"
  mkdir -p "$public_tmp"
  TMPDIR="$public_tmp" "$script" --repository xinglun/ai-cockpit --tag v0.1.1 --target "$target" --output "$output" --source-repo "$repo"
  jq -e '.adopterAcceptance == "passed" and .releasePublished == true and ([.steps[] | select(.state != "passed")] | length == 0)' "$output/acceptance.json" >/dev/null
  jq -e --arg version v0.1.1 '.version == $version and .releasePublished == true and (.binaryDigest | startswith("sha256:"))' "$output/runtime.json" >/dev/null
  jq -e '.state == "not_ready" and .intent == "" and (.scope | length == 0) and .authority == "unknown"' "$output/work-items/first-adopter-smoke.contract.json" >/dev/null
  (cd "$output" && shasum -a 256 -c SHA256SUMS >/dev/null)
  [[ -z "$(find "$public_tmp" -mindepth 1 -maxdepth 1 -type d -name 'ai-cockpit-adopter-acceptance.*' -print -quit)" ]] || {
    printf 'success path left a run_root behind\n' >&2
    exit 1
  }
fi

printf 'adopter acceptance harness checks passed\n'
