#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
policy="$root/tests/workflow/resource_finalization_policy.sh"

[[ -x "$policy" ]] || chmod +x "$policy"

# Positive repository check.
"$policy" "$root"

# Regression: removing one language-neutral transition must fail the gate. The
# fixture is isolated and disposable; no repository policy file is mutated.
fixture=$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-finalization-policy.XXXXXX")
cleanup() { rm -rf "$fixture"; }
trap cleanup EXIT

mkdir -p "$fixture/.ai" "$fixture/docs/reference" "$fixture/docs/work-items"
for path in \
  AGENTS.md \
  .ai/README.md \
  docs/reference/agent-workflow.md \
  docs/reference/agent-workflow.ja.md \
  docs/reference/agent-workflow.zh-CN.md \
  docs/reference/repository-workflow.md \
  docs/reference/repository-workflow.ja.md \
  docs/reference/repository-workflow.zh-CN.md \
  docs/reference/work-item-lifecycle-closure.md \
  docs/reference/work-item-lifecycle-closure.ja.md \
  docs/reference/work-item-lifecycle-closure.zh-CN.md \
  docs/reference/reference-parity.md \
  docs/reference/reference-parity.ja.md \
  docs/reference/reference-parity.zh-CN.md \
  docs/work-items/WI-160-resource-finalization-baseline.md \
  docs/work-items/WI-160-resource-finalization-baseline.ja.md \
  docs/work-items/WI-160-resource-finalization-baseline.zh-CN.md \
  docs/work-items/WI-161-historical-runtime-close.md \
  docs/work-items/WI-161-historical-runtime-close.ja.md \
  docs/work-items/WI-161-historical-runtime-close.zh-CN.md; do
  mkdir -p "$fixture/$(dirname "$path")"
  cp "$root/$path" "$fixture/$path"
done

python3 - "$fixture/docs/reference/agent-workflow.zh-CN.md" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace("finalize-verify", "verification-step-removed")
path.write_text(text, encoding="utf-8")
PY

if "$policy" "$fixture" >/dev/null 2>&1; then
  printf 'resource finalization policy regression did not fail after removing finalize-verify\n' >&2
  exit 1
fi

# Regression: reversing resource cleanup and close must fail the policy gate.
cp "$root/docs/reference/agent-workflow.zh-CN.md" \
  "$fixture/docs/reference/agent-workflow.zh-CN.md"
python3 - "$fixture/AGENTS.md" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace(
    "archive → synchronize default branch → perform exact provider cleanup under the accepted plan\n→ record finalize receipt → finalize-verify → close",
    "archive → close → synchronize default branch → remove exact branch/worktree",
)
path.write_text(text, encoding="utf-8")
PY
if "$policy" "$fixture" >"$fixture/ordering.out" 2>"$fixture/ordering.err"; then
  printf 'resource finalization policy accepted branch cleanup after close\n' >&2
  exit 1
fi
grep -Fq 'AGENTS.md: missing resource-bound order' "$fixture/ordering.err"

# Regression: a correct route must not mask a contradictory claim elsewhere
# in the same instruction projection.
cp "$root/AGENTS.md" "$fixture/AGENTS.md"
python3 - "$fixture/AGENTS.md" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text += "\nThe close command deletes the exact Work Item branch and worktree.\n"
path.write_text(text, encoding="utf-8")
PY
if "$policy" "$fixture" >"$fixture/contradiction.out" 2>"$fixture/contradiction.err"; then
  printf 'resource finalization policy accepted a contradictory close cleanup claim\n' >&2
  exit 1
fi
grep -Fq 'AGENTS.md: contradictory close cleanup claim' "$fixture/contradiction.err"

# Regression: localized projections must reject their own contradictory
# cleanup-after-close language while preserving each valid canonical projection.
localized_acceptances=()
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
for localized_case in \
  'docs/reference/agent-workflow.ja.md|close コマンド自体が Work Item の branch と worktree を削除します。' \
  'docs/reference/agent-workflow.zh-CN.md|close 命令本身会删除 Work Item 分支和工作树。' \
  'docs/reference/agent-workflow.md|For a resource-bound Work Item, after close the branch and worktree are deleted.' \
  'docs/reference/agent-workflow.md|For a resource-bound Work Item, after close the operator removes the branch and worktree.' \
  'docs/reference/repository-workflow.ja.md|resource-bound の Work Item は close の後に branch/worktree が削除されます。' \
  'docs/reference/repository-workflow.zh-CN.md|有外部资源的 Work Item 在 close 后，分支和工作树会被删除。' \
  'docs/reference/repository-workflow.ja.md|resource-bound の Work Item は close の後に branch/worktree を削除します。' \
  'docs/reference/repository-workflow.zh-CN.md|有外部资源的 Work Item 在 close 后删除分支和工作树。'; do
  path=${localized_case%%|*}
  claim=${localized_case#*|}
  for ordering_doc in "${ordering_docs[@]}"; do
    cp "$root/$ordering_doc" "$fixture/$ordering_doc"
  done
  python3 - "$fixture/$path" "$claim" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
path.write_text(
    path.read_text(encoding="utf-8") + "\n" + sys.argv[2] + "\n",
    encoding="utf-8",
)
PY
  if "$policy" "$fixture" >"$fixture/localized.out" 2>"$fixture/localized.err"; then
    localized_acceptances+=("$path")
  else
    grep -Fq "$path: contradictory close cleanup claim" "$fixture/localized.err"
  fi
done
if ((${#localized_acceptances[@]} > 0)); then
  printf 'resource finalization policy accepted localized contradictions:\n' >&2
  printf ' - %s\n' "${localized_acceptances[@]}" >&2
  exit 1
fi

# The scanner must distinguish a negated clarification from an actual
# contradiction; localized policy language commonly states that close itself
# does not delete the branch or worktree.
for localized_case in \
  'docs/reference/agent-workflow.md|For a resource-bound Work Item, after close the branch is not deleted.' \
  'docs/reference/agent-workflow.md|The close command itself does not delete the Work Item branch or worktree.' \
  'AGENTS.md|resource-bound Work Item cleanup occurs before close. For a no-resource Work Item, after close the branch is removed.' \
  'docs/reference/agent-workflow.md|Resource-bound Work Item cleanup occurs before close; for a no-resource Work Item, after close the branch and worktree are removed.' \
  'docs/reference/agent-workflow.md|Resource-bound Work Item cleanup occurs before close, for a no-resource Work Item the branch and worktree are removed after close.' \
  'docs/reference/repository-workflow.ja.md|resource-bound の Work Item は close 前に cleanup を行い、no-resource route は close 後に branch/worktree を削除します。' \
  'docs/reference/repository-workflow.ja.md|resource-bound の Work Item は close 前に cleanup を行います。no-resource route は close 後に branch/worktree を削除します。' \
  'docs/reference/repository-workflow.zh-CN.md|有外部资源的 Work Item 在 close 前完成清理；无外部资源的路径 close 后删除精确 branch/worktree。' \
  'docs/reference/repository-workflow.zh-CN.md|有外部资源的 Work Item 在 close 前完成清理。无资源的 Work Item close 后删除分支和工作树。' \
  'docs/reference/repository-workflow.ja.md|resource-bound の Work Item は close の後に branch/worktree を削除してはいけません。' \
  'docs/reference/repository-workflow.ja.md|resource-bound の Work Item は close の後に branch/worktree が削除されてはならない。' \
  'docs/reference/agent-workflow.ja.md|close コマンド自体は Work Item の branch と worktree を削除しません。' \
  'docs/reference/repository-workflow.zh-CN.md|有外部资源的 Work Item 在 close 后，分支和工作树不应被删除。' \
  'docs/reference/repository-workflow.zh-CN.md|有外部资源的 Work Item 在 close 后，分支和工作树不应该被删除。' \
  'docs/reference/agent-workflow.zh-CN.md|close 命令本身不会删除 Work Item 分支和工作树。'; do
  path=${localized_case%%|*}
  claim=${localized_case#*|}
  for ordering_doc in "${ordering_docs[@]}"; do
    cp "$root/$ordering_doc" "$fixture/$ordering_doc"
  done
  python3 - "$fixture/$path" "$claim" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
path.write_text(
    path.read_text(encoding="utf-8") + "\n" + sys.argv[2] + "\n",
    encoding="utf-8",
)
PY
  if ! "$policy" "$fixture" >"$fixture/valid-negation.out" 2>"$fixture/valid-negation.err"; then
    printf 'resource finalization policy rejected valid localized negation in %s:\n' "$path" >&2
    cat "$fixture/valid-negation.err" >&2
    exit 1
  fi
done

printf 'resource finalization policy tests passed\n'
