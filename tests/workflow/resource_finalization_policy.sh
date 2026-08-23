#!/usr/bin/env bash
set -euo pipefail

root=${1:-$(cd "$(dirname "$0")/../.." && pwd -P)}

failures=()
require_file() {
  local path=$1
  if [[ ! -f "$root/$path" || -L "$root/$path" ]]; then
    failures+=("missing or symlinked policy file: $path")
  fi
}

require_text() {
  local path=$1
  local marker=$2
  if [[ -f "$root/$path" && ! -L "$root/$path" ]] && ! grep -Fq -- "$marker" "$root/$path"; then
    failures+=("$path: missing required finalization rule: $marker")
  fi
}

workflow_docs=(
  docs/reference/agent-workflow.md
  docs/reference/agent-workflow.ja.md
  docs/reference/agent-workflow.zh-CN.md
)
parity_docs=(
  docs/reference/reference-parity.md
  docs/reference/reference-parity.ja.md
  docs/reference/reference-parity.zh-CN.md
)
work_item_docs=(
  docs/work-items/WI-160-resource-finalization-baseline.md
  docs/work-items/WI-160-resource-finalization-baseline.ja.md
  docs/work-items/WI-160-resource-finalization-baseline.zh-CN.md
)

for path in "${workflow_docs[@]}" "${parity_docs[@]}" "${work_item_docs[@]}"; do
  require_file "$path"
done

# The protocol names are deliberately language-neutral. Keeping the same names
# in all projections prevents a translated page from silently dropping a gate.
for path in "${workflow_docs[@]}"; do
  for marker in finalize-plan finalize finalize-verify unknown retain; do
    require_text "$path" "$marker"
  done
done

require_text docs/reference/agent-workflow.md 'Resource finalization boundary'
require_text docs/reference/agent-workflow.md 'Runtime integration'
require_text docs/reference/agent-workflow.md 'policy baseline'
require_text docs/reference/agent-workflow.md 'Silent branch deletion is'
require_text docs/reference/agent-workflow.md 'close` must not occur before'
require_text docs/reference/agent-workflow.md 'explicit human decision'

require_text docs/reference/agent-workflow.ja.md 'Resource finalization の境界'
require_text docs/reference/agent-workflow.ja.md 'Runtime 統合'
require_text docs/reference/agent-workflow.ja.md 'policy baseline'
require_text docs/reference/agent-workflow.ja.md 'silent deletionは禁止'
require_text docs/reference/agent-workflow.ja.md '明示的な Human'
require_text docs/reference/agent-workflow.ja.md 'Decision です'
require_text docs/reference/agent-workflow.ja.md 'close`'

require_text docs/reference/agent-workflow.zh-CN.md '资源收尾边界'
require_text docs/reference/agent-workflow.zh-CN.md 'Runtime'
require_text docs/reference/agent-workflow.zh-CN.md '命令'
require_text docs/reference/agent-workflow.zh-CN.md 'policy baseline'
require_text docs/reference/agent-workflow.zh-CN.md '禁止静默删除 branch'
require_text docs/reference/agent-workflow.zh-CN.md '明确的人类决定'
require_text docs/reference/agent-workflow.zh-CN.md '之前不得 `close`'

for path in "${parity_docs[@]}"; do
  require_text "$path" 'WI-160'
  require_text "$path" 'finalize-plan'
  require_text "$path" 'finalize-verify'
done
require_text docs/reference/reference-parity.md 'Runtime command/receipt integration'
require_text docs/reference/reference-parity.ja.md 'Runtime の command と receipt 統合'
require_text docs/reference/reference-parity.zh-CN.md 'Runtime 命令与 receipt 集成'

for path in "${work_item_docs[@]}"; do
  require_text "$path" 'workItemId: WI-160-resource-finalization-baseline'
  require_text "$path" 'finalize-plan'
  require_text "$path" 'finalize-verify'
  require_text "$path" 'unknown'
  require_text "$path" 'retain'
done

if ((${#failures[@]} > 0)); then
  printf 'resource finalization policy failed:\n' >&2
  printf ' - %s\n' "${failures[@]}" >&2
  exit 1
fi

printf 'resource finalization policy passed: tri-lingual closure boundary is present\n'
