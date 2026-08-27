#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
helper="$root/tests/docs/promote_closed_work_item.py"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/promote-closed-work-item.XXXXXX")
cleanup() {
  find "$tmp" -depth -mindepth 0 -delete
}
trap cleanup EXIT

fixture="$tmp/repository"
python3 - "$fixture" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
repository_id = "sha256:" + "e" * 64
archive_dir = root / ".ai/work-items/archive"
decision_dir = root / ".ai/decisions"
evidence_dir = root / ".ai/evidence"
docs_dir = root / "docs/work-items"
parity_dir = root / "docs/reference"
for directory in (archive_dir, decision_dir, evidence_dir, docs_dir, parity_dir):
    directory.mkdir(parents=True, exist_ok=True)

def canonical_digest(value: object) -> str:
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    return "sha256:" + hashlib.sha256(encoded).hexdigest()

def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

contract = {
    "baseRevision": "a" * 40,
    "repositoryId": repository_id,
    "resourceContext": {
        "baseBranch": "main",
        "baseRemote": "origin",
        "branch": "codex/wi-999-closed-docs-fixture",
        "provider": "github",
        "pullRequest": "https://example.invalid/pull/999",
        "worktree": "/tmp/wi-999",
    },
    "state": "implementation_active",
    "workItemId": work_item,
}
contract_path = archive_dir / f"{work_item}.contract.json"
write_json(contract_path, contract)
contract_digest = "sha256:" + hashlib.sha256(contract_path.read_bytes()).hexdigest()
archive = {
    "createdAt": "2026-01-01T00:00:00Z",
    "files": {
        "contractDigest": contract_digest,
        "contractPath": f".ai/work-items/archive/{work_item}.contract.json",
    },
    "protocolVersion": 1,
    "state": "archived",
    "workItemId": work_item,
}
write_json(archive_dir / f"{work_item}.archive.json", archive)

evidence_path = f".ai/evidence/{work_item}.verification.json"
evidence = {
    "evidenceSchemaVersion": 2,
    "passed": True,
    "receipt": {
        "passed": True,
        "repositoryId": repository_id,
        "workItemId": work_item,
    },
    "repositoryId": repository_id,
    "repositorySnapshotDigest": "sha256:" + "b" * 64,
    "workItemId": work_item,
}
write_json(root / evidence_path, evidence)

def receipt(disposition: str, merge_commit: str | None) -> dict[str, object]:
    return {
        "after": {"branch": "deleted" if disposition == "deleted" else "present", "pullRequest": "merged" if merge_commit else "open", "worktree": "removed" if disposition == "deleted" else "clean"},
        "before": {"branch": "present", "pullRequest": "merged" if merge_commit else "open", "worktree": "clean"},
        "contractDigest": contract_digest,
        "provider": "github",
        "pullRequest": {
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": "a" * 40,
            "headRevision": "c" * 40,
            "mergeCommit": merge_commit,
            "number": 999,
            "url": "https://example.invalid/pull/999",
        },
        "repositoryId": repository_id,
        "resourceContext": contract["resourceContext"],
        "result": {"disposition": disposition, "failureCodes": [], "unknownCodes": []},
        "runtimeDigest": "sha256:" + "d" * 64,
        "runtimeVersion": "9.9.9",
        "schemaVersion": 1,
        "workItemId": work_item,
    }

root_receipt = receipt("blocked", None)
write_json(decision_dir / f"{work_item}.finalize.json", root_receipt)
root_digest = canonical_digest(root_receipt)
sequence_one_receipt = receipt("retained", "f" * 40)
sequence_one = {
    "predecessorReceiptDigest": root_digest,
    "receipt": sequence_one_receipt,
    "schemaVersion": 1,
    "sequence": 1,
    "transitionId": "merge-observed",
}
sequence_one_name = f"{work_item}.finalize.{canonical_digest(sequence_one).removeprefix('sha256:')}.json"
write_json(decision_dir / sequence_one_name, sequence_one)
sequence_two_receipt = receipt("deleted", "f" * 40)
sequence_two = {
    "predecessorReceiptDigest": canonical_digest(sequence_one_receipt),
    "receipt": sequence_two_receipt,
    "schemaVersion": 1,
    "sequence": 2,
    "transitionId": "cleanup-deleted",
}
sequence_two_name = f"{work_item}.finalize.{canonical_digest(sequence_two).removeprefix('sha256:')}.json"
write_json(decision_dir / sequence_two_name, sequence_two)
sequence_two_path = f".ai/decisions/{sequence_two_name}"
close = {
    "decisionState": "confirmed",
    "finalReport": {"bindings": {"evidenceRefs": [evidence_path], "repositoryId": repository_id, "workItemId": work_item}},
    "humanDecision": "approved",
    "repositoryId": repository_id,
    "resourceFinalizationHeadDigest": canonical_digest(sequence_two_receipt),
    "resourceFinalizationHeadPath": sequence_two_path,
    "resourceFinalizationSequence": 2,
    "state": "closed",
    "structuredDecision": {
        "actor": "human:fixture-owner",
        "authoritySource": "fixture",
        "decidedAt": "2026-01-01T00:00:00Z",
        "decision": "approved",
        "evidenceRefs": [evidence_path, sequence_two_path],
        "reason": "fixture close",
    },
    "workItemId": work_item,
}
write_json(decision_dir / f"{work_item}.close.json", close)
write_json(root / ".ai/project.json", {"repositoryId": repository_id})

languages = (("", "fixture", "In progress → Implemented after verified close"), (".zh-CN", "fixture zh", "进行中 → 验证关闭后已实现"), (".ja", "fixture ja", "In progress → Implemented after verified close"))
for suffix, title, status in languages:
    (docs_dir / f"{work_item}{suffix}.md").write_text(
        "---\n"
        "author: fixture\n"
        f"title: {title}\n"
        f"workItemId: {work_item}\n"
        "status: in_progress\n"
        f"lastVerifiedBy: {work_item}\n"
        "authority: canonical\n"
        "---\n\n"
        "# Contract-language fixture\n\n"
        "This pre-archive planning sentence must remain byte-for-byte unchanged.\n",
        encoding="utf-8",
    )
    parity_suffix = suffix
    link_suffix = suffix
    (parity_dir / f"reference-parity{parity_suffix}.md").write_text(
        f"| WI-999 — {title} | {status} | [Work Item](../work-items/{work_item}{link_suffix}.md); future lifecycle paths. |\n",
        encoding="utf-8",
    )
PY

cp -R "$fixture" "$tmp/unpromoted"

if python3 "$helper" --repo "$fixture" --work-item WI-999-closed-docs-fixture --check \
  >"$tmp/precheck.out" 2>"$tmp/precheck.err"; then
  echo 'promotion check accepted stale pre-close documentation' >&2
  exit 1
fi
grep -Fq 'promotion required' "$tmp/precheck.err"

python3 "$helper" --repo "$fixture" --work-item WI-999-closed-docs-fixture
python3 "$helper" --repo "$fixture" --work-item WI-999-closed-docs-fixture --check
before=$(find "$fixture/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)
python3 "$helper" --repo "$fixture" --work-item WI-999-closed-docs-fixture
after=$(find "$fixture/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256)
test "$before" = "$after"

for document in "$fixture"/docs/work-items/WI-999-closed-docs-fixture*.md; do
  grep -Fq 'status: implemented' "$document"
  grep -Fq 'terminalArchive: .ai/work-items/archive/WI-999-closed-docs-fixture.contract.json' "$document"
  grep -Fq 'terminalVerification: .ai/evidence/WI-999-closed-docs-fixture.verification.json' "$document"
  grep -Fq 'terminalDecision: .ai/decisions/WI-999-closed-docs-fixture.close.json' "$document"
  grep -Fq 'This pre-archive planning sentence must remain byte-for-byte unchanged.' "$document"
done
python3 "$helper" --repo "$fixture" --check-all

# A Runtime-generated canonical receipt may already bind the merged PR and
# exact cleanup in one terminal observation. It has no transition children,
# so promotion must accept sequence 0 without weakening the strict two-step
# chain required for non-terminal roots.
cp -R "$tmp/unpromoted" "$tmp/direct-terminal"
python3 - "$tmp/direct-terminal" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
decision_dir = root / ".ai/decisions"

def canonical_digest(value: object) -> str:
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    return "sha256:" + hashlib.sha256(encoded).hexdigest()

def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

root_path = decision_dir / f"{work_item}.finalize.json"
receipt = json.loads(root_path.read_text(encoding="utf-8"))
receipt["before"] = {"branch": "present", "pullRequest": "merged", "worktree": "clean"}
receipt["after"] = {"branch": "deleted", "pullRequest": "merged", "worktree": "removed"}
receipt["pullRequest"]["mergeCommit"] = "f" * 40
receipt["result"] = {"disposition": "deleted", "failureCodes": [], "unknownCodes": []}
write_json(root_path, receipt)
for path in decision_dir.glob(f"{work_item}.finalize.*.json"):
    path.unlink()

close_path = decision_dir / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
finalization_path = f".ai/decisions/{work_item}.finalize.json"
close["resourceFinalizationSequence"] = 0
close["resourceFinalizationHeadPath"] = finalization_path
close["resourceFinalizationHeadDigest"] = canonical_digest(receipt)
close["structuredDecision"]["evidenceRefs"] = [
    ".ai/evidence/" + work_item + ".verification.json",
    finalization_path,
]
write_json(close_path, close)
PY
if python3 "$helper" --repo "$tmp/direct-terminal" --work-item WI-999-closed-docs-fixture --check \
  >"$tmp/direct-terminal.out" 2>"$tmp/direct-terminal.err"; then
  echo 'promotion check accepted stale direct-terminal documentation' >&2
  exit 1
fi
grep -Fq 'promotion required' "$tmp/direct-terminal.err"
python3 "$helper" --repo "$tmp/direct-terminal" --work-item WI-999-closed-docs-fixture
python3 "$helper" --repo "$tmp/direct-terminal" --work-item WI-999-closed-docs-fixture --check

# A legacy Runtime could close after a retained merged root.  The current
# Runtime rejects that order, but a later append-only deleted transition is a
# valid bounded reconciliation and must be promotable without rewriting the
# original close binding.
cp -R "$tmp/unpromoted" "$tmp/post-close-reconciliation"
python3 - "$tmp/post-close-reconciliation" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
repository_id = json.loads((root / ".ai/project.json").read_text())["repositoryId"]
decision_dir = root / ".ai/decisions"

def canonical_digest(value: object) -> str:
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    return "sha256:" + hashlib.sha256(encoded).hexdigest()

def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

for path in decision_dir.glob(f"{work_item}.finalize.*.json"):
    path.unlink()
root_path = decision_dir / f"{work_item}.finalize.json"
retained = json.loads(root_path.read_text(encoding="utf-8"))
retained["before"] = {"branch": "present", "pullRequest": "merged", "worktree": "clean"}
retained["after"] = {"branch": "present", "pullRequest": "merged", "worktree": "clean"}
retained["pullRequest"]["mergeCommit"] = "f" * 40
retained["result"] = {"disposition": "retained", "failureCodes": [], "unknownCodes": []}
write_json(root_path, retained)
root_digest = canonical_digest(retained)

close_path = decision_dir / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
root_relative = f".ai/decisions/{work_item}.finalize.json"
close["resourceFinalizationSequence"] = 0
close["resourceFinalizationHeadPath"] = root_relative
close["resourceFinalizationHeadDigest"] = root_digest
close["structuredDecision"]["evidenceRefs"] = [
    f".ai/evidence/{work_item}.verification.json",
    root_relative,
]
write_json(close_path, close)

deleted = json.loads(json.dumps(retained))
deleted["receiptId"] = "legacy-reconciliation"
deleted["operationId"] = "legacy-reconciliation-operation"
deleted["before"] = retained["after"]
deleted["after"] = {"branch": "deleted", "pullRequest": "merged", "worktree": "removed"}
deleted["result"] = {"disposition": "deleted", "failureCodes": [], "unknownCodes": []}
transition = {
    "schemaVersion": 1,
    "transitionId": "legacy-cleanup-reconciliation",
    "sequence": 1,
    "predecessorReceiptDigest": root_digest,
    "receipt": deleted,
}
transition_path = decision_dir / f"{work_item}.finalize.{canonical_digest(transition).removeprefix('sha256:')}.json"
write_json(transition_path, transition)
PY
if python3 "$helper" --repo "$tmp/post-close-reconciliation" --work-item WI-999-closed-docs-fixture --check \
  >"$tmp/post-close-reconciliation.out" 2>"$tmp/post-close-reconciliation.err"; then
  echo 'promotion check accepted stale post-close reconciliation documentation' >&2
  exit 1
fi
grep -Fq 'promotion required' "$tmp/post-close-reconciliation.err"
python3 "$helper" --repo "$tmp/post-close-reconciliation" --work-item WI-999-closed-docs-fixture
python3 "$helper" --repo "$tmp/post-close-reconciliation" --work-item WI-999-closed-docs-fixture --check

# A valid recovery receipt makes an immutable predecessor historical rather
# than normally promotable.  An invalid/non-canonical close may remain for
# audit purposes, but check-all must skip that predecessor and never demand an
# invented approved close.
cp -R "$tmp/unpromoted" "$tmp/recovered-predecessor"
python3 - "$tmp/recovered-predecessor" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
repository_id = json.loads((root / ".ai/project.json").read_text())[
    "repositoryId"
]
(root / ".ai/decisions" / f"{work_item}.close.json").write_text(
    json.dumps(
        {
            "workItemId": work_item,
            "repositoryId": repository_id,
            "state": "closed",
            "decisionState": "confirmed",
            "humanDecision": "historical descriptive close",
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
(root / ".ai/decisions" / f"{work_item}.recovery.json").write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "workItemId": work_item,
            "predecessorWorkItemId": work_item,
            "successorWorkItemId": "WI-1000-successor",
            "decision": "successor",
            "repositoryId": repository_id,
            "reason": "The predecessor close is preserved as immutable history.",
            "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
PY
python3 "$helper" --repo "$tmp/recovered-predecessor" --check-all

# A retry receipt may remain the canonical recovery record while a later,
# digest-addressed supersede receipt records the immutable predecessor
# boundary.  Promotion must discover the valid hashed receipt rather than
# treating the predecessor as a normal approved close.
cp -R "$tmp/unpromoted" "$tmp/retry-then-superseded"
python3 - "$tmp/retry-then-superseded" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
repository_id = json.loads((root / ".ai/project.json").read_text())["repositoryId"]

def canonical_digest(value: object) -> str:
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    return "sha256:" + hashlib.sha256(encoded).hexdigest()

(root / ".ai/decisions" / f"{work_item}.close.json").write_text(
    json.dumps(
        {
            "workItemId": work_item,
            "repositoryId": repository_id,
            "state": "closed",
            "decisionState": "confirmed",
            "humanDecision": "historical descriptive close",
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
(root / ".ai/decisions" / f"{work_item}.recovery.json").write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "workItemId": work_item,
            "predecessorWorkItemId": work_item,
            "decision": "retry",
            "repositoryId": repository_id,
            "reason": "A failed delivery was retried before the predecessor was superseded.",
            "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
supersede = {
    "schemaVersion": 1,
    "workItemId": work_item,
    "predecessorWorkItemId": work_item,
    "successorWorkItemId": "WI-1000-successor",
    "decision": "supersede",
    "repositoryId": repository_id,
    "reason": "The predecessor close is preserved as immutable history.",
    "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
}
suffix = canonical_digest(supersede).removeprefix("sha256:")
(root / ".ai/decisions" / f"{work_item}.recovery.{suffix}.json").write_text(
    json.dumps(supersede, indent=2) + "\n", encoding="utf-8"
)
PY
python3 "$helper" --repo "$tmp/retry-then-superseded" --check-all

cp -R "$tmp/unpromoted" "$tmp/invalid-recovery"
python3 - "$tmp/invalid-recovery" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
repository_id = json.loads((root / ".ai/project.json").read_text())[
    "repositoryId"
]
(root / ".ai/decisions" / f"{work_item}.recovery.json").write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "workItemId": work_item,
            "predecessorWorkItemId": work_item,
            "successorWorkItemId": "WI-1000-successor",
            "decision": "successor",
            "repositoryId": "sha256:" + "f" * 64,
            "reason": "foreign recovery must not suppress validation",
            "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
PY
if python3 "$helper" --repo "$tmp/invalid-recovery" --check-all >"$tmp/invalid-recovery.out" 2>"$tmp/invalid-recovery.err"; then
  echo 'promotion accepted invalid recovery receipt' >&2
  exit 1
fi
grep -Fq 'recovery' "$tmp/invalid-recovery.err"

docs_digest() {
  find "$1/docs" -type f -print0 | sort -z | xargs -0 shasum -a 256
}

expect_no_write_failure() {
  label=$1
  target=$2
  expected=$3
  before_failure=$(docs_digest "$target")
  if python3 "$helper" --repo "$target" --work-item WI-999-closed-docs-fixture \
    >"$tmp/$label.out" 2>"$tmp/$label.err"; then
    echo "promotion accepted invalid fixture: $label" >&2
    exit 1
  fi
  grep -Fq "$expected" "$tmp/$label.err"
  after_failure=$(docs_digest "$target")
  test "$before_failure" = "$after_failure"
}

cp -R "$tmp/unpromoted" "$tmp/wrong-repository"
perl -0pi -e 's/sha256:eeee/sha256:ffff/' "$tmp/wrong-repository/.ai/project.json"
expect_no_write_failure wrong-repository "$tmp/wrong-repository" 'repository identity mismatch'

cp -R "$tmp/unpromoted" "$tmp/wrong-work-item"
perl -0pi -e 's/WI-999-closed-docs-fixture/WI-998-foreign-fixture/' \
  "$tmp/wrong-work-item/.ai/evidence/WI-999-closed-docs-fixture.verification.json"
expect_no_write_failure wrong-work-item "$tmp/wrong-work-item" 'verification receipt identity or result mismatch'

cp -R "$tmp/unpromoted" "$tmp/contract-digest"
perl -0pi -e 's/"contractDigest": "sha256:[^"]+"/"contractDigest": "sha256:bad"/' \
  "$tmp/contract-digest/.ai/work-items/archive/WI-999-closed-docs-fixture.archive.json"
expect_no_write_failure contract-digest "$tmp/contract-digest" 'archive Contract digest mismatch'

cp -R "$tmp/unpromoted" "$tmp/close-head"
perl -0pi -e 's/"resourceFinalizationHeadDigest": "sha256:[^"]+"/"resourceFinalizationHeadDigest": "sha256:bad"/' \
  "$tmp/close-head/.ai/decisions/WI-999-closed-docs-fixture.close.json"
expect_no_write_failure close-head "$tmp/close-head" 'close finalization head digest mismatch'

cp -R "$tmp/unpromoted" "$tmp/nondeleted"
sequence_two=
while IFS= read -r candidate; do
  if test "$(jq -r .sequence "$candidate")" = 2; then
    sequence_two=$candidate
    break
  fi
done < <(find "$tmp/nondeleted/.ai/decisions" \
  -name 'WI-999-closed-docs-fixture.finalize.*.json' -print)
test -n "$sequence_two"
perl -0pi -e 's/"disposition": "deleted"/"disposition": "retained"/' "$sequence_two"
expect_no_write_failure nondeleted "$tmp/nondeleted" 'finalization chain digest or filename mismatch'

cp -R "$tmp/unpromoted" "$tmp/duplicate-row"
cat "$tmp/duplicate-row/docs/reference/reference-parity.md" >> \
  "$tmp/duplicate-row/docs/reference/reference-parity.md.copy"
mv "$tmp/duplicate-row/docs/reference/reference-parity.md.copy" \
  "$tmp/duplicate-row/docs/reference/reference-parity.md"
printf '| WI-999 — duplicate | Implemented | duplicate |\n' >> \
  "$tmp/duplicate-row/docs/reference/reference-parity.md"
expect_no_write_failure duplicate-row "$tmp/duplicate-row" 'expected exactly one parity row'

cp -R "$tmp/unpromoted" "$tmp/missing-language"
find "$tmp/missing-language/docs/work-items" -name '*.ja.md' -delete
expect_no_write_failure missing-language "$tmp/missing-language" 'must be a regular non-symlink file'

cp -R "$tmp/unpromoted" "$tmp/symlink-evidence"
mv "$tmp/symlink-evidence/.ai/evidence/WI-999-closed-docs-fixture.verification.json" \
  "$tmp/symlink-evidence/.ai/evidence/real.json"
ln -s real.json "$tmp/symlink-evidence/.ai/evidence/WI-999-closed-docs-fixture.verification.json"
expect_no_write_failure symlink-evidence "$tmp/symlink-evidence" 'must be a regular non-symlink file'

cp -R "$tmp/unpromoted" "$tmp/malformed-close"
printf '{"state":' > "$tmp/malformed-close/.ai/decisions/WI-999-closed-docs-fixture.close.json"
expect_no_write_failure malformed-close "$tmp/malformed-close" 'malformed JSON'

echo 'closed Work Item promotion regression passed'
