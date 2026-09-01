#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
checker="$root/tests/docs/work_item_status_consistency.py"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/work-item-status-consistency.XXXXXX")
cleanup() {
  find "$tmp" -depth -mindepth 0 -delete
}
trap cleanup EXIT

fixture="$tmp/repository"
mkdir -p \
  "$fixture/.ai/decisions" \
  "$fixture/.ai/work-items/archive" \
  "$fixture/docs/reference" \
  "$fixture/docs/work-items"

repository_id='sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
work_item='WI-999-status-drift-fixture'
verifier='WI-998-doc-verifier'

printf '{"repositoryId":"%s"}\n' "$repository_id" > "$fixture/.ai/project.json"
printf '{"workItemId":"%s","repositoryId":"%s"}\n' \
  "$work_item" "$repository_id" \
  > "$fixture/.ai/work-items/archive/$work_item.contract.json"
printf '{"workItemId":"%s","repositoryId":"%s"}\n' \
  "$verifier" "$repository_id" \
  > "$fixture/.ai/work-items/archive/$verifier.contract.json"
printf '{"schemaVersion":1,"workItemId":"%s","predecessorWorkItemId":"%s","repositoryId":"%s","decision":"successor","successorWorkItemId":"WI-1000-status-recovery"}\n' \
  "$work_item" "$work_item" "$repository_id" \
  > "$fixture/.ai/decisions/$work_item.recovery.json"

for language in en zh-CN ja; do
  suffix=
  status_label=Recovered
  case "$language" in
    zh-CN) suffix=.zh-CN; status_label=已恢复 ;;
    ja) suffix=.ja ;;
  esac
  printf '| WI-999 — fixture | %s | `.ai/decisions/%s.recovery.json` |\n' \
    "$status_label" "$work_item" \
    > "$fixture/docs/reference/reference-parity${suffix}.md"
  cat > "$fixture/docs/work-items/$work_item${suffix}.md" <<EOF
---
author: fixture
title: fixture
description: fixture
audience:
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: $verifier
workItemId: $work_item
---
EOF
done

python3 "$checker" --repo "$fixture"

for document in "$fixture"/docs/work-items/$work_item*.md; do
  perl -0pi -e 's/status: recovered/status: historical/' "$document"
done
python3 "$checker" --repo "$fixture"

for document in "$fixture"/docs/work-items/$work_item*.md; do
  perl -0pi -e 's/status: historical/status: recovered/' "$document"
  perl -0pi -e "s/lastVerifiedBy: $verifier/lastVerifiedBy: WI-999-close-verification/" "$document"
done
python3 "$checker" --repo "$fixture"

for document in "$fixture"/docs/work-items/$work_item*.md; do
  perl -0pi -e "s/lastVerifiedBy: WI-999-close-verification/lastVerifiedBy: $verifier/" "$document"
done
perl -0pi -e 's/Recovered/Implemented/' \
  "$fixture/docs/reference/reference-parity.md" \
  "$fixture/docs/reference/reference-parity.ja.md"
perl -0pi -e 's/已恢复/已实现/' \
  "$fixture/docs/reference/reference-parity.zh-CN.md"
printf '\nThis document declares immutable recovery history.\n' \
  >> "$fixture/docs/work-items/$work_item.md"
python3 "$checker" --repo "$fixture"

perl -0pi -e 's/\nThis document declares immutable recovery history\.\n//' \
  "$fixture/docs/work-items/$work_item.md"
if python3 "$checker" --repo "$fixture" >"$tmp/implemented-recovery.out" 2>"$tmp/implemented-recovery.err"; then
  echo 'status consistency accepted recovered projection without immutable recovery evidence' >&2
  exit 1
fi
grep -Fq 'expected one of implemented from authoritative parity and terminal decision' \
  "$tmp/implemented-recovery.err"

perl -0pi -e 's/Implemented/Recovered/' \
  "$fixture/docs/reference/reference-parity.md" \
  "$fixture/docs/reference/reference-parity.ja.md"
perl -0pi -e 's/已实现/已恢复/' \
  "$fixture/docs/reference/reference-parity.zh-CN.md"

for document in "$fixture"/docs/work-items/$work_item*.md; do
  perl -0pi -e 's/status: recovered/status: in_progress/' "$document"
done
if python3 "$checker" --repo "$fixture" >"$tmp/status.out" 2>"$tmp/status.err"; then
  echo 'status consistency accepted stale in_progress frontmatter' >&2
  exit 1
fi
grep -Fq 'expected one of historical,recovered from authoritative parity and terminal decision' "$tmp/status.err"

for document in "$fixture"/docs/work-items/$work_item*.md; do
  perl -0pi -e 's/status: in_progress/status: recovered/' "$document"
done
perl -0pi -e 's/status: recovered/status: implemented/' \
  "$fixture/docs/work-items/$work_item.zh-CN.md"
if python3 "$checker" --repo "$fixture" >"$tmp/language.out" 2>"$tmp/language.err"; then
  echo 'status consistency accepted a three-language status mismatch' >&2
  exit 1
fi
grep -Fq 'three-language status mismatch' "$tmp/language.err"

if AI_COCKPIT_STATUS_DOCS_REPO="$fixture" \
  bash "$root/tests/docs/documentation_acceptance.sh" \
  >"$tmp/integration.out" 2>"$tmp/integration.err"; then
  echo 'documentation acceptance did not consume the status consistency checker' >&2
  exit 1
fi
if ! grep -Fq 'three-language status mismatch' "$tmp/integration.err"; then
  sed -n '1,120p' "$tmp/integration.err" >&2
  exit 1
fi

perl -0pi -e 's/status: implemented/status: recovered/' \
  "$fixture/docs/work-items/$work_item.zh-CN.md"
printf '{"workItemId":"%s","repositoryId":"%s","state":"closed","decisionState":"confirmed","humanDecision":"approved"}\n' \
  "$work_item" "$repository_id" \
  > "$fixture/.ai/decisions/$work_item.close.json"
for parity_document in \
  "$fixture/docs/reference/reference-parity.md" \
  "$fixture/docs/reference/reference-parity.ja.md"; do
  perl -0pi -e 's/Recovered/In progress → Implemented after verified close/' "$parity_document"
done
perl -0pi -e 's/已恢复/进行中 → 验证关闭后已实现/' \
  "$fixture/docs/reference/reference-parity.zh-CN.md"
if python3 "$checker" --repo "$fixture" >"$tmp/conditional.out" 2>"$tmp/conditional.err"; then
  echo 'status consistency accepted conditional parity for a closed Work Item' >&2
  exit 1
fi
grep -Fq 'terminal Work Item retains conditional parity status' "$tmp/conditional.err"

# A bounded documentation-promotion Contract is the one intentional terminal
# exception: its own conditional row remains a pre-archive projection after
# close and must not create a recursive successor.
python3 - "$fixture" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-status-drift-fixture"
path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(path.read_text(encoding="utf-8"))
contract["scope"] = [
    f"docs/work-items/{work_item}.md",
    f"docs/work-items/{work_item}.zh-CN.md",
    f"docs/work-items/{work_item}.ja.md",
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
]
path.write_text(json.dumps(contract) + "\n", encoding="utf-8")
PY
python3 "$checker" --repo "$fixture"

for parity_document in \
  "$fixture/docs/reference/reference-parity.md" \
  "$fixture/docs/reference/reference-parity.ja.md"; do
  perl -0pi -e 's/In progress → Implemented after verified close/Implemented/' "$parity_document"
done
perl -0pi -e 's/进行中 → 验证关闭后已实现/已实现/' \
  "$fixture/docs/reference/reference-parity.zh-CN.md"
for document in "$fixture"/docs/work-items/$work_item*.md; do
  perl -0pi -e 's/status: recovered/status: implemented/' "$document"
done
python3 "$checker" --repo "$fixture"

for case in \
  ".md|This pre-archive status becomes Implemented after verified close." \
  ".zh-CN.md|此预归档状态会在验证关闭后变为已实现。" \
  ".ja.md|この pre-archive status は verified close 後に Implemented になります。"; do
  suffix=${case%%|*}
  conditional=${case#*|}
  document="$fixture/docs/work-items/$work_item$suffix"
  printf '\n%s\n' "$conditional" >> "$document"
  python3 "$checker" --repo "$fixture"
  perl -0pi -e 's/\n[^\n]*\n\z/\n/' "$document"
done

# Recovery records emitted after a supersession/retry are content-bound and
# may use a digest-suffixed filename. The checker must discover that form
# instead of requiring the legacy unsuffixed path.
digest_recovery="$tmp/digest-recovery"
cp -R "$fixture" "$digest_recovery"
rm "$digest_recovery/.ai/decisions/$work_item.close.json"
python3 - "$digest_recovery" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-status-drift-fixture"
source = root / ".ai/decisions" / f"{work_item}.recovery.json"
value = json.loads(source.read_text(encoding="utf-8"))
digest = hashlib.sha256(
    json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
).hexdigest()
source.rename(source.with_name(f"{work_item}.recovery.{digest}.json"))
for suffix, status in (("", "recovered"), (".zh-CN", "recovered"), (".ja", "recovered")):
    document = root / "docs/work-items" / f"{work_item}{suffix}.md"
    text = document.read_text(encoding="utf-8").replace("status: implemented", f"status: {status}")
    document.write_text(text, encoding="utf-8")
for suffix, status in (("", "Recovered"), (".zh-CN", "已恢复"), (".ja", "Recovered")):
    parity = root / "docs/reference" / f"reference-parity{suffix}.md"
    text = parity.read_text(encoding="utf-8").replace("fixture | Implemented", f"fixture | {status}")
    parity.write_text(text, encoding="utf-8")
PY
python3 "$checker" --repo "$digest_recovery"

python3 "$checker" --repo "$root"

echo 'work item status consistency regression passed'
