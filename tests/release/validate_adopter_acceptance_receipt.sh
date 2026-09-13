#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: validate_adopter_acceptance_receipt.sh --receipt FILE --phases FILE --tag vX.Y.Z --kind install|upgrade
USAGE
}

die() {
  printf 'adopter acceptance receipt validation failed: %s\n' "$*" >&2
  exit 1
}

receipt=''
phases=''
tag=''
kind=''
while (($# > 0)); do
  case "$1" in
    --receipt) receipt=${2:?missing value for --receipt}; shift 2 ;;
    --phases) phases=${2:?missing value for --phases}; shift 2 ;;
    --tag) tag=${2:?missing value for --tag}; shift 2 ;;
    --kind) kind=${2:?missing value for --kind}; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done

[[ -n "$receipt" && -f "$receipt" ]] || die 'receipt file is missing'
[[ -n "$tag" ]] || die 'tag is missing'
[[ "$kind" == install || "$kind" == upgrade ]] || die 'kind must be install or upgrade'

if jq -e '.adopterAcceptance == "not_applicable"' "$receipt" >/dev/null 2>&1; then
  [[ "$kind" == upgrade ]] || die 'only an upgrade receipt may be not applicable'
  jq -e --arg tag "$tag" \
    '.schemaVersion == 1 and
     .adopterAcceptance == "not_applicable" and
     .releasePublished == true and
     .fromTag == null and
     .toTag == $tag and
     (.reason | type == "string" and length > 0)' \
    "$receipt" >/dev/null || die 'not-applicable receipt is not bound to the requested Release'
  exit 0
fi

jq -e \
  '.schemaVersion == 1 and
   .adopterAcceptance == "passed" and
   .cleanupState == "passed" and
   .closeDecision.validated == true' \
  "$receipt" >/dev/null || die 'receipt state or close decision is invalid'

if [[ "$kind" == upgrade ]]; then
  jq -e --arg tag "$tag" \
    '.toTag == $tag and (.fromTag | type == "string") and .fromTag != $tag' \
    "$receipt" >/dev/null || die 'upgrade receipt is not bound to the requested Release tag'
else
  jq -e --arg tag "$tag" '.tag == $tag' "$receipt" >/dev/null || \
    die 'install receipt is not bound to the requested Release tag'
fi

[[ -n "$phases" && -f "$phases" ]] || die 'phase receipt file is missing'
jq -e '.schemaVersion == 1 and ([.results[]? | select(.phase == "close" and .status == "succeeded")] | length == 1)' \
  "$phases" >/dev/null || die 'phase receipt does not contain exactly one succeeded close phase'
