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
  for work_item in WI-177 WI-178 WI-179; do
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
  done
done

printf 'parity status check passed\n'
