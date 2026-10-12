#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
policy="$root/tests/workflow/resource_finalization_policy.sh"

[[ -x "$policy" ]] || chmod +x "$policy"

# Positive repository check.
"$policy" "$root"

# The accepted WI-1076 history is a two-merge graph. Pin its parents, trees,
# stage paths, and modes independently of the Rust implementation so a
# later cleanup cannot silently reuse only a three-dot comparison or an
# incomplete path allowlist.
python3 - "$root" <<'PY'
from pathlib import Path
import json
import subprocess
import sys

root = Path(sys.argv[1])

def git(*args):
    return subprocess.run(
        ["git", "-C", str(root), *args],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ).stdout.strip()

source_base = "095e185b710b0c0016c655d3792f1b3438a62bac"
source_head = "b0e98ae9efe54cde951bb8f547762736411c1628"
source_merge = "686a71a07cc0bd6f3d76ab7261c829067db7ee26"
finish = "d651f93d4a8bcbe9413d055d501ceae6f7d0132e"
archive = "700f6cd5dcb81833aa63d38913908c5543865a8d"
docs = "0f1d54b334104a67eb383f5946ce2305b7b0ba94"
sync_merge = "ae1d0db0221c3afed297b05a0af34b4cd5f0b6c9"

def parents(revision):
    fields = git("show", "-s", "--format=%P", revision).split()
    return fields

assert parents(source_merge) == [source_base, source_head]
assert parents(archive) == [finish]
assert parents(docs) == [archive]
assert parents(sync_merge) == [source_merge, docs]
assert git("rev-parse", f"{source_merge}^{{tree}}") == git("rev-parse", f"{source_head}^{{tree}}")
assert git("rev-parse", f"{sync_merge}^{{tree}}") == git("rev-parse", f"{docs}^{{tree}}")
assert git("merge-base", source_merge, docs) == source_head
remote_main = git("rev-parse", "--verify", "refs/remotes/origin/main")
assert subprocess.run(
    ["git", "-C", str(root), "merge-base", "--is-ancestor", sync_merge, remote_main]
).returncode == 0
assert subprocess.run(
    ["git", "-C", str(root), "merge-base", "--is-ancestor", source_merge, docs]
).returncode == 1
for ancestor, descendant in ((source_head, finish), (finish, archive), (archive, docs), (docs, sync_merge)):
    assert subprocess.run(
        ["git", "-C", str(root), "merge-base", "--is-ancestor", ancestor, descendant]
    ).returncode == 0

prefix = "WI-1076-governance-recovered-predecessor-projection"
active = f".ai/work-items/active/{prefix}"
archived = f".ai/work-items/archive/{prefix}"
expected = {
    finish: {
        f"M:.ai/evidence/{prefix}.verification.json",
        f"M:{active}.events.jsonl",
        f"M:{active}.outcome.json",
        f"M:{active}.summary.json",
        f"M:{active}.task-report.json",
        f"M:{active}.task-report.md",
    },
    archive: {
        f"D:{active}.events.jsonl",
        f"A:{archived}.archive.json",
        f"R100:{active}.contract.json=>{archived}.contract.json",
        f"R095:{active}.outcome.json=>{archived}.outcome.json",
        f"R100:{active}.summary.json=>{archived}.summary.json",
        f"R100:{active}.task-report.json=>{archived}.task-report.json",
        f"R100:{active}.task-report.md=>{archived}.task-report.md",
        f"A:{archived}.events.jsonl",
    },
    docs: {
        "M:docs/reference/reference-parity.ja.md",
        "M:docs/reference/reference-parity.md",
        "M:docs/reference/reference-parity.zh-CN.md",
        f"M:docs/work-items/{prefix}.ja.md",
        f"M:docs/work-items/{prefix}.md",
        f"M:docs/work-items/{prefix}.zh-CN.md",
    },
}

for revision, wanted in expected.items():
    parent = parents(revision)[0]
    raw = git("diff-tree", "--raw", "--no-commit-id", "-r", "-M", parent, revision)
    actual = set()
    for line in raw.splitlines():
        metadata, *paths = line.split("\t")
        fields = metadata.split()
        old_mode = fields[0][1:]
        new_mode = fields[1]
        status = fields[4]
        assert old_mode in {"000000", "100644"} and new_mode in {"000000", "100644"}
        if status.startswith("R"):
            assert len(paths) == 2
            actual.add(f"{status}:{paths[0]}=>{paths[1]}")
        else:
            assert len(paths) == 1
            actual.add(f"{status}:{paths[0]}")
    assert actual == wanted, (revision, actual, wanted)

def blob(revision, path):
    return git("rev-parse", "--verify", f"{revision}:{path}")

verification = f".ai/evidence/{prefix}.verification.json"
assert blob(finish, verification) == blob(sync_merge, verification)
verification_json = json.loads(git("show", f"{sync_merge}:{verification}"))
assert verification_json["passed"] is True
assert verification_json["receipt"]["runtimeDigest"] == "sha256:b2a43b0a1f7c460f5f4d4969d02b02c4f27d45152b1027d46e68d458a9ec63fd"
manifest = f"{archived}.archive.json"
assert blob(archive, manifest) == blob(sync_merge, manifest)
manifest_json = json.loads(git("show", f"{sync_merge}:{manifest}"))
assert manifest_json["runtimeDigest"] == "sha256:b2a43b0a1f7c460f5f4d4969d02b02c4f27d45152b1027d46e68d458a9ec63fd"
for name in ("contract.json", "events.jsonl", "outcome.json", "summary.json", "task-report.json", "task-report.md"):
    path = f"{archived}.{name}"
    assert blob(archive, path) == blob(sync_merge, path)
PY

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

# `finalize` records the already-observed cleanup result; it does not delete
# provider resources itself.
cp "$root/docs/reference/agent-workflow.md" "$fixture/docs/reference/agent-workflow.md"
python3 - "$fixture/docs/reference/agent-workflow.md" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
path.write_text(
    path.read_text(encoding="utf-8")
    + "\nThe finalize command deletes the exact Work Item branch and worktree.\n",
    encoding="utf-8",
)
PY
if "$policy" "$fixture" >"$fixture/finalize.out" 2>"$fixture/finalize.err"; then
  printf 'resource finalization policy accepted a claim that finalize deletes provider resources\n' >&2
  exit 1
fi
grep -Fq 'agent-workflow.md: finalize must record cleanup evidence' "$fixture/finalize.err"

# The cleanup ownership boundary is language-neutral. Localized projections
# must not imply that `finalize` itself deletes provider resources.
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
  'docs/reference/agent-workflow.ja.md|finalize コマンドが branch と worktree を削除します。' \
  'docs/reference/agent-workflow.ja.md|branch と worktree は finalize によって削除されます。' \
  'docs/reference/agent-workflow.zh-CN.md|finalize 命令会删除 Work Item 分支和工作树。' \
  'docs/reference/agent-workflow.zh-CN.md|Work Item 分支和工作树由 finalize 删除。'; do
  path=${localized_case%%|*}
  claim=${localized_case#*|}
  for ordering_doc in "${ordering_docs[@]}"; do
    cp "$root/$ordering_doc" "$fixture/$ordering_doc"
  done
  cp "$root/$path" "$fixture/$path"
  python3 - "$fixture/$path" "$claim" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
path.write_text(path.read_text(encoding="utf-8") + "\n" + sys.argv[2] + "\n", encoding="utf-8")
PY
  if "$policy" "$fixture" >"$fixture/localized-finalize.out" 2>"$fixture/localized-finalize.err"; then
    printf 'resource finalization policy accepted a localized finalize cleanup claim in %s\n' "$path" >&2
    exit 1
  fi
  grep -Fq "$path: finalize must record cleanup evidence" "$fixture/localized-finalize.err"
done

# Localized negations are valid and must remain accepted.
for localized_case in \
  'docs/reference/agent-workflow.ja.md|finalize コマンドは Work Item の branch と worktree を削除しません。' \
  'docs/reference/agent-workflow.zh-CN.md|finalize 命令本身不会删除 Work Item 分支和工作树。'; do
  path=${localized_case%%|*}
  claim=${localized_case#*|}
  for ordering_doc in "${ordering_docs[@]}"; do
    cp "$root/$ordering_doc" "$fixture/$ordering_doc"
  done
  python3 - "$fixture/$path" "$claim" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
path.write_text(path.read_text(encoding="utf-8") + "\n" + sys.argv[2] + "\n", encoding="utf-8")
PY
  if ! "$policy" "$fixture" >"$fixture/localized-finalize-negation.out" 2>"$fixture/localized-finalize-negation.err"; then
    printf 'resource finalization policy rejected valid localized finalize negation in %s:\n' "$path" >&2
    cat "$fixture/localized-finalize-negation.err" >&2
    exit 1
  fi
done

# The resource-bound qualifier remains in force across sentence and paragraph
# boundaries unless a new route is explicitly declared.
for separator in sentence paragraph long-gap; do
  cp "$root/AGENTS.md" "$fixture/AGENTS.md"
  python3 - "$fixture/AGENTS.md" "$separator" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
kind = sys.argv[2]
separator = ". " if kind == "sentence" else ".\n\n"
context = (
    "This paragraph gives unrelated lifecycle background so the rule must retain its route context. " * 4
    if kind == "long-gap"
    else ""
)
path.write_text(
    path.read_text(encoding="utf-8")
    + "\nFor a resource-bound Work Item, cleanup completes before close"
    + separator
    + context
    + "After close, its branch and worktree are deleted.\n",
    encoding="utf-8",
)
PY
  if "$policy" "$fixture" >"$fixture/cross-$separator.out" 2>"$fixture/cross-$separator.err"; then
    printf 'resource finalization policy accepted a cross-%s cleanup contradiction\n' "$separator" >&2
    exit 1
  fi
  grep -Fq 'AGENTS.md: contradictory close cleanup claim' "$fixture/cross-$separator.err"
done

# Regression: localized projections must reject their own contradictory
# cleanup-after-close language while preserving each valid canonical projection.
localized_acceptances=()
for localized_case in \
  'docs/reference/agent-workflow.ja.md|close コマンド自体が Work Item の branch と worktree を削除します。' \
  'docs/reference/agent-workflow.zh-CN.md|close 命令本身会删除 Work Item 分支和工作树。' \
  'docs/reference/agent-workflow.md|For a resource-bound Work Item, after close the branch and worktree are deleted.' \
  'docs/reference/agent-workflow.md|For a resource-bound Work Item, after close the operator removes the branch and worktree.' \
  'docs/reference/repository-workflow.ja.md|resource-bound の Work Item は close の後に branch/worktree が削除されます。' \
  'docs/reference/repository-workflow.zh-CN.md|有外部资源的 Work Item 在 close 后，分支和工作树会被删除。' \
  'docs/reference/repository-workflow.ja.md|resource-bound の Work Item は close の後に branch/worktree を削除します。' \
  'docs/reference/repository-workflow.zh-CN.md|有外部资源的 Work Item 在 close 后删除分支和工作树。' \
  'docs/reference/agent-workflow.ja.md|resource-bound の Work Item は close 前に cleanup を行います。Unrelated explanatory context preserves the route binding across a long gap. Unrelated explanatory context preserves the route binding across a long gap. Unrelated explanatory context preserves the route binding across a long gap. close の後に branch/worktree が削除されます。' \
  'docs/reference/agent-workflow.zh-CN.md|有外部资源的 Work Item 在 close 前完成 cleanup。Unrelated explanatory context preserves the route binding across a long gap. Unrelated explanatory context preserves the route binding across a long gap. Unrelated explanatory context preserves the route binding across a long gap. close 后分支和工作树会被删除。'; do
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
