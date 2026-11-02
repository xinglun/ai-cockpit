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

# Distinct Work Items may share a numeric prefix without being a recovery
# alias.  Full Work Item ids in parity rows must remain independently
# addressable in the documentation status projection.
full_id_fixture="$tmp/full-id-parity"
cp -R "$fixture" "$full_id_fixture"
python3 - "$full_id_fixture" "$checker" <<'PY'
import importlib.util
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
checker_path = Path(sys.argv[2])
repository_id = 'sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
first = 'WI-999-status-drift-fixture'
second = 'WI-999-independent-recovery'
archive = root / '.ai/work-items/archive'
decisions = root / '.ai/decisions'
for work_item in (first, second):
    (archive / f'{work_item}.contract.json').write_text(
        json.dumps({'workItemId': work_item, 'repositoryId': repository_id}) + '\n',
        encoding='utf-8',
    )
    (decisions / f'{work_item}.recovery.json').write_text(
        json.dumps({
            'schemaVersion': 1,
            'workItemId': work_item,
            'predecessorWorkItemId': work_item,
            'repositoryId': repository_id,
            'decision': 'successor',
            'successorWorkItemId': f'{work_item}-successor',
        }) + '\n',
        encoding='utf-8',
    )
suffixes = ('', '.zh-CN', '.ja')
for suffix in suffixes:
    source = root / 'docs/work-items' / f'{first}{suffix}.md'
    text = source.read_text(encoding='utf-8').replace(
        f'workItemId: {first}', f'workItemId: {second}'
    )
    (root / 'docs/work-items' / f'{second}{suffix}.md').write_text(
        text, encoding='utf-8'
    )
for suffix, recovered in (('', 'Recovered'), ('.zh-CN', '已恢复'), ('.ja', 'Recovered')):
    parity = root / 'docs/reference' / f'reference-parity{suffix}.md'
    lines = parity.read_text(encoding='utf-8').splitlines()
    lines[0] = lines[0].replace(f'WI-999 —', f'{first} —')
    lines.append(
        f'| {second} — independent recovery | {recovered} | `.ai/decisions/{second}.recovery.json` |'
    )
    parity.write_text('\n'.join(lines) + '\n', encoding='utf-8')

spec = importlib.util.spec_from_file_location(
    'status_consistency', checker_path
)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.path.insert(0, str(checker_path.parent))
spec.loader.exec_module(module)
rows, errors = module.parity_statuses(root)
assert not errors, errors
assert set(rows) >= {first, second}, rows
PY
python3 "$checker" --repo "$full_id_fixture"

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

# A repository-wide policy must not retroactively reinterpret a closed
# Contract created before the policy's effective boundary. It must continue
# enforcing the same conditional-parity invariant for Contracts at the
# boundary and later.
write_projection_policy() {
  local target=$1
  mkdir -p "$target/.ai/project"
  cat > "$target/.ai/project/documentation-policy.json" <<EOF
{
  "schemaVersion": 2,
  "repositoryId": "$repository_id",
  "defaultProjection": "derived",
  "effectiveFromContractCreatedAt": "2026-09-15T00:43:38Z",
  "requiredModes": [],
  "requiredOperations": [],
  "preserveExistingRegistrations": true
}
EOF
  cat > "$target/.ai/project/capabilities.json" <<EOF
{"repositoryId":"$repository_id","operationMappings":{}}
EOF
}

# Keep the language-neutral Python selector aligned with Rust's conservative
# scope relation: an ordinary source glob is disjoint from generated docs,
# while an explicit Work Item docs scope still selects the projection.
python3 - "$root/tests/docs" <<'PY'
import sys

sys.path.insert(0, sys.argv[1])
from work_item_projection_policy import ProjectionPolicy, requires_documentation_projection

policy = ProjectionPolicy(
    repository_id="sha256:fixture",
    default_projection="derived",
    required_modes=("docs", "documentation", "release"),
    required_operations=("documentation.modify", "release.publish"),
    preserve_existing_registrations=False,
)
ordinary_code = {"mode": "code", "scope": ["src/**/*.rs"]}
documentation = {"mode": "code", "scope": ["docs/work-items/**"]}
assert not requires_documentation_projection(
    ordinary_code,
    "WI-1000-source-glob",
    policy,
    has_existing_registration=False,
), "ordinary Rust source glob must not require generated documentation"
assert requires_documentation_projection(
    documentation,
    "WI-1001-docs-glob",
    policy,
    has_existing_registration=False,
), "explicit documentation scope must retain the projection requirement"
PY

historical_conditional="$tmp/historical-conditional"
cp -R "$fixture" "$historical_conditional"
write_projection_policy "$historical_conditional"
python3 - "$historical_conditional" "$work_item" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = sys.argv[2]
path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(path.read_text(encoding="utf-8"))
contract["createdAt"] = "2026-09-15T00:43:37Z"
path.write_text(json.dumps(contract) + "\n", encoding="utf-8")
PY
python3 "$checker" --repo "$historical_conditional"

current_conditional="$tmp/current-conditional"
cp -R "$fixture" "$current_conditional"
write_projection_policy "$current_conditional"
python3 - "$current_conditional" "$work_item" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = sys.argv[2]
path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(path.read_text(encoding="utf-8"))
contract["createdAt"] = "2026-09-15T00:43:38Z"
path.write_text(json.dumps(contract) + "\n", encoding="utf-8")
PY
if python3 "$checker" --repo "$current_conditional" \
  >"$tmp/current-conditional.out" 2>"$tmp/current-conditional.err"; then
  echo 'status consistency accepted current conditional parity for a closed Work Item' >&2
  exit 1
fi
grep -Fq 'terminal Work Item retains conditional parity status' "$tmp/current-conditional.err"

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
    old_status = "已实现" if suffix == ".zh-CN" else "Implemented"
    text = parity.read_text(encoding="utf-8")
    needle = f"fixture | {old_status}"
    assert text.count(needle) == 1, (suffix, text)
    text = text.replace(needle, f"fixture | {status}", 1)
    parity.write_text(text, encoding="utf-8")
PY
python3 "$checker" --repo "$digest_recovery"

# A conditional predecessor row is terminal only when a digest-bound
# supersession points to a fully archived successor, its confirmed close, and
# a passing verification receipt bound through the successor Summary.
terminal_supersede="$tmp/terminal-supersede"
cp -R "$fixture" "$terminal_supersede"
python3 - "$terminal_supersede" <<'PY'
import hashlib
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
repository_id = "sha256:" + "e" * 64
predecessor = "WI-999-status-drift-fixture"
successor = "WI-1000-terminal-successor"
snapshot = "sha256:" + "a" * 64
archive_dir = root / ".ai/work-items/archive"
decision_dir = root / ".ai/decisions"
evidence_dir = root / ".ai/evidence"
evidence_dir.mkdir(parents=True, exist_ok=True)

def raw_digest(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()

def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")

def canonical_digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(
        json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    ).hexdigest()

predecessor_contract_path = archive_dir / f"{predecessor}.contract.json"
predecessor_contract = json.loads(predecessor_contract_path.read_text(encoding="utf-8"))
predecessor_contract["scope"] = ["crates/example/src/lib.rs"]
write_json(predecessor_contract_path, predecessor_contract)
predecessor_summary_path = archive_dir / f"{predecessor}.summary.json"
predecessor_events_path = archive_dir / f"{predecessor}.events.jsonl"
predecessor_outcome_path = archive_dir / f"{predecessor}.outcome.json"
write_json(predecessor_summary_path, {"workItemId": predecessor})
predecessor_events_path.write_text('{"workItemId":"' + predecessor + '"}\n', encoding="utf-8")
write_json(predecessor_outcome_path, {"workItemId": predecessor, "state": "recovered"})

for path in decision_dir.glob(f"{predecessor}.recovery*.json"):
    path.unlink()
predecessor_archive = archive_dir / f"{predecessor}.archive.json"
predecessor_files = {
    "contractPath": f".ai/work-items/archive/{predecessor}.contract.json",
    "contractDigest": raw_digest(predecessor_contract_path),
    "summaryPath": f".ai/work-items/archive/{predecessor}.summary.json",
    "summaryDigest": raw_digest(predecessor_summary_path),
    "eventsPath": f".ai/work-items/archive/{predecessor}.events.jsonl",
    "eventsDigest": raw_digest(predecessor_events_path),
    "outcomePath": f".ai/work-items/archive/{predecessor}.outcome.json",
    "outcomeDigest": raw_digest(predecessor_outcome_path),
}
write_json(predecessor_archive, {
    "workItemId": predecessor,
    "state": "archived",
    "files": predecessor_files,
})

verification_path = f".ai/evidence/{successor}.verification.json"
close_path = f".ai/decisions/{successor}.close.json"
recovery = {
    "schemaVersion": 1,
    "workItemId": predecessor,
    "predecessorWorkItemId": predecessor,
    "successorWorkItemId": successor,
    "repositoryId": repository_id,
    "decision": "supersede",
    "reason": "preserve immutable predecessor and bind terminal successor",
    "predecessorArchiveManifestDigest": raw_digest(predecessor_archive),
    "predecessorContractDigest": canonical_digest(predecessor_contract),
    "predecessorSummaryDigest": canonical_digest(json.loads(predecessor_summary_path.read_text(encoding="utf-8"))),
    "predecessorEventsDigest": predecessor_files["eventsDigest"],
    "predecessorOutcomeDigest": canonical_digest(json.loads(predecessor_outcome_path.read_text(encoding="utf-8"))),
    "evidenceRefs": [
        f".ai/work-items/archive/{predecessor}.archive.json",
        close_path,
        verification_path,
    ],
}
recovery_name = f"{predecessor}.recovery.{canonical_digest(recovery).removeprefix('sha256:')}.json"
recovery_relative = f".ai/decisions/{recovery_name}"
recovery_path = decision_dir / recovery_name

contract = {
    "workItemId": successor,
    "repositoryId": repository_id,
    "predecessorWorkItemId": predecessor,
    "recoveryDecisionPath": recovery_relative,
    "baseRevision": "1" * 40,
}
contract_path = archive_dir / f"{successor}.contract.json"
write_json(contract_path, contract)
contract_digest = raw_digest(contract_path)
summary = {
    "workItemId": successor,
    "repositoryId": repository_id,
    "checkpointContractDigest": contract_digest,
    "preflightContractDigest": contract_digest,
    "checkpointRepositorySnapshotDigest": snapshot,
    "preflightRepositorySnapshotDigest": snapshot,
}
summary_path = archive_dir / f"{successor}.summary.json"
write_json(summary_path, summary)
verification = {
    "workItemId": successor,
    "repositoryId": repository_id,
    "contractDigest": contract_digest,
    "repositorySnapshotDigest": snapshot,
    "passed": True,
    "receipt": {
        "passed": True,
        "repositoryId": repository_id,
        "workItemId": successor,
        "planReceipt": {
            "passed": True,
            "repositoryId": repository_id,
            "workItemId": successor,
            "repositorySnapshotDigest": snapshot,
        },
    },
}
verification["receiptDigest"] = canonical_digest(verification["receipt"])
write_json(root / verification_path, verification)
outcome_path = archive_dir / f"{successor}.outcome.json"
write_json(outcome_path, {"workItemId": successor, "state": "finish_ready"})
archive_manifest = {
    "workItemId": successor,
    "state": "archived",
    "files": {
        "contractPath": f".ai/work-items/archive/{successor}.contract.json",
        "contractDigest": contract_digest,
        "summaryPath": f".ai/work-items/archive/{successor}.summary.json",
        "summaryDigest": raw_digest(summary_path),
        "outcomePath": f".ai/work-items/archive/{successor}.outcome.json",
        "outcomeDigest": raw_digest(outcome_path),
    },
}
write_json(archive_dir / f"{successor}.archive.json", archive_manifest)

close = {
    "workItemId": successor,
    "repositoryId": repository_id,
    "state": "closed",
    "decisionState": "confirmed",
    "humanDecision": "approved",
    "structuredDecision": {
        "decision": "approved",
        "actor": "fixture-human",
        "authoritySource": "fixture-policy",
        "reason": "terminal successor evidence",
        "decidedAt": "2026-09-15T00:00:00Z",
        "evidenceRefs": [verification_path],
    },
    "finalReport": {
        "status": "verified",
        "humanStatusColor": "green",
        "bindings": {
            "workItemId": successor,
            "repositoryId": repository_id,
            "repositorySnapshotDigest": snapshot,
            "evidenceRefs": [verification_path],
        }
    },
}
close["finalReportDigest"] = canonical_digest(close["finalReport"])
write_json(decision_dir / f"{successor}.close.json", close)

write_json(recovery_path, recovery)

for suffix, status in (
    ("", "In progress → Implemented after verified close"),
    (".zh-CN", "进行中 → 验证关闭后已实现"),
    (".ja", "In progress → verified close 後 Implemented"),
):
    path = root / "docs/reference" / f"reference-parity{suffix}.md"
    lines = path.read_text(encoding="utf-8").splitlines()
    for index, line in enumerate(lines):
        if line.startswith("| WI-999 —"):
            cells = [cell.strip() for cell in line.split("|")[1:-1]]
            cells[1] = status
            lines[index] = "| " + " | ".join(cells) + " |"
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
python3 "$checker" --repo "$terminal_supersede"

# A terminal successor may itself require one bounded recovery.  The
# documentation gate must follow that identity-continuous lineage instead of
# treating the first, still-unclosed successor as a permanent conditional row.
two_hop="$tmp/two-hop-terminal"
cp -R "$terminal_supersede" "$two_hop"
python3 - "$two_hop" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
repo_id = "sha256:fixture"
predecessor = "WI-999-status-drift-fixture"
middle = "WI-1000-terminal-successor"
terminal = "WI-1001-terminal-successor"
archive = root / ".ai/work-items/archive"
decisions = root / ".ai/decisions"
evidence = root / ".ai/evidence"

def write(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")

def canonical(value):
    return "sha256:" + hashlib.sha256(
        json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    ).hexdigest()

def raw(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()

def refresh_manifest(work_item):
    contract = archive / f"{work_item}.contract.json"
    summary = archive / f"{work_item}.summary.json"
    outcome = archive / f"{work_item}.outcome.json"
    write(archive / f"{work_item}.archive.json", {
        "workItemId": work_item,
        "state": "archived",
        "files": {
            "contractPath": f".ai/work-items/archive/{work_item}.contract.json",
            "contractDigest": raw(contract),
            "summaryPath": f".ai/work-items/archive/{work_item}.summary.json",
            "summaryDigest": raw(summary),
            "outcomePath": f".ai/work-items/archive/{work_item}.outcome.json",
            "outcomeDigest": raw(outcome),
        },
    })

# Replace the direct root supersession with the first edge of a two-hop chain.
for path in decisions.glob(f"{predecessor}.recovery*.json"):
    path.unlink()
root_contract = json.loads((archive / f"{predecessor}.contract.json").read_text())
root_summary = json.loads((archive / f"{predecessor}.summary.json").read_text())
root_outcome = json.loads((archive / f"{predecessor}.outcome.json").read_text())
root_events = archive / f"{predecessor}.events.jsonl"
root_manifest = archive / f"{predecessor}.archive.json"
root_recovery = {
    "schemaVersion": 1,
    "workItemId": predecessor,
    "predecessorWorkItemId": predecessor,
    "successorWorkItemId": middle,
    "repositoryId": repo_id,
    "decision": "successor",
    "reason": "bind the first archived recovery successor",
    "predecessorContractDigest": canonical(root_contract),
    "predecessorSummaryDigest": canonical(root_summary),
    "predecessorEventsDigest": raw(root_events),
    "predecessorOutcomeDigest": canonical(root_outcome),
    "evidenceRefs": [f".ai/work-items/archive/{predecessor}.archive.json"],
}
root_name = f"{predecessor}.recovery.{canonical(root_recovery).removeprefix('sha256:')}.json"
root_recovery_path = decisions / root_name
write(root_recovery_path, root_recovery)

# The previous terminal becomes an archived intermediate node without a close.
(decisions / f"{middle}.close.json").unlink()
middle_contract_path = archive / f"{middle}.contract.json"
middle_contract = json.loads(middle_contract_path.read_text())
middle_contract["predecessorWorkItemId"] = predecessor
middle_contract["recoveryDecisionPath"] = f".ai/decisions/{root_name}"
write(middle_contract_path, middle_contract)
refresh_manifest(middle)

# Build the verified, closed terminal successor from the intermediate fixture.
middle_summary = json.loads((archive / f"{middle}.summary.json").read_text())
middle_outcome = json.loads((archive / f"{middle}.outcome.json").read_text())
middle_manifest = archive / f"{middle}.archive.json"
middle_recovery = {
    "schemaVersion": 1,
    "workItemId": middle,
    "predecessorWorkItemId": middle,
    "successorWorkItemId": terminal,
    "repositoryId": repo_id,
    "decision": "successor",
    "reason": "bind the terminal recovery successor",
    "predecessorContractDigest": canonical(middle_contract),
    "predecessorSummaryDigest": canonical(middle_summary),
    "predecessorOutcomeDigest": canonical(middle_outcome),
    "evidenceRefs": [f".ai/work-items/archive/{middle}.archive.json"],
}
middle_name = f"{middle}.recovery.{canonical(middle_recovery).removeprefix('sha256:')}.json"
middle_recovery_path = decisions / middle_name
write(middle_recovery_path, middle_recovery)

terminal_contract = dict(middle_contract)
terminal_contract.update({
    "workItemId": terminal,
    "predecessorWorkItemId": middle,
    "recoveryDecisionPath": f".ai/decisions/{middle_name}",
})
write(archive / f"{terminal}.contract.json", terminal_contract)
terminal_summary = dict(middle_summary)
terminal_summary["workItemId"] = terminal
write(archive / f"{terminal}.summary.json", terminal_summary)
terminal_outcome = dict(middle_outcome)
terminal_outcome["workItemId"] = terminal
write(archive / f"{terminal}.outcome.json", terminal_outcome)
refresh_manifest(terminal)

verification = json.loads((evidence / f"{middle}.verification.json").read_text())
verification["workItemId"] = terminal
verification["receipt"]["workItemId"] = terminal
verification["receipt"]["planReceipt"]["workItemId"] = terminal
verification["receiptDigest"] = canonical(verification["receipt"])
write(evidence / f"{terminal}.verification.json", verification)

close = {
    "workItemId": terminal,
    "repositoryId": repo_id,
    "state": "closed",
    "decisionState": "confirmed",
    "humanDecision": "approved",
    "structuredDecision": {
        "decision": "approved",
        "actor": "fixture-human",
        "authoritySource": "fixture-policy",
        "reason": "terminal successor evidence",
        "decidedAt": "2026-09-15T00:00:00Z",
        "evidenceRefs": [f".ai/evidence/{terminal}.verification.json"],
    },
    "finalReport": {
        "status": "verified",
        "humanStatusColor": "green",
        "bindings": {
            "workItemId": terminal,
            "repositoryId": repo_id,
            "repositorySnapshotDigest": verification["repositorySnapshotDigest"],
            "evidenceRefs": [f".ai/evidence/{terminal}.verification.json"],
        },
    },
}
close["finalReportDigest"] = canonical(close["finalReport"])
write(decisions / f"{terminal}.close.json", close)
PY
if python3 "$checker" --repo "$two_hop" >"$tmp/two-hop-terminal.out" \
  2>"$tmp/two-hop-terminal.err"; then
  echo 'status consistency treated a successor lineage as terminal supersession' >&2
  exit 1
fi
grep -Fq 'terminal Work Item retains conditional parity status' "$tmp/two-hop-terminal.err"

# A bound successor that remains open is a continuation, not a terminal
# supersession. Its conditional parity row must remain truthful while the
# successor is still in progress.
pending_successor="$tmp/pending-successor"
cp -R "$terminal_supersede" "$pending_successor"
python3 - "$pending_successor" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-status-drift-fixture"
successor = "WI-1000-terminal-successor"
decisions = root / ".ai/decisions"
old_path = next(decisions.glob(f"{work_item}.recovery.*.json"))
recovery = json.loads(old_path.read_text(encoding="utf-8"))
recovery["decision"] = "successor"
encoded = json.dumps(
    recovery, ensure_ascii=False, separators=(",", ":"), sort_keys=True
).encode("utf-8")
new_path = decisions / f"{work_item}.recovery.{hashlib.sha256(encoded).hexdigest()}.json"
old_path.unlink()
new_path.write_text(json.dumps(recovery, indent=2) + "\n", encoding="utf-8")
relative = new_path.relative_to(root).as_posix()
for name in ("contract", "summary"):
    path = root / ".ai/work-items/archive" / f"{successor}.{name}.json"
    value = json.loads(path.read_text(encoding="utf-8"))
    value["predecessorWorkItemId"] = work_item
    value["predecessorContractDigest"] = recovery["predecessorContractDigest"]
    value["recoveryDecisionPath"] = relative
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
(decisions / f"{work_item}.close.json").unlink()
(decisions / f"{successor}.close.json").unlink()
PY
python3 "$checker" --repo "$pending_successor"

cp -R "$pending_successor" "$tmp/pending-successor-foreign-binding"
python3 - "$tmp/pending-successor-foreign-binding" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
path = root / ".ai/work-items/archive/WI-1000-terminal-successor.summary.json"
value = json.loads(path.read_text(encoding="utf-8"))
value["predecessorContractDigest"] = "sha256:" + "f" * 64
path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
PY

expect_conditional_failure() {
  local name=$1
  if python3 "$checker" --repo "$tmp/$name" >"$tmp/$name.out" 2>"$tmp/$name.err"; then
    echo "status consistency accepted invalid terminal supersede: $name" >&2
    exit 1
  fi
  grep -Fq 'terminal Work Item retains conditional parity status' "$tmp/$name.err"
}

if python3 "$checker" --repo "$tmp/pending-successor-foreign-binding" \
  >"$tmp/pending-successor-foreign-binding.out" \
  2>"$tmp/pending-successor-foreign-binding.err"; then
  echo 'status consistency accepted a mismatched pending successor binding' >&2
  exit 1
fi
grep -Fq 'terminal Work Item retains conditional parity status' \
  "$tmp/pending-successor-foreign-binding.err"

cp -R "$terminal_supersede" "$tmp/incomplete-successor"
rm "$tmp/incomplete-successor/.ai/evidence/WI-1000-terminal-successor.verification.json"
expect_conditional_failure incomplete-successor

cp -R "$terminal_supersede" "$tmp/tampered-supersede-digest"
python3 - "$tmp/tampered-supersede-digest" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
decisions = root / ".ai/decisions"
path = next(decisions.glob("WI-999-status-drift-fixture.recovery.*.json"))
value = json.loads(path.read_text(encoding="utf-8"))
value["reason"] = "edited without changing the immutable digest filename"
path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
PY
expect_conditional_failure tampered-supersede-digest

cp -R "$terminal_supersede" "$tmp/foreign-successor-close"
python3 - "$tmp/foreign-successor-close" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
path = root / ".ai/decisions/WI-1000-terminal-successor.close.json"
value = json.loads(path.read_text(encoding="utf-8"))
value["repositoryId"] = "sha256:" + "f" * 64
path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
PY
expect_conditional_failure foreign-successor-close

python3 "$checker" --repo "$root"


# Ordinary approved close must not be rewritten as superseded to continue.
# This is a normalized protocol fixture, not proof that a forged issuer is trusted.
python3 - "$checker" "$tmp/ordinary-native-shaped-lineage" <<'PYTEST'
from pathlib import Path
import copy, hashlib, importlib.util, json, os, subprocess, sys
path=Path(sys.argv[1]);sys.path.insert(0,str(path.parent))
spec=importlib.util.spec_from_file_location("status_checker_under_test",path)
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
root=Path(sys.argv[2]);root.mkdir();rid="sha256:"+"e"*64
p="WI-1201-ordinary-predecessor-v9-9-9";s="WI-1202-doc-projection-v9-9-9";runtime="sha256:"+"a"*64;snapshot="sha256:"+"b"*64
canonical=module.canonical_json_digest

def put(relative,value):
 target=root/relative;target.parent.mkdir(parents=True,exist_ok=True)
 target.write_text(json.dumps(value,ensure_ascii=False,indent=2)+"\n")
 return target

def native_archive(wi, contract, summary):
 records={"contract":contract,"summary":summary,"outcome":{"workItemId":wi,"decisionState":"green","verification":{"status":"verified"}}}
 files={}
 for kind,value in records.items():
  rel=f".ai/work-items/archive/{wi}.{kind}.json";target=put(rel,value);files[kind+"Path"]=rel;files[kind+"Digest"]=module.raw_file_digest(target)
 rel=f".ai/work-items/archive/{wi}.events.jsonl";target=root/rel;target.write_text("{}\n");files["eventsPath"]=rel;files["eventsDigest"]=module.raw_file_digest(target)
 manifest={"state":"archived","workItemId":wi,"repositoryId":rid,"files":files,"runtimeDigest":runtime}
 put(f".ai/work-items/archive/{wi}.archive.json",manifest)
 return manifest

def native_verification_close(wi, contract):
 relative=f".ai/evidence/{wi}.verification.json"
 receipt={"workItemId":wi,"repositoryId":rid,"passed":True,"runtimeDigest":runtime,
          "planReceipt":{"workItemId":wi,"repositoryId":rid,"repositorySnapshotDigest":snapshot}}
 verification={"evidenceSchemaVersion":1,"workItemId":wi,"repositoryId":rid,"passed":True,"contractDigest":canonical(contract),"repositorySnapshotDigest":snapshot,"runtimeDigest":runtime,"receipt":receipt,"receiptDigest":canonical(receipt)}
 put(relative,verification)
 report={"workItemId":wi,"status":"verified","humanStatusColor":"green","bindings":{"workItemId":wi,"repositoryId":rid,"repositorySnapshotDigest":snapshot,"evidenceRefs":[relative]}}
 decision={"actor":"human:fixture-owner","authoritySource":"fixture:explicit-actual-close","reason":"fixture approved result","decidedAt":"2026-01-02T00:00:00Z","decision":"approved","evidenceRefs":[relative],"policyRefs":[]}
 close={"workItemId":wi,"repositoryId":rid,"state":"closed","decisionState":"confirmed","humanDecision":"approved","runtimeDigest":runtime,"structuredDecision":decision,"finalReport":report,"finalReportDigest":canonical(report)}
 writer="sha256:"+"c"*64
 close.update(runtimeDigest=writer,runtimeVersion="1.0.1")
 binding={"schemaVersion":1,"workItemId":wi,"repositoryId":rid,"contractDigest":canonical(contract),"archiveManifestDigest":module.raw_file_digest(root/f".ai/work-items/archive/{wi}.archive.json"),"headRevision":"d"*40,"branch":"fixture-branch","branchRef":"refs/heads/fixture-branch","capturedAt":"2026-01-02T00:00:00Z","worktreeGitDir":"/fixture/git","worktreeId":"fixture-worktree","worktreePath":"/fixture/worktree","runtimeVersion":"1.0.1","runtimeDigest":writer}
 close.update(ordinaryCleanupBinding=binding,ordinaryCleanupBindingDigest=canonical(binding))
 put(f".ai/decisions/{wi}.close.json",close)
 return verification,close

def summary(wi,contract):
 return {"workItemId":wi,"repositoryId":rid,"state":"finish_ready","checkpointCount":1,"preflightState":"green","preflightContractDigest":canonical(contract),"checkpointContractDigest":canonical(contract),"checkpointRepositorySnapshotDigest":snapshot,"preflightRepositorySnapshotDigest":snapshot}

put(".ai/project.json",{"repositoryId":rid})
put("cargo-metadata.json",{"packages":[{"name":"fixture","version":"9.9.9","source":None}]})
put(".ai/project/documentation-policy.json",{"schemaVersion":1,"repositoryId":rid,"defaultProjection":"required","requiredModes":[],"requiredOperations":[],"preserveExistingRegistrations":True})
put("docs/reference/pending-parity-registry.json",{"schemaVersion":1,"entries":[]})

def projection(phase):
 for suffix,conditional,implemented,recovered in (
  ("","In progress → Implemented after verified close","Implemented","Recovered"),
  (".zh-CN","进行中 → 验证关闭后已实现","已实现","已恢复"),
  (".ja","In progress → verified close 後 Implemented","Implemented","Recovered"),
 ):
  rows=[]
  for wi in (p,s):
   if wi==s or phase!="closed": status=conditional
   else: status=recovered
   refs=[f".ai/work-items/archive/{wi}.contract.json",f".ai/evidence/{wi}.verification.json",f".ai/decisions/{wi}.finalize.json",f".ai/decisions/{wi}.close.json"]
   if wi==p: refs.append(f".ai/decisions/{p}.recovery.json")
   rows.append("| "+wi+" — fixture | "+status+" | "+"; ".join("`"+v+"`" for v in refs)+" |")
   target=root/f"docs/work-items/{wi}{suffix}.md";target.parent.mkdir(parents=True,exist_ok=True)
   # Predecessor projection is owned by its exact docs successor. No unrelated
   # WI gains verifier status merely by having a valid Contract.
   target.write_text(f"---\nworkItemId: {wi}\nstatus: {'recovered' if wi==p and phase=='closed' else 'in_progress'}\nlastVerifiedBy: {s}\n---\n")
  target=root/f"docs/reference/reference-parity{suffix}.md";target.parent.mkdir(parents=True,exist_ok=True)
  target.write_text("| Work Item | Status | Evidence |\n| --- | --- | --- |\n"+"\n".join(rows)+"\n")

def inventory(expected, invalid=False):
 gate=path.parents[1]/"ci/governance_integrity_gate.py"
 report=root.parent/(root.name+"-inventory.json")
 env={k:v for k,v in os.environ.items() if k not in {"GITHUB_EVENT_NAME","GITHUB_REF","GITHUB_REF_NAME","GITHUB_SHA","GITHUB_EVENT_PATH","GITHUB_BASE_REF"}}
 completed=subprocess.run([sys.executable,str(gate),"--repo",str(root),"--report",str(report)],capture_output=True,text=True,env=env)
 assert completed.returncode in (0,1),(completed.returncode,completed.stdout,completed.stderr)
 document=json.loads(report.read_text())
 records={v["workItemId"]:v for v in document["inventory"]}
 assert records[p]["lifecycleState"]==expected,(records,document["findings"])
 codes=[v["code"] for v in document["findings"] if v["workItemId"]==p]
 if invalid:
  assert completed.returncode==1 and "invalid_ordinary_successor_lineage" in codes,(codes,completed)
 else:
  assert "invalid_ordinary_successor_lineage" not in codes,(codes,completed)
  if expected=="closed": assert records[p]["pendingSuccessorWorkItemId"]==s,records[p]
 # Other independent fixture-policy findings do not certify a green full gate.
 # This integration assertion specifically exercises the actual inventory route;
 # real native-producer full-gate acceptance is a separately required regression.
 return document

pc={"workItemId":p,"repositoryId":rid,"createdAt":"2026-01-01T00:00:00Z","baseRevision":"d"*40,"scope":["src/old.rs"]};ps=summary(p,pc);pm=native_archive(p,pc,ps);pv,pclose=native_verification_close(p,pc)
# State0: real predecessor close alone; no child or recovery can be invented.
recovery_path=root/f".ai/decisions/{p}.recovery.json"
assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,{}) is None
recovery={"schemaVersion":1,"decisionId":"work-item-recovery","decision":"successor","workItemId":p,"predecessorWorkItemId":p,"repositoryId":rid,"predecessorContractDigest":canonical(pc),"predecessorSummaryDigest":canonical(ps),"predecessorOutcomeDigest":canonical({"workItemId":p,"decisionState":"green","verification":{"status":"verified"}}),"predecessorEventsDigest":pm["files"]["eventsDigest"],"successorWorkItemId":s,"runtimeVersion":"1.0.1","runtimeDigest":runtime,"actor":"agent:fixture","authoritySource":"fixture:native-shaped-technical-recovery","reason":"project actual completed result","decidedAt":"2026-01-03T00:00:00Z","resumeCondition":"bounded docs and actual close required","policyRefs":[],"evidenceRefs":[f".ai/work-items/archive/{p}.archive.json",f".ai/decisions/{p}.close.json"]}
put(recovery_path.relative_to(root),recovery)
sc={"workItemId":s,"repositoryId":rid,"createdAt":"2026-01-03T00:00:00Z","baseRevision":"d"*40,"predecessorWorkItemId":p,"predecessorContractDigest":canonical(pc),"recoveryDecisionPath":recovery_path.relative_to(root).as_posix(),"scope":["docs/reference/reference-parity.md","docs/reference/reference-parity.zh-CN.md","docs/reference/reference-parity.ja.md"]+[f"docs/work-items/{wi}{suffix}.md" for wi in (p,s) for suffix in ("",".zh-CN",".ja")]};ss=summary(s,sc);ss.update({k:sc[k] for k in ("predecessorWorkItemId","predecessorContractDigest","recoveryDecisionPath")})
ac=put(f".ai/work-items/active/{s}.contract.json",sc);asu=put(f".ai/work-items/active/{s}.summary.json",ss)
# State1: ordinary approved predecessor, canonical first receipt, exact active child.
assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery)==("pending",s)
assert not (root/f".ai/decisions/{s}.close.json").exists()
assert module.ordinary_closed_projection_continuation(root,p,rid)==s
projection("pending");inventory("closed")
# Canonical receipt is a real native form; an arbitrary suffix is not an alias.
bad=root/f".ai/decisions/{p}.recovery.{('0'*64)}.json";bad.write_bytes(recovery_path.read_bytes());recovery_path.unlink()
assert module.ordinary_closed_successor_state(root,p,rid,bad,recovery) is None
bad.unlink();put(recovery_path.relative_to(root),recovery)
# Every negative restores exact native-shaped bytes; none recomputes approval to hide corruption.
for target,mutate in [
 (root/f".ai/decisions/{p}.close.json",lambda v:v.update(humanDecision="superseded")),
 (root/f".ai/decisions/{p}.close.json",lambda v:v.update(repositoryId="sha256:"+"f"*64)),
 (root/f".ai/evidence/{p}.verification.json",lambda v:v.update(receiptDigest="sha256:"+"0"*64)),
 (recovery_path,lambda v:v.update(predecessorContractDigest="sha256:"+"0"*64)),
 (recovery_path,lambda v:v.update(predecessorEventsDigest="sha256:"+"0"*64)),
 (recovery_path,lambda v:v.update(decision="supersede")),
 (recovery_path,lambda v:v.update(unrecognizedTrust=True)),
 (recovery_path,lambda v:v.update(successorBindingMode="arbitrary")),
 (root/f".ai/work-items/archive/{p}.archive.json",lambda v:v["files"].update(summaryDigest="sha256:"+"0"*64)),
 (ac,lambda v:v.update(recoveryDecisionPath=".ai/decisions/unrelated.recovery.json")),
 (asu,lambda v:v.update(predecessorWorkItemId="WI-999-foreign")),
 (asu,lambda v:v.update(state="closed")),
]:
 original=target.read_bytes();value=json.loads(original);mutate(value);target.write_text(json.dumps(value)+"\n")
 current=json.loads(recovery_path.read_bytes());assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,current) is None,(target,value)
 target.write_bytes(original)
original=(root/f".ai/decisions/{p}.close.json").read_bytes();(root/f".ai/decisions/{p}.close.json").unlink()
assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
(root/f".ai/decisions/{p}.close.json").write_bytes(original)
# A symlink and duplicate JSON key are rejected even when their resolved value looks right.
original=ac.read_bytes();ac.unlink();outside=root/"untrusted-contract.json";outside.write_bytes(original);ac.symlink_to(outside)
assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
ac.unlink();ac.write_bytes(original)
original=recovery_path.read_bytes();recovery_path.write_bytes(original.replace(b'"schemaVersion": 1',b'"schemaVersion": 1, "schemaVersion": 1',1))
assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
recovery_path.write_bytes(original)
# State1b: archived child without its close is still pending, never closed.
ac.unlink();asu.unlink();native_archive(s,sc,ss)
native_verification_close(s,sc);(root/f".ai/decisions/{s}.close.json").unlink()
assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery)==("pending",s)
projection("pending");inventory("closed")
# State2: child's own valid archive + verification + real-close-shaped record.
sv,sclose=native_verification_close(s,sc)
assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery)==("closed",s)
projection("closed");inventory("recovered")
for target,mutate in [
 (root/f".ai/decisions/{s}.close.json",lambda v:v.update(finalReportDigest="sha256:"+"0"*64)),
 (root/f".ai/decisions/{s}.close.json",lambda v:v["structuredDecision"].update(decision="rejected")),
 (root/f".ai/evidence/{s}.verification.json",lambda v:v.update(passed=False)),
 (root/f".ai/evidence/{s}.verification.json",lambda v:v.update(repositorySnapshotDigest="sha256:"+"0"*64)),
]:
 original=target.read_bytes();v=json.loads(original);mutate(v);target.write_text(json.dumps(v)+"\n")
 assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
 inventory("closure_invalid",invalid=True)
 target.write_bytes(original)
# Fixed346 explicitly requires exactly one checkpoint; do not invent future compatibility.
# Writer Runtime is deliberately different from executor Runtime in this positive fixture.
for wi in (p,s):
 close_path=root/f".ai/decisions/{wi}.close.json"
 value=json.loads(close_path.read_bytes());original=close_path.read_bytes()
 value["humanDecision"]="confirmed";value["structuredDecision"]["decision"]="confirmed";put(close_path.relative_to(root),value)
 assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery)==("closed",s)
 close_path.write_bytes(original)
for wi in (p,s):
 target=root/f".ai/decisions/{wi}.close.json"
 for mutate in (
  lambda v:v.update(ordinaryCleanupBindingDigest="sha256:"+"0"*64),
  lambda v:v["ordinaryCleanupBinding"].update(archiveManifestDigest="sha256:"+"0"*64),
  lambda v:v["ordinaryCleanupBinding"].update(contractDigest="sha256:"+"0"*64),
 ):
  original=target.read_bytes();value=json.loads(original);mutate(value);put(target.relative_to(root),value)
  assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
  target.write_bytes(original)
 original=target.read_bytes();value=json.loads(original)
 value["finalReport"]["bindings"]["repositorySnapshotDigest"]="sha256:"+"0"*64
 value["finalReportDigest"]=canonical(value["finalReport"]);put(target.relative_to(root),value)
 assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
 target.write_bytes(original)
 # Keep the old real-shaped close unchanged while replacing a self-consistent bundle.
 paths=list((root/".ai/work-items/archive").glob(wi+".*"))+[root/f".ai/evidence/{wi}.verification.json"]
 saved={path:path.read_bytes() for path in paths}
 contract=json.loads((root/f".ai/work-items/archive/{wi}.contract.json").read_bytes())
 altered=copy.deepcopy(contract);altered["goal"]="different same-WI contract"
 altered_summary=summary(wi,altered)
 if wi==s: altered_summary.update({k:sc[k] for k in ("predecessorWorkItemId","predecessorContractDigest","recoveryDecisionPath")})
 native_archive(wi,altered,altered_summary)
 formal=json.loads((root/f".ai/evidence/{wi}.verification.json").read_bytes());formal["contractDigest"]=canonical(altered);put(f".ai/evidence/{wi}.verification.json",formal)
 assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
 for path,raw in saved.items(): path.write_bytes(raw)
# Suspicious existing competing candidates must not disappear as 'missing'.
other=root/f".ai/decisions/{p}.recovery.{('f'*64)}.json"
for raw in (b'{broken',b'{"decision":"retry","decision":"retry"}'):
 other.write_bytes(raw)
 assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
 other.unlink()
# Old close retained while the child formal/summaries/archive all consistently
# move to another snapshot must still be rejected by report/cleanup binding.
saved={target:target.read_bytes() for target in root.rglob("*") if target.is_file()}
summary_path=root/f".ai/work-items/archive/{s}.summary.json"
value=json.loads(summary_path.read_bytes());new_snapshot="sha256:"+"9"*64
value.update(checkpointRepositorySnapshotDigest=new_snapshot,preflightRepositorySnapshotDigest=new_snapshot)
put(summary_path.relative_to(root),value)
manifest_path=root/f".ai/work-items/archive/{s}.archive.json"
manifest=json.loads(manifest_path.read_bytes());manifest["files"]["summaryDigest"]=module.raw_file_digest(summary_path);put(manifest_path.relative_to(root),manifest)
formal_path=root/f".ai/evidence/{s}.verification.json";formal=json.loads(formal_path.read_bytes())
formal["repositorySnapshotDigest"]=new_snapshot;formal["receipt"]["planReceipt"]["repositorySnapshotDigest"]=new_snapshot;formal["receiptDigest"]=canonical(formal["receipt"]);put(formal_path.relative_to(root),formal)
assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
inventory("closure_invalid",invalid=True)
for target,raw in saved.items(): target.write_bytes(raw)
# Fixed Runtime count==1: rewrite raw manifest consistently so count is the
# precise violated condition, not an incidental archive digest mismatch.
for count in (0,2,True,1.0):
 close_path=root/f".ai/decisions/{s}.close.json"
 originals={target:target.read_bytes() for target in (summary_path,manifest_path,close_path)}
 value=json.loads(summary_path.read_bytes());value["checkpointCount"]=count;put(summary_path.relative_to(root),value)
 manifest=json.loads(manifest_path.read_bytes());manifest["files"]["summaryDigest"]=module.raw_file_digest(summary_path);put(manifest_path.relative_to(root),manifest)
 # This is our synthetic fixture, never an actual human-close record. Repair
 # its own archive tuple so the counter's type/value is the only bad condition.
 close=json.loads(close_path.read_bytes())
 close["ordinaryCleanupBinding"]["archiveManifestDigest"]=module.raw_file_digest(manifest_path)
 close["ordinaryCleanupBindingDigest"]=canonical(close["ordinaryCleanupBinding"])
 put(close_path.relative_to(root),close)
 assert module.ordinary_cleanup_tuple_matches(root,s,rid,sc,close)
 archived=module.archived_recovery_bundle(root,s,rid)
 assert archived is not None and module.terminal_successor_is_verified_and_closed(root,s,rid,archived)
 assert module.ordinary_closed_successor_state(root,p,rid,recovery_path,recovery) is None
 for target,raw in originals.items(): target.write_bytes(raw)
# Exact ordinary conditional verifier validation is reached through check(),
# not asserted merely by calling the new helper. A foreign valid WI is negative.
projection("pending")
foreign="WI-1299-unrelated-verifier"
put(f".ai/work-items/active/{foreign}.contract.json",{"workItemId":foreign,"repositoryId":rid})
for suffix in ("",".zh-CN",".ja"):
 target=root/f"docs/work-items/{p}{suffix}.md";target.write_text(target.read_text().replace(f"lastVerifiedBy: {s}",f"lastVerifiedBy: {foreign}"))
errors=module.check(root)
assert any("not the predecessor or its exact docs successor" in error for error in errors),errors
projection("pending")
ja=root/"docs/reference/reference-parity.ja.md";raw=ja.read_bytes()
ja.write_text("\n".join(line for line in ja.read_text().splitlines() if not line.startswith("| "+p+" "))+"\n")
errors=module.check(root);assert any("missing or mixed three-language parity" in error for error in errors),errors
ja.write_bytes(raw)
ja.write_text(ja.read_text().replace("In progress → verified close 後 Implemented","Implemented",1))
errors=module.check(root);assert any("missing or mixed three-language parity" in error for error in errors),errors
ja.write_bytes(raw)
print("ordinary canonical three-state lineage draft cases passed")
PYTEST

echo 'work item status consistency regression passed'
