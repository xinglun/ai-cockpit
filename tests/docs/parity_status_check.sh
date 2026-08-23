#!/usr/bin/env bash
set -euo pipefail

repo="${1:-$(pwd)}"
docs=(
  "$repo/docs/reference/reference-parity.md"
  "$repo/docs/reference/reference-parity.zh-CN.md"
  "$repo/docs/reference/reference-parity.ja.md"
)

for file in "${docs[@]}"; do
  [[ -f "$file" ]] || {
    printf 'parity status check: missing %s\n' "$file" >&2
    exit 1
  }
  for work_item in WI-177 WI-178 WI-179 WI-180 WI-181 WI-182 WI-183 WI-184 WI-185 WI-186; do
    line="$(rg -F "$work_item" "$file" || true)"
    [[ -n "$line" ]] || {
      printf 'parity status check: missing %s in %s\n' "$work_item" "$file" >&2
      exit 1
    }
    case "$file" in
      *.zh-CN.md) printf '%s\n' "$line" | rg -F '已实现' >/dev/null || {
        printf 'parity status check: %s is not marked 已实现 in %s\n' "$work_item" "$file" >&2
        exit 1
      } ;;
      *) printf '%s\n' "$line" | rg -F 'Implemented' >/dev/null || {
        printf 'parity status check: %s is not marked Implemented in %s\n' "$work_item" "$file" >&2
        exit 1
      } ;;
    esac
    if printf '%s\n' "$line" | rg -F 'In progress|进行中' >/dev/null; then
      printf 'parity status check: stale in-progress status for %s in %s\n' "$work_item" "$file" >&2
      exit 1
    fi

    case "$work_item" in
      WI-177)
        required_refs=(
          '.ai/evidence/WI-177-post-release-adopter-v0-2-22/'
          'WI-178'
        ) ;;
      WI-178)
        required_refs=(
          '.ai/evidence/WI-178-post-release-adopter-finalization-reconciliation.verification.json'
          '.ai/decisions/WI-178-post-release-adopter-finalization-reconciliation.finalize.json'
          '.ai/decisions/WI-178-post-release-adopter-finalization-reconciliation.close.json'
        ) ;;
      WI-179)
        required_refs=(
          '.ai/evidence/WI-179-post-release-parity-v0-2-22.verification.json'
          '.ai/decisions/WI-179-post-release-parity-v0-2-22.finalize.json'
          '.ai/decisions/WI-179-post-release-parity-v0-2-22.close.json'
        ) ;;
      WI-180)
        required_refs=(
          '.ai/evidence/WI-180-parity-status-closure-correction.verification.json'
          '.ai/decisions/WI-180-parity-status-closure-correction.finalize.json'
          '.ai/decisions/WI-180-parity-status-closure-correction.close.json'
        ) ;;
      WI-181)
        required_refs=(
          '.ai/evidence/WI-181-parity-evidence-binding.verification.json'
          '.ai/decisions/WI-181-parity-evidence-binding.finalize.json'
          '.ai/decisions/WI-181-parity-evidence-binding.close.json'
        ) ;;
      WI-182)
        required_refs=(
          '.ai/evidence/WI-182-parallel-lease-atomic-install.verification.json'
          '.ai/decisions/WI-182-parallel-lease-atomic-install.finalize.json'
          '.ai/decisions/WI-182-parallel-lease-atomic-install.close.json'
        ) ;;
      WI-183)
        required_refs=(
          '.ai/evidence/WI-183-release-v0-2-23.verification.json'
          '.ai/work-items/archive/WI-183-release-v0-2-23.archive.json'
          '.ai/decisions/WI-183-release-v0-2-23.recovery.json'
        ) ;;
      WI-184)
        required_refs=(
          '.ai/evidence/WI-184-release-v0-2-23-finalization-reconciliation.verification.json'
          '.ai/decisions/WI-184-release-v0-2-23-finalization-reconciliation.finalize.json'
          '.ai/decisions/WI-184-release-v0-2-23-finalization-reconciliation.close.json'
        ) ;;
      WI-185)
        required_refs=(
          '.ai/evidence/WI-185-release-v0-2-23-parity-closure.verification.json'
          '.ai/work-items/archive/WI-185-release-v0-2-23-parity-closure.archive.json'
          '.ai/decisions/WI-185-release-v0-2-23-parity-closure.finalize.json'
          '.ai/decisions/WI-185-release-v0-2-23-parity-closure.close.json'
        ) ;;
      WI-186)
        required_refs=(
          '.ai/evidence/external/v0.2.23/release-adopter-acceptance/acceptance.json'
          '.ai/evidence/external/v0.2.23/adopter/acceptance.json'
          '.ai/evidence/external/v0.2.23/upgrade/acceptance.json'
          '.ai/evidence/WI-186-release-v0-2-23-post-release-acceptance.verification.json'
        ) ;;
    esac
    for ref in "${required_refs[@]}"; do
      printf '%s\n' "$line" | rg -F "$ref" >/dev/null || {
        printf 'parity status check: missing evidence binding %s for %s in %s\n' "$ref" "$work_item" "$file" >&2
        exit 1
      }
      if [[ "$ref" == .ai/* && "$ref" != */ ]]; then
        [[ -f "$repo/$ref" ]] || {
          printf 'parity status check: referenced evidence file does not exist: %s\n' "$repo/$ref" >&2
          exit 1
        }
      fi
    done
  done
done

printf 'parity status check passed\n'
