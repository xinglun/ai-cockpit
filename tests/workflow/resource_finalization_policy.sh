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
  docs/work-items/WI-161-historical-runtime-close.md
  docs/work-items/WI-161-historical-runtime-close.ja.md
  docs/work-items/WI-161-historical-runtime-close.zh-CN.md
)
ordering_docs=(
  AGENTS.md
  .ai/README.md
  docs/reference/agent-workflow.md
  docs/reference/agent-workflow.ja.md
  docs/reference/agent-workflow.zh-CN.md
  docs/reference/repository-workflow.md
  docs/reference/repository-workflow.ja.md
  docs/reference/repository-workflow.zh-CN.md
  docs/reference/work-item-lifecycle-closure.md
  docs/reference/work-item-lifecycle-closure.ja.md
  docs/reference/work-item-lifecycle-closure.zh-CN.md
)

for path in "${workflow_docs[@]}" "${parity_docs[@]}" "${work_item_docs[@]}"; do
  require_file "$path"
done
for path in "${ordering_docs[@]}"; do
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
require_text docs/reference/agent-workflow.md 'Runtime commands'
require_text docs/reference/agent-workflow.md 'historical'
require_text docs/reference/agent-workflow.md 'Silent branch deletion is'
require_text docs/reference/agent-workflow.md 'close` must not occur before'
require_text docs/reference/agent-workflow.md 'explicit human decision'

require_text docs/reference/agent-workflow.ja.md 'Resource finalization の境界'
require_text docs/reference/agent-workflow.ja.md 'Runtime upgrade'
require_text docs/reference/agent-workflow.ja.md 'historical'
require_text docs/reference/agent-workflow.ja.md 'silent deletionは禁止'
require_text docs/reference/agent-workflow.ja.md '明示的な Human'
require_text docs/reference/agent-workflow.ja.md 'Decision です'
require_text docs/reference/agent-workflow.ja.md 'close`'

require_text docs/reference/agent-workflow.zh-CN.md '资源收尾边界'
require_text docs/reference/agent-workflow.zh-CN.md 'Runtime'
require_text docs/reference/agent-workflow.zh-CN.md '命令'
require_text docs/reference/agent-workflow.zh-CN.md '历史'
require_text docs/reference/agent-workflow.zh-CN.md '禁止静默删除 branch'
require_text docs/reference/agent-workflow.zh-CN.md '明确的人类决定'
require_text docs/reference/agent-workflow.zh-CN.md '之前不得 `close`'

for path in "${parity_docs[@]}"; do
  require_text "$path" 'WI-160'
  require_text "$path" 'finalize-plan'
  require_text "$path" 'finalize-verify'
done
require_text docs/reference/reference-parity.md 'WI-159 implements the Runtime commands'
require_text docs/reference/reference-parity.ja.md 'Runtime command/receipt 統合は WI-159'
require_text docs/reference/reference-parity.zh-CN.md 'WI-159 实现 Runtime 命令与 receipt'

for path in \
  docs/work-items/WI-160-resource-finalization-baseline.md \
  docs/work-items/WI-160-resource-finalization-baseline.ja.md \
  docs/work-items/WI-160-resource-finalization-baseline.zh-CN.md; do
  require_text "$path" 'workItemId: WI-160-resource-finalization-baseline'
  require_text "$path" 'finalize-plan'
  require_text "$path" 'finalize-verify'
  require_text "$path" 'unknown'
  require_text "$path" 'retain'
done

for path in \
  docs/work-items/WI-161-historical-runtime-close.md \
  docs/work-items/WI-161-historical-runtime-close.ja.md \
  docs/work-items/WI-161-historical-runtime-close.zh-CN.md; do
  require_text "$path" 'workItemId: WI-161-historical-runtime-close'
  require_text "$path" 'WI-159'
  require_text "$path" 'historical'
  require_text "$path" 'close'
done

for path in \
  docs/work-items/WI-160-resource-finalization-baseline.md \
  docs/work-items/WI-160-resource-finalization-baseline.ja.md \
  docs/work-items/WI-160-resource-finalization-baseline.zh-CN.md \
  docs/work-items/WI-161-historical-runtime-close.md \
  docs/work-items/WI-161-historical-runtime-close.ja.md \
  docs/work-items/WI-161-historical-runtime-close.zh-CN.md; do
  require_text "$path" 'WI-159'
  require_text "$path" 'WI-161'
done

python3 - "$root" <<'PY'
from pathlib import Path
import sys
import re

root = Path(sys.argv[1])
paths = (
    "AGENTS.md",
    ".ai/README.md",
    "docs/reference/agent-workflow.md",
    "docs/reference/agent-workflow.ja.md",
    "docs/reference/agent-workflow.zh-CN.md",
    "docs/reference/repository-workflow.md",
    "docs/reference/repository-workflow.ja.md",
    "docs/reference/repository-workflow.zh-CN.md",
    "docs/reference/work-item-lifecycle-closure.md",
    "docs/reference/work-item-lifecycle-closure.ja.md",
    "docs/reference/work-item-lifecycle-closure.zh-CN.md",
)
expected = (
    "archive → synchronize default branch → perform exact provider cleanup under the accepted plan "
    "→ record finalize receipt → finalize-verify → close"
)
contradictory_close_cleanup = tuple(
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        # English instructions may claim either that close removes resources,
        # or that close is the actor which removes them.
        r"\bclose(?: command)?\s+(?:itself\s+)?(?:deletes?|removes?|cleans?\s+up)\b.{0,100}\b(?:branch|worktree)\b"
        r"|\b(?:branch|worktree)\b.{0,100}\b(?:is|are)\s+(?:deleted|removed|cleaned\s+up)\s+by\s+close\b",
        # Keep the resource-bound qualifier local so this does not reject the
        # valid no-resource close-then-cleanup route. Sentences are evaluated
        # independently below, so this qualifier cannot bleed into the next
        # route statement.
        r"\bresource-bound\b.{0,80}\bWork Item\b.{0,100}\bafter close\b.{0,120}\b(?:branch|worktree)\b.{0,60}\b(?:is|are)\s+(?!not\b|never\b)(?:deleted|removed|cleaned\s+up)\b"
        r"|\bresource-bound\b.{0,80}\bWork Item\b.{0,100}\bafter close\b.{0,120}(?:the\s+)?(?:authorized\s+)?(?:operator|runtime|provider|agent)\b.{0,30}(?<!not\s)(?<!never\s)(?:deletes?|removes?|cleans?\s+up)\b.{0,60}\b(?:branch|worktree)\b",
        # Japanese projections retain the protocol term `close`; reject claims
        # that it removes/deletes a branch or worktree, in either word order.
        r"\bclose(?:\s*コマンド)?\s*(?:自体|自身).{0,100}(?:branch|worktree|ブランチ|ワークツリー).{0,100}(?:削除|消去|クリーンアップ)"
        r"(?!は?(?:しません|しない|されません|されない|してはいけません|してはいけない|してはならない|してはなりません|しないでください|されてはいけません|されてはいけない|されてはならない|されてはなりません))"
        r"|\bclose\s*(?:が|時に|の実行で|によって).{0,60}(?:branch|worktree|ブランチ|ワークツリー).{0,60}(?:削除|消去|クリーンアップ)"
        r"(?!は?(?:しません|しない|されません|されない|してはいけません|してはいけない|してはならない|してはなりません|しないでください|されてはいけません|されてはいけない|されてはならない|されてはなりません))",
        r"\bresource-bound\b.{0,80}\bWork Item\b.{0,100}\bclose\b.{0,24}(?:の)?(?:後|あと).{0,80}(?:branch|worktree|ブランチ|ワークツリー).{0,60}(?:が|を)?(?:削除|消去|クリーンアップ)"
        r"(?!は?(?:しません|しない|されません|されない|してはいけません|してはいけない|してはならない|してはなりません|しないでください|されてはいけません|されてはいけない|されてはならない|されてはなりません))",
        # Simplified Chinese projections may likewise retain `close` as the
        # protocol term while localizing the action and resource nouns. The
        # fixed-width negative lookbehind prevents `不会` from matching `会`.
        r"\bclose(?:\s*命令)?\s*本身(?:(?<!不)会|将|可以)(?!不).{0,20}(?:删除|移除|清理).{0,100}(?:branch|worktree|分支|工作树)"
        r"|\bclose(?:\s*命令)?(?:(?<!不)会|将)(?!不).{0,60}(?:删除|移除|清理).{0,100}(?:branch|worktree|分支|工作树)",
        r"(?:有外部资源|resource-bound).{0,80}Work Item.{0,100}\bclose\s*后(?!不|不应|不该|不应该|不应当).{0,80}(?:branch|worktree|分支|工作树).{0,60}(?:(?:会|将)?被)?(?<!不应)(?<!不该)(?<!不能)(?<!不会)(?<!不应被)(?<!不该被)(?<!不能被)(?<!不会被)(?<!不应该被)(?<!不应当被)(?<!不应该)(?<!不应当)(?:删除|移除|清理)"
        r"|(?:有外部资源|resource-bound).{0,80}Work Item.{0,100}\bclose\s*后(?!不|不应|不该|不应该|不应当).{0,60}(?:(?<!不应)(?<!不该)(?<!不能)(?<!不会)(?<!不应被)(?<!不该被)(?<!不能被)(?<!不会被)(?<!不应该被)(?<!不应当被)(?<!不应该)(?<!不应当)(?:会|将)?(?:删除|移除|清理)).{0,80}(?:branch|worktree|分支|工作树)",
    )
)
# Separate adjacent route declarations even when authors use clause punctuation
# rather than a sentence boundary. Do not split arbitrary commas/semicolons:
# the delimiter only ends a clause when the next phrase explicitly starts a
# distinct resource-bound or no-resource route.
route_boundary = re.compile(
    r"[,，、;；]\s*(?=(?:(?:for\s+(?:a\s+)?)?(?:no-resource|resource-bound)\b|"
    r"(?:no-resource|resource-bound)\s+(?:route|Work Item)\b|"
    r"有外部资源|无外部资源|无资源))",
    re.IGNORECASE,
)
failures = []
for relative in paths:
    text = " ".join((root / relative).read_text(encoding="utf-8").split())
    if expected not in text:
        failures.append(f"{relative}: missing resource-bound order {expected!r}")
    clauses = []
    # Chinese and Japanese full stops normally have no following space. Keep
    # the whitespace requirement for ASCII punctuation so file names and
    # abbreviations containing periods do not become sentence boundaries.
    for sentence in re.split(r"(?<=[。！？])\s*|(?<=[.!?])\s+", text):
        clauses.extend(route_boundary.split(sentence))
    if any(pattern.search(clause) for pattern in contradictory_close_cleanup for clause in clauses):
        failures.append(f"{relative}: contradictory close cleanup claim")
if failures:
    print("resource-bound close ordering failed:", file=sys.stderr)
    for failure in failures:
        print(f" - {failure}", file=sys.stderr)
    raise SystemExit(1)
PY

if ((${#failures[@]} > 0)); then
  printf 'resource finalization policy failed:\n' >&2
  printf ' - %s\n' "${failures[@]}" >&2
  exit 1
fi

printf 'resource finalization policy passed: tri-lingual closure boundary is present\n'
