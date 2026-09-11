#!/usr/bin/env bash
set -euo pipefail

repo="$(git rev-parse --show-toplevel)"
script="$repo/tests/release/adopter_acceptance.sh"
helper="${COCKPIT_RELEASE_BIN:-$repo/target/debug/cockpit-release}"
if [[ ! -x "$helper" ]]; then
  cargo build --locked --package cockpit-release >/dev/null
fi
[[ -x "$helper" ]] || { echo 'cockpit-release helper is unavailable' >&2; exit 1; }

tmp="$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-adopter-resume.XXXXXX")"
cleanup() { find "$tmp" -depth -mindepth 0 -delete; }
trap cleanup EXIT

candidate="$tmp/candidate"
runtime="$tmp/runtime"
output="$tmp/output"
counter="$tmp/command-counter"
mkdir -p "$candidate" "$runtime" "$output"
cat > "$runtime/ai-cockpit" <<'FAKE_RUNTIME'
#!/bin/sh
if [ "$1" = "--version" ]; then
  printf '%s\n' 'ai-cockpit 0.2.90'
  exit 0
fi
if [ "$1" = "isolation-manifest" ]; then
  output=''
  shift
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --output)
        output=$2
        shift 2
        ;;
      *)
        shift
        ;;
    esac
  done
  [ -n "$output" ] || exit 2
  : > "$output"
  exit 0
fi
exit 99
FAKE_RUNTIME
chmod +x "$runtime/ai-cockpit"
tar -czf "$candidate/ai-cockpit-v0.2.90-x86_64-unknown-linux-gnu.tar.gz" -C "$runtime" ai-cockpit
archive="ai-cockpit-v0.2.90-x86_64-unknown-linux-gnu.tar.gz"
archive_sha="$(shasum -a 256 "$candidate/$archive" | awk '{print $1}')"
commit="$(git -C "$repo" rev-parse 'HEAD^{commit}')"
cat > "$candidate/release-manifest.json" <<EOF
{"version":"0.2.90","tag":"v0.2.90","commit":"$commit","artifacts":[{"target":"x86_64-unknown-linux-gnu","archive":{"filename":"$archive","sha256":"$archive_sha"}}]}
EOF
printf '%s  %s\n' "$archive_sha" "$archive" > "$candidate/SHA256SUMS"

set +e
pre_output="$tmp/pre-output"
AI_COCKPIT_ACCEPTANCE_FAIL_BEFORE_PHASE=prepare \
COCKPIT_RELEASE_BIN="$helper" \
TMPDIR="$tmp" \
  "$script" --repository xinglun/ai-cockpit --tag v0.2.90 \
    --target x86_64-unknown-linux-gnu --candidate-dir "$candidate" \
    --source-repo "$repo" --output "$pre_output"
pre_status=$?
set -e
[[ "$pre_status" -ne 0 ]] || { echo 'injected pre-phase failure unexpectedly passed' >&2; exit 1; }
jq -e '[.results[] | select(.phase == "prepare" and .status == "failed" and .failure.kind == "runner")] | length == 1' "$pre_output/phase-receipts.json" >/dev/null

set +e
AI_COCKPIT_ACCEPTANCE_FAIL_AFTER_PHASE=prepare \
AI_COCKPIT_ACCEPTANCE_COMMAND_COUNTER="$counter" \
COCKPIT_RELEASE_BIN="$helper" \
TMPDIR="$tmp" \
  "$script" --repository xinglun/ai-cockpit --tag v0.2.90 \
    --target x86_64-unknown-linux-gnu --candidate-dir "$candidate" \
    --source-repo "$repo" --output "$output"
first_status=$?
set -e
[[ "$first_status" -ne 0 ]] || { echo 'injected first-phase failure unexpectedly passed' >&2; exit 1; }
jq -e '[.results[] | select(.phase == "prepare")] | length == 1' "$output/phase-receipts.json" >/dev/null
jq -e '[.results[] | select(.phase == "source_verification_build" and .status == "failed" and .failure.kind == "runner")] | length == 1' "$output/phase-receipts.json" >/dev/null

set +e
AI_COCKPIT_ACCEPTANCE_FAIL_AFTER_PHASE=source_verification_build \
AI_COCKPIT_ACCEPTANCE_COMMAND_COUNTER="$counter" \
COCKPIT_RELEASE_BIN="$helper" \
TMPDIR="$tmp" \
  "$script" --repository xinglun/ai-cockpit --tag v0.2.90 \
    --target x86_64-unknown-linux-gnu --candidate-dir "$candidate" \
    --source-repo "$repo" --output "$output" --resume
second_status=$?
set -e
[[ "$second_status" -ne 0 ]] || { echo 'injected resume-phase failure unexpectedly passed' >&2; exit 1; }
[[ "$(wc -l < "$counter" | tr -d ' ')" == 1 ]] || {
  echo 'resume repeated the candidate artifact preparation command' >&2
  exit 1
}
jq -e '[.results[] | select(.phase == "prepare")] | length == 1' "$output/phase-receipts.json" >/dev/null
jq -e '[.results[] | select(.phase == "source_verification_build")] | length == 1' "$output/phase-receipts.json" >/dev/null
jq -e '.adopterAcceptance == "failed" and .cleanupState == "passed"' "$output/acceptance.json" >/dev/null

# Exercise the same Rust receipt consumer for the expensive scoped phases.
# This keeps the shell fixture small while proving candidate/public/close are
# persisted, resumed, and rejected on identity/evidence drift.
scoped="$tmp/scoped"
mkdir -p "$scoped"
identity="$scoped/identity.json"
jq -n \
  '{source:{repository:"sha256:repository",commit:("a" * 40),cargoLockDigest:"sha256:lock"},candidate:{version:"0.2.90",tag:"v0.2.90",manifestDigest:"sha256:manifest",assets:{"artifact.tar.gz":"sha256:artifact"}},previous:null,runtime:{version:"0.2.90",digest:"sha256:runtime"},target:"x86_64-unknown-linux-gnu",isolation:{home:("/tmp/" + "acceptance-home"),xdgConfigHome:("/tmp/" + "acceptance-xdg"),tmp:("/tmp/" + "acceptance-tmp"),cargoHome:("/tmp/" + "acceptance-cargo")}}' \
  > "$identity"
record_scoped_phase() {
  local scope="$1" phase="$2" receipt="$3"
  local evidence="$scoped/$scope-$phase.json"
  printf '{"scope":"%s","phase":"%s"}\n' "$scope" "$phase" > "$evidence"
  "$helper" acceptance-record --scope "$scope" --identity "$identity" --receipts "$receipt" --phase "$phase" --evidence "$evidence" >/dev/null
}
candidate_receipts="$scoped/candidate-receipts.json"
record_scoped_phase candidate prepare "$candidate_receipts"
record_scoped_phase candidate source_verification_build "$candidate_receipts"
record_scoped_phase candidate candidate_acceptance "$candidate_receipts"
candidate_plan="$scoped/candidate-plan.json"
"$helper" acceptance-plan --scope candidate --identity "$identity" --receipts "$candidate_receipts" --output "$candidate_plan"
jq -e '[.actions[] | select(.phase == "candidate_acceptance" and .action == "reuse")] | length == 1' "$candidate_plan" >/dev/null
jq -e '[.actions[] | select(.phase == "close" and .action == "run")] | length == 1' "$candidate_plan" >/dev/null
record_scoped_phase candidate close "$candidate_receipts"
jq -e '[.results[] | select(.phase == "close")] | length == 1' "$candidate_receipts" >/dev/null
"$helper" acceptance-plan --scope candidate --identity "$identity" --receipts "$candidate_receipts" --output "$scoped/candidate-closed-plan.json"
jq -e '[.actions[] | select(.phase == "close" and .action == "reuse")] | length == 1' "$scoped/candidate-closed-plan.json" >/dev/null

candidate_digest_before="$(shasum -a 256 "$candidate_receipts" | awk '{print $1}')"
printf '{"scope":"candidate","phase":"candidate_acceptance","changed":true}\n' > "$scoped/candidate-changed.json"
set +e
"$helper" acceptance-record --scope candidate --identity "$identity" --receipts "$candidate_receipts" --phase candidate_acceptance --evidence "$scoped/candidate-changed.json" >/dev/null 2>&1
duplicate_status=$?
set -e
[[ "$duplicate_status" -ne 0 ]] || { echo 'changed duplicate phase was accepted' >&2; exit 1; }
candidate_digest_after="$(shasum -a 256 "$candidate_receipts" | awk '{print $1}')"
[[ "$candidate_digest_before" == "$candidate_digest_after" ]] || { echo 'duplicate phase changed historical receipt' >&2; exit 1; }

changed_identity="$scoped/changed-identity.json"
jq '.candidate.version = "0.2.91"' "$identity" > "$changed_identity"
set +e
"$helper" acceptance-plan --scope candidate --identity "$changed_identity" --receipts "$candidate_receipts" --output "$scoped/changed-plan.json" >/dev/null 2>&1
identity_status=$?
set -e
[[ "$identity_status" -ne 0 ]] || { echo 'identity-mismatched receipt was accepted' >&2; exit 1; }

public_receipts="$scoped/public-receipts.json"
record_scoped_phase public prepare "$public_receipts"
record_scoped_phase public source_verification_build "$public_receipts"
record_scoped_phase public publish "$public_receipts"
record_scoped_phase public public_acceptance "$public_receipts"
public_plan="$scoped/public-plan.json"
"$helper" acceptance-plan --scope public --identity "$identity" --receipts "$public_receipts" --output "$public_plan"
jq -e '[.actions[] | select(.phase == "publish" and .action == "reuse")] | length == 1' "$public_plan" >/dev/null
jq -e '[.actions[] | select(.phase == "public_acceptance" and .action == "reuse")] | length == 1' "$public_plan" >/dev/null
jq -e '[.actions[] | select(.phase == "close" and .action == "run")] | length == 1' "$public_plan" >/dev/null
record_scoped_phase public close "$public_receipts"
jq -e '[.results[] | select(.phase == "close")] | length == 1' "$public_receipts" >/dev/null
"$helper" acceptance-plan --scope public --identity "$identity" --receipts "$public_receipts" --output "$scoped/public-closed-plan.json"
jq -e '[.actions[] | select(.phase == "close" and .action == "reuse")] | length == 1' "$scoped/public-closed-plan.json" >/dev/null
echo 'adopter acceptance resume regression passed'
