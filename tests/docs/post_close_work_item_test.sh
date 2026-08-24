#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
wrapper="$root/tests/docs/post_close_work_item.py"
promoter="$root/tests/docs/promote_closed_work_item.py"
work_item=WI-255-recovery-read-side
tmp=$(mktemp -d "${TMPDIR:-/tmp}/post-close-work-item.XXXXXX")
cleanup() {
  find "$tmp" -depth -mindepth 0 -delete
}
trap cleanup EXIT

fixture="$tmp/repository"
mkdir -p \
  "$fixture/.ai/work-items/archive" \
  "$fixture/.ai/evidence" \
  "$fixture/.ai/decisions" \
  "$fixture/docs/work-items" \
  "$fixture/docs/reference"

cp "$root/.ai/project.json" "$fixture/.ai/project.json"
cp "$root/.ai/work-items/archive/$work_item.archive.json" \
  "$root/.ai/work-items/archive/$work_item.contract.json" \
  "$fixture/.ai/work-items/archive/"
cp "$root/.ai/evidence/$work_item.verification.json" "$fixture/.ai/evidence/"
cp "$root/.ai/decisions/$work_item.finalize.json" \
  "$root"/.ai/decisions/"$work_item".finalize.*.json \
  "$root/.ai/decisions/$work_item.close.json" \
  "$fixture/.ai/decisions/"
if test -f "$root/.ai/decisions/$work_item.recovery.json"; then
  cp "$root/.ai/decisions/$work_item.recovery.json" "$fixture/.ai/decisions/"
fi
cp "$root/docs/work-items/$work_item.md" \
  "$root/docs/work-items/$work_item.zh-CN.md" \
  "$root/docs/work-items/$work_item.ja.md" \
  "$fixture/docs/work-items/"
cp "$root/docs/reference/reference-parity.md" \
  "$root/docs/reference/reference-parity.zh-CN.md" \
  "$root/docs/reference/reference-parity.ja.md" \
  "$fixture/docs/reference/"

# Build a deliberately pre-promotion fixture even when the source repository
# has already promoted WI-255. The immutable evidence remains current; only
# the six controlled documentation projections are reset to their conditional
# form for this regression.
python3 - "$fixture" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
for suffix in ("", ".zh-CN", ".ja"):
    path = root / "docs/work-items" / f"WI-255-recovery-read-side{suffix}.md"
    text = path.read_text(encoding="utf-8")
    text = text.replace("status: implemented\n", "status: in_progress\n", 1)
    path.write_text(text, encoding="utf-8")

rows = {
    "": ("Implemented", "In progress → Implemented after verified close"),
    ".zh-CN": ("已实现", "进行中 → 验证关闭后已实现"),
    ".ja": ("Implemented", "In progress → verified close 後 Implemented"),
}
for suffix, (current, pending) in rows.items():
    path = root / "docs/reference" / f"reference-parity{suffix}.md"
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    lines = [
        line.replace(f"| {current} |", f"| {pending} |", 1)
        if "WI-255" in line
        else line
        for line in lines
    ]
    path.write_text("".join(lines), encoding="utf-8")
PY

git -C "$fixture" init -b main >/dev/null
git -C "$fixture" config user.name 'AI Cockpit Fixture'
git -C "$fixture" config user.email 'fixture@example.invalid'
git -C "$fixture" add .
git -C "$fixture" commit -m 'prepare post-close fixture' >/dev/null
git init --bare "$tmp/origin.git" >/dev/null
git -C "$fixture" remote add origin "$tmp/origin.git"
git -C "$fixture" push -u origin main >/dev/null
git --git-dir="$tmp/origin.git" symbolic-ref HEAD refs/heads/main

revision=$(git -C "$fixture" rev-parse HEAD)
repository_id=$(jq -r .repositoryId "$fixture/.ai/project.json")
plan="$tmp/post-close-plan.json"
python3 "$wrapper" --repo "$fixture" --work-item "$work_item" --plan-out "$plan" \
  >"$tmp/plan.out"

python3 - "$plan" "$revision" "$repository_id" <<'PY'
import json
import re
import sys
from pathlib import Path

plan = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
revision = sys.argv[2]
repository_id = sys.argv[3]
assert plan["schemaVersion"] == 1
assert plan["kind"] == "ai-cockpit.post-close-documentation-plan"
assert plan["repositoryId"] == repository_id
assert plan["workItemId"] == "WI-255-recovery-read-side"
assert plan["base"] == {"branch": "main", "remote": "origin", "revision": revision}
assert plan["repositoryRevision"] == revision
assert plan["terminalEvidence"]["finalizationSequence"] == 2
assert plan["terminalEvidence"]["close"]["path"] == ".ai/decisions/WI-255-recovery-read-side.close.json"
assert plan["terminalEvidence"]["finalization"]["path"].startswith(
    ".ai/decisions/WI-255-recovery-read-side.finalize."
)
expected_paths = [
    "docs/reference/reference-parity.ja.md",
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/work-items/WI-255-recovery-read-side.ja.md",
    "docs/work-items/WI-255-recovery-read-side.md",
    "docs/work-items/WI-255-recovery-read-side.zh-CN.md",
]
assert [change["path"] for change in plan["changes"]] == expected_paths
for binding in (
    plan["terminalEvidence"]["archive"],
    plan["terminalEvidence"]["contract"],
    plan["terminalEvidence"]["verification"],
    plan["terminalEvidence"]["finalization"],
    plan["terminalEvidence"]["close"],
):
    assert re.fullmatch(r"sha256:[0-9a-f]{64}", binding["digest"])
for change in plan["changes"]:
    assert re.fullmatch(r"sha256:[0-9a-f]{64}", change["beforeDigest"])
    assert re.fullmatch(r"sha256:[0-9a-f]{64}", change["afterDigest"])
    assert change["beforeDigest"] != change["afterDigest"]
PY

immutable_before=$(find "$fixture/.ai" -type f -print0 | sort -z | xargs -0 shasum -a 256)
docs_before=$(find "$fixture/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)
python3 "$wrapper" --repo "$fixture" --work-item "$work_item" --apply-plan "$plan" \
  >"$tmp/apply.out"
python3 "$promoter" --repo "$fixture" --work-item "$work_item" --check >/dev/null
immutable_after=$(find "$fixture/.ai" -type f -print0 | sort -z | xargs -0 shasum -a 256)
docs_after=$(find "$fixture/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)
test "$immutable_before" = "$immutable_after"
test "$docs_before" != "$docs_after"
test "$(jq -r .state "$tmp/apply.out")" = promoted

python3 "$wrapper" --repo "$fixture" --work-item "$work_item" --apply-plan "$plan" \
  >"$tmp/reapply.out"
test "$(jq -r .state "$tmp/reapply.out")" = current
test "$docs_after" = "$(find "$fixture/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)"

dirty="$tmp/dirty-repository"
git clone -q "$tmp/origin.git" "$dirty"
printf 'unrelated user change\n' > "$dirty/unexpected.txt"
dirty_docs_before=$(find "$dirty/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)
if python3 "$wrapper" --repo "$dirty" --work-item "$work_item" --apply-plan "$plan" \
  >"$tmp/dirty.out" 2>"$tmp/dirty.err"; then
  echo 'post-close apply accepted an unexpected dirty path' >&2
  exit 1
fi
grep -Fq 'unexpected dirty path' "$tmp/dirty.err"
test "$dirty_docs_before" = \
  "$(find "$dirty/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)"
if python3 "$wrapper" --repo "$dirty" --work-item "$work_item" \
  --plan-out "$tmp/dirty-plan.json" >"$tmp/dirty-plan.out" 2>"$tmp/dirty-plan.err"; then
  echo 'post-close planning accepted an unexpected dirty path' >&2
  exit 1
fi
grep -Fq 'clean repository' "$tmp/dirty-plan.err"

unknown="$tmp/unknown-plan-repository"
git clone -q "$tmp/origin.git" "$unknown"
jq '.unexpectedField = true' "$plan" > "$tmp/unknown-plan.json"
unknown_docs_before=$(find "$unknown/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)
if python3 "$wrapper" --repo "$unknown" --work-item "$work_item" \
  --apply-plan "$tmp/unknown-plan.json" >"$tmp/unknown.out" 2>"$tmp/unknown.err"; then
  echo 'post-close apply accepted an unknown plan field' >&2
  exit 1
fi
grep -Fq 'unknown plan field' "$tmp/unknown.err"
test "$unknown_docs_before" = \
  "$(find "$unknown/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)"

nested="$tmp/nested-unknown-plan-repository"
git clone -q "$tmp/origin.git" "$nested"
jq '.changes[0].unexpectedField = true' "$plan" > "$tmp/nested-unknown-plan.json"
if python3 "$wrapper" --repo "$nested" --work-item "$work_item" \
  --apply-plan "$tmp/nested-unknown-plan.json" \
  >"$tmp/nested-unknown.out" 2>"$tmp/nested-unknown.err"; then
  echo 'post-close apply accepted a nested unknown plan field' >&2
  exit 1
fi
grep -Fq 'unknown plan field' "$tmp/nested-unknown.err"

descendant="$tmp/descendant-repository"
git clone -q "$tmp/origin.git" "$descendant"
git -C "$descendant" config user.name 'AI Cockpit Fixture'
git -C "$descendant" config user.email 'fixture@example.invalid'
printf 'descendant commit\n' > "$descendant/descendant.txt"
git -C "$descendant" add descendant.txt
git -C "$descendant" commit -m 'create unsynchronized descendant' >/dev/null
if python3 "$wrapper" --repo "$descendant" --work-item "$work_item" \
  --plan-out "$tmp/descendant-plan.json" \
  >"$tmp/descendant.out" 2>"$tmp/descendant.err"; then
  echo 'post-close plan accepted a descendant of origin/main' >&2
  exit 1
fi
grep -Fq 'exactly synchronized with origin/main' "$tmp/descendant.err"

symlink_output_repo="$tmp/symlink-output-repository"
git clone -q "$tmp/origin.git" "$symlink_output_repo"
printf 'sentinel\n' > "$tmp/plan-target.json"
ln -s "$tmp/plan-target.json" "$tmp/plan-output-link.json"
if python3 "$wrapper" --repo "$symlink_output_repo" --work-item "$work_item" \
  --plan-out "$tmp/plan-output-link.json" \
  >"$tmp/symlink-output.out" 2>"$tmp/symlink-output.err"; then
  echo 'post-close plan followed a symlink output path' >&2
  exit 1
fi
grep -Fq 'plan output must be a regular non-symlink path' "$tmp/symlink-output.err"
test "$(cat "$tmp/plan-target.json")" = sentinel

symlink_input_repo="$tmp/symlink-input-repository"
git clone -q "$tmp/origin.git" "$symlink_input_repo"
ln -s "$plan" "$tmp/plan-input-link.json"
if python3 "$wrapper" --repo "$symlink_input_repo" --work-item "$work_item" \
  --apply-plan "$tmp/plan-input-link.json" \
  >"$tmp/symlink-input.out" 2>"$tmp/symlink-input.err"; then
  echo 'post-close apply followed a symlink input path' >&2
  exit 1
fi
grep -Fq 'plan input must be a regular non-symlink file' "$tmp/symlink-input.err"

duplicate_repo="$tmp/duplicate-plan-repository"
git clone -q "$tmp/origin.git" "$duplicate_repo"
python3 - "$plan" "$tmp/duplicate-plan.json" <<'PY'
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text(encoding="utf-8")
Path(sys.argv[2]).write_text(source.replace("{", '{"schemaVersion":1,', 1), encoding="utf-8")
PY
if python3 "$wrapper" --repo "$duplicate_repo" --work-item "$work_item" \
  --apply-plan "$tmp/duplicate-plan.json" \
  >"$tmp/duplicate.out" 2>"$tmp/duplicate.err"; then
  echo 'post-close apply accepted duplicate JSON keys' >&2
  exit 1
fi
grep -Fq 'duplicate key' "$tmp/duplicate.err"

echo 'post-close Work Item wrapper regression passed'
