#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"
validator="$repo_root/tests/release/validate_adopter_acceptance_receipt.sh"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-receipt-validation.XXXXXX")"
cleanup() { find "$test_root" -depth -mindepth 0 -delete; }
trap cleanup EXIT

phases="$test_root/phases.json"
jq -n '{schemaVersion:1,results:[{phase:"close",status:"succeeded"}]}' > "$phases"

install_receipt="$test_root/install.json"
jq -n '{schemaVersion:1,tag:"v0.2.91",adopterAcceptance:"passed",cleanupState:"passed",closeDecision:{validated:true}}' > "$install_receipt"
"$validator" --receipt "$install_receipt" --phases "$phases" --tag v0.2.91 --kind install

upgrade_receipt="$test_root/upgrade.json"
jq -n '{schemaVersion:1,fromTag:"v0.2.90",toTag:"v0.2.91",adopterAcceptance:"passed",cleanupState:"passed",closeDecision:{validated:true}}' > "$upgrade_receipt"
"$validator" --receipt "$upgrade_receipt" --phases "$phases" --tag v0.2.91 --kind upgrade

if "$validator" --receipt "$upgrade_receipt" --phases "$phases" --tag v0.2.92 --kind upgrade >/dev/null 2>&1; then
  printf 'upgrade receipt bound to a different tag must fail\n' >&2
  exit 1
fi

not_applicable="$test_root/not-applicable.json"
jq -n '{schemaVersion:1,releasePublished:true,adopterAcceptance:"not_applicable",fromTag:null,toTag:"v0.2.91",reason:"no previous published Release"}' > "$not_applicable"
"$validator" --receipt "$not_applicable" --tag v0.2.91 --kind upgrade

printf 'adopter acceptance receipt validation tests passed\n'
