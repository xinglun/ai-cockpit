#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
helper="$root/tests/docs/promote_closed_work_item.py"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/promote-closed-work-item.XXXXXX")
cleanup() {
  find "$tmp" -depth -mindepth 0 -delete
}
trap cleanup EXIT

python3 - "$root/.ai/project/documentation-policy.json" <<'PY'
import json
import sys
from pathlib import Path

policy = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
assert "release" in policy["requiredModes"], policy["requiredModes"]
PY

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
    "createdAt": "2026-09-16T00:00:00Z",
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

# These synthetic handoffs exercise the real CLI consumer and local binding
# checks; their actor/runtime labels do not assert external authentication.
python3 - "$helper" "$tmp/unpromoted" "$tmp" <<'PY_HANDOFF'
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

helper, original, temporary = map(Path, sys.argv[1:])
wi = "WI-999-closed-docs-fixture"
archive = Path(".ai/work-items/archive")
decisions = Path(".ai/decisions")
binding_path = decisions / f"{wi}.resource-context.json"
contract_path = archive / f"{wi}.contract.json"
manifest_path = archive / f"{wi}.archive.json"
close_path = decisions / f"{wi}.close.json"
root_path = decisions / f"{wi}.finalize.json"


def load(root, path):
    return json.loads((root / path).read_text(encoding="utf-8"))


def write(root, path, value):
    (root / path).write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def change(root, path, **values):
    value = load(root, path)
    value.update(values)
    write(root, path, value)


def raw(root, path):
    return "sha256:" + hashlib.sha256((root / path).read_bytes()).hexdigest()


def canonical(value):
    return "sha256:" + hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def inventory(root, directory):
    return {p.relative_to(root).as_posix(): p.read_bytes()
            for p in (root / directory).rglob("*") if p.is_file()}


def invoke(root, *args):
    return subprocess.run(
        [sys.executable, str(helper), "--repo", str(root), "--work-item", wi, *args],
        capture_output=True, text=True,
    )


def bound_fixture(label, *, direct=False, in_contract=False, absent=False):
    root = temporary / ("handoff-" + label)
    shutil.copytree(original, root)
    contract = load(root, contract_path)
    context = contract["resourceContext"]
    if absent:
        contract.pop("resourceContext")
    elif not in_contract:
        contract["resourceContext"] = None
    write(root, contract_path, contract)
    manifest = load(root, manifest_path)
    manifest["files"]["contractDigest"] = raw(root, contract_path)
    write(root, manifest_path, manifest)
    transitions = sorted(
        [(load(root, p.relative_to(root))["sequence"], p, load(root, p.relative_to(root)))
         for p in (root / decisions).glob(f"{wi}.finalize.*.json")]
    )
    receipt = transitions[-1][2]["receipt"] if direct else load(root, root_path)
    def bind_concrete_resources(value):
        value["branch"] = {"name": context["branch"], "remote": context["baseRemote"],
                           "headRevision": "c" * 40}
        value["worktree"] = {"branch": context["branch"], "path": context["worktree"],
                             "headRevision": "c" * 40, "worktreeId": "fixture-worktree-identity"}

    bind_concrete_resources(receipt)
    receipt["contractDigest"] = raw(root, contract_path)
    write(root, root_path, receipt)
    previous = canonical(receipt)
    head, head_receipt, sequence = root_path, receipt, 0
    for seq, path, envelope in transitions:
        path.unlink()
        if direct:
            continue
        bind_concrete_resources(envelope["receipt"])
        envelope["receipt"]["contractDigest"] = raw(root, contract_path)
        envelope["predecessorReceiptDigest"] = previous
        head = decisions / f"{wi}.finalize.{canonical(envelope)[7:]}.json"
        write(root, head, envelope)
        head_receipt, sequence = envelope["receipt"], seq
        previous = canonical(head_receipt)
    change(root, close_path, resourceFinalizationHeadPath=head.as_posix(),
           resourceFinalizationHeadDigest=canonical(head_receipt),
           resourceFinalizationSequence=sequence)
    write(root, binding_path, {
        "schemaVersion": 1, "operation": "resource_context_binding",
        "workItemId": wi, "repositoryId": contract["repositoryId"],
        "contractDigest": raw(root, contract_path),
        "archiveManifestDigest": raw(root, manifest_path),
        "resourceContext": context, "runtimeVersion": "9.9.9",
        "runtimeDigest": "sha256:" + "d" * 64, "recordedAt": "2026-09-16T00:00:00Z",
    })
    return root


positives = 0
for label, options in (
    ("sequence-2", {}), ("sequence-0", {"direct": True}),
    ("absent-contract-context", {"absent": True}),
    ("matching-contract-context", {"in_contract": True}),
):
    root = bound_fixture(label, **options)
    before = inventory(root, ".ai")
    result = invoke(root)
    assert result.returncode == 0, (label, result.returncode, result.stdout, result.stderr)
    assert inventory(root, ".ai") == before, label
    projected = inventory(root, "docs")
    result = invoke(root, "--check")
    assert result.returncode == 0, (label, result.stdout, result.stderr)
    result = invoke(root)
    assert result.returncode == 0, (label, result.stdout, result.stderr)
    assert inventory(root, "docs") == projected, label
    assert inventory(root, ".ai") == before, label
    positives += 1

negatives = 0


def reject(label, mutate, expected, *, direct=True):
    global negatives
    root = bound_fixture(label, direct=direct)
    mutate(root)
    before = inventory(root, "docs")
    result = invoke(root)
    assert result.returncode != 0, (label, result.stdout)
    assert expected in result.stderr, (label, result.stderr)
    assert inventory(root, "docs") == before, label
    negatives += 1


for index, (field, value) in enumerate((
    ("schemaVersion", 2), ("schemaVersion", True), ("operation", "other"),
    ("repositoryId", "sha256:" + "f" * 64), ("workItemId", "WI-998-foreign"),
    ("contractDigest", "sha256:" + "f" * 64),
    ("archiveManifestDigest", "sha256:" + "f" * 64),
    ("runtimeDigest", "invalid"), ("runtimeVersion", ""),
    ("recordedAt", ""), ("unexpected", True),
)):
    reject(f"invalid-binding-{index}",
           lambda root, f=field, v=value: change(root, binding_path, **{f: v}), "binding")


def mutate_context(root, field, value):
    binding = load(root, binding_path)
    binding["resourceContext"][field] = value
    write(root, binding_path, binding)


for field in ("baseBranch", "baseRemote", "provider", "pullRequest"):
    reject("pending-" + field,
           lambda root, f=field: mutate_context(root, f, "pending:future"), "binding")
for field in ("branch", "worktree", "baseBranch", "baseRemote", "provider", "pullRequest"):
    reject("foreign-resource-" + field,
           lambda root, f=field: mutate_context(root, f, "foreign"), "resource context")
    reject("empty-resource-" + field,
           lambda root, f=field: mutate_context(root, f, " "), "binding")


def remove_field(root):
    binding = load(root, binding_path)
    del binding["resourceContext"]["worktree"]
    write(root, binding_path, binding)


reject("missing-context-field", remove_field, "binding")
reject("extra-context-field", lambda root: mutate_context(root, "extra", "value"), "binding")
reject("missing-binding", lambda root: (root / binding_path).unlink(), "no-resource")
reject("malformed-binding", lambda root: (root / binding_path).write_text("{"), "malformed JSON")


def duplicate(root):
    path = root / binding_path
    path.write_text('{"schemaVersion":1,' + path.read_text()[1:], encoding="utf-8")


def symlink(root):
    path = root / binding_path
    target = path.with_suffix(".original")
    path.rename(target)
    path.symlink_to(target.name)


reject("duplicate-binding", duplicate, "duplicate key")
reject("symlink-binding", symlink, "regular non-symlink")


def symlink_parent(root):
    path = root / decisions
    target = path.with_name("real-decisions")
    path.rename(target)
    path.symlink_to(target.name, target_is_directory=True)


reject("symlink-binding-parent", symlink_parent, "binding parent")
reject("tampered-contract-bytes",
       lambda root: (root / contract_path).write_bytes((root / contract_path).read_bytes() + b" "),
       "archive Contract digest mismatch")
reject("tampered-manifest-bytes",
       lambda root: (root / manifest_path).write_bytes((root / manifest_path).read_bytes() + b" "),
       "binding")


def inconsistent_receipt(root, field):
    receipt = load(root, root_path)
    if field == "provider":
        receipt["provider"] = "foreign"
    else:
        receipt["pullRequest"]["url"] = "https://example.invalid/pull/998"
    write(root, root_path, receipt)
    change(root, close_path, resourceFinalizationHeadDigest=canonical(receipt))


for field in ("provider", "pullRequest"):
    reject("inconsistent-receipt-" + field,
           lambda root, f=field: inconsistent_receipt(root, f), "resource context")


def conflicting_contract(root):
    contract = load(root, contract_path)
    contract["resourceContext"] = dict(load(root, binding_path)["resourceContext"], branch="foreign")
    write(root, contract_path, contract)
    manifest = load(root, manifest_path)
    manifest["files"]["contractDigest"] = raw(root, contract_path)
    write(root, manifest_path, manifest)
    change(root, binding_path, contractDigest=raw(root, contract_path), archiveManifestDigest=raw(root, manifest_path))


reject("conflicting-contract", conflicting_contract, "binding")
reject("missing-finalize", lambda root: (root / root_path).unlink(), "regular non-symlink")
for index, (field, value) in enumerate((
    ("resourceFinalizationSequence", 1), ("resourceFinalizationSequence", False),
    ("resourceFinalizationHeadPath", ".ai/decisions/foreign.finalize.json"),
    ("resourceFinalizationHeadDigest", "sha256:" + "f" * 64),
)):
    reject(f"bad-close-{index}",
           lambda root, f=field, v=value: change(root, close_path, **{f: v}), "finalization")


def spurious_finalization(root):
    (root / binding_path).unlink()
    close = load(root, close_path)
    for field in ("resourceFinalizationSequence", "resourceFinalizationHeadPath", "resourceFinalizationHeadDigest"):
        close.pop(field)
    write(root, close_path, close)


reject("no-resource-spurious-finalization", spurious_finalization, "no-resource")


def concrete_resource_mismatch(root, object_name, field, value):
    receipt = load(root, root_path)
    if field is None:
        receipt.pop(object_name)
    else:
        receipt[object_name][field] = value
    write(root, root_path, receipt)
    change(root, close_path, resourceFinalizationHeadDigest=canonical(receipt))


for index, (object_name, field, value) in enumerate((
    ("branch", None, None), ("worktree", None, None),
    ("branch", "name", "foreign"), ("branch", "remote", "foreign"),
    ("worktree", "branch", "foreign"), ("worktree", "path", "/tmp/foreign"),
    ("worktree", "worktreeId", ""), ("worktree", "worktreeId", False),
)):
    reject(f"concrete-resource-{index}",
           lambda root, o=object_name, f=field, v=value: concrete_resource_mismatch(root, o, f, v),
           "resource identity")


def changed_chain_identity(root):
    previous = canonical(load(root, root_path))
    for seq, path, envelope in sorted(
        [(load(root, p.relative_to(root))["sequence"], p, load(root, p.relative_to(root)))
         for p in (root / decisions).glob(f"{wi}.finalize.*.json")]
    ):
        envelope["receipt"]["worktree"]["worktreeId"] = "foreign-worktree-identity"
        envelope["predecessorReceiptDigest"] = previous
        path.unlink()
        new_path = decisions / f"{wi}.finalize.{canonical(envelope)[7:]}.json"
        write(root, new_path, envelope)
        previous = canonical(envelope["receipt"])
        if seq == 2:
            change(root, close_path, resourceFinalizationHeadPath=new_path.as_posix(),
                   resourceFinalizationHeadDigest=previous)


reject("changed-chain-worktree", changed_chain_identity, "resource identity", direct=False)


def retained_root(root):
    receipt = load(root, root_path)
    receipt["result"]["disposition"] = "retained"
    receipt["after"].update(branch="present", worktree="clean")
    write(root, root_path, receipt)
    change(root, close_path, resourceFinalizationHeadDigest=canonical(receipt))


reject("retained-root", retained_root, "must be deleted or abandoned")
print(f"archived handoff consumer: {positives} positive, {negatives} negative cases passed")
PY_HANDOFF

# A finalization receipt binds the archived Contract base separately from the
# provider PR base when the PR was created from an earlier reviewed main.  The
# Contract binding must be checked without pretending those two identities are
# the same.
cp -R "$tmp/unpromoted" "$tmp/separate-contract-base"
python3 - "$tmp/separate-contract-base" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
decision_dir = root / ".ai/decisions"

def canonical(value: object) -> str:
    return "sha256:" + hashlib.sha256(
        json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    ).hexdigest()

def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

contract_path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["baseRevision"] = "b" * 40
write(contract_path, contract)
contract_digest = "sha256:" + hashlib.sha256(contract_path.read_bytes()).hexdigest()
archive_path = root / ".ai/work-items/archive" / f"{work_item}.archive.json"
archive = json.loads(archive_path.read_text(encoding="utf-8"))
archive["files"]["contractDigest"] = contract_digest
write(archive_path, archive)

root_path = decision_dir / f"{work_item}.finalize.json"
root_receipt = json.loads(root_path.read_text(encoding="utf-8"))
root_receipt["contractDigest"] = contract_digest
root_receipt["contractBaseRevision"] = "b" * 40
write(root_path, root_receipt)
previous_digest = canonical(root_receipt)
renamed = []
transition_items = []
for path in decision_dir.glob(f"{work_item}.finalize.*.json"):
    envelope = json.loads(path.read_text(encoding="utf-8"))
    transition_items.append((envelope["sequence"], path, envelope))
for _, path, envelope in sorted(transition_items):
    envelope["predecessorReceiptDigest"] = previous_digest
    envelope["receipt"]["contractDigest"] = contract_digest
    envelope["receipt"]["contractBaseRevision"] = "b" * 40
    new_path = decision_dir / f"{work_item}.finalize.{canonical(envelope).removeprefix('sha256:')}.json"
    path.unlink()
    write(new_path, envelope)
    renamed.append((envelope["sequence"], new_path, envelope))
    previous_digest = canonical(envelope["receipt"])

head_sequence, head_path, head_envelope = renamed[-1]
close_path = decision_dir / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
close["resourceFinalizationHeadPath"] = f".ai/decisions/{head_path.name}"
close["resourceFinalizationHeadDigest"] = canonical(head_envelope["receipt"])
write(close_path, close)
PY
python3 "$helper" --repo "$tmp/separate-contract-base" --work-item WI-999-closed-docs-fixture
python3 "$helper" --repo "$tmp/separate-contract-base" --work-item WI-999-closed-docs-fixture --check

# A superseded predecessor has immutable archive/verification/close evidence,
# a valid append-only supersede decision, and no provider finalization receipt.
# It must be promotable without being mistaken for an approved normal close.
cp -R "$tmp/unpromoted" "$tmp/superseded-predecessor"
python3 - "$tmp/superseded-predecessor" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
repository_id = json.loads((root / ".ai/project.json").read_text(encoding="utf-8"))["repositoryId"]
decision_dir = root / ".ai/decisions"

def canonical(value: object) -> str:
    return "sha256:" + hashlib.sha256(
        json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    ).hexdigest()

def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

for path in decision_dir.glob(f"{work_item}.finalize*.json"):
    path.unlink()

supersede = {
    "schemaVersion": 1,
    "workItemId": work_item,
    "predecessorWorkItemId": work_item,
    "successorWorkItemId": "WI-1000-successor",
    "decision": "supersede",
    "repositoryId": repository_id,
    "reason": "The predecessor close remains immutable history while the successor owns continuation.",
    "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
}
recovery_path = decision_dir / f"{work_item}.recovery.{canonical(supersede).removeprefix('sha256:')}.json"
write(recovery_path, supersede)

close_path = decision_dir / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
close["humanDecision"] = "superseded"
close["structuredDecision"]["decision"] = "superseded"
close["structuredDecision"]["evidenceRefs"] = [
    f".ai/evidence/{work_item}.verification.json",
    f".ai/decisions/{recovery_path.name}",
]
for field in (
    "resourceFinalizationSequence",
    "resourceFinalizationHeadPath",
    "resourceFinalizationHeadDigest",
):
    close.pop(field, None)
write(close_path, close)
PY
python3 "$helper" --repo "$tmp/superseded-predecessor" --work-item WI-999-closed-docs-fixture
python3 "$helper" --repo "$tmp/superseded-predecessor" --work-item WI-999-closed-docs-fixture --check
for document in "$tmp/superseded-predecessor"/docs/work-items/WI-999-closed-docs-fixture*.md; do
  if grep -Fq 'terminalFinalization:' "$document"; then
    echo 'superseded predecessor fabricated terminal finalization' >&2
    exit 1
  fi
done

# A documentation-promotion Work Item registers its own tri-language pages and
# parity ledgers before close.  Its conditional self projection is a bounded
# terminal state: check-all must validate the immutable evidence without
# demanding an endless successor.
cp -R "$tmp/unpromoted" "$tmp/self-terminal"
python3 - "$tmp/self-terminal" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
contract_path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["scope"] = [
    f"docs/work-items/{work_item}.md",
    f"docs/work-items/{work_item}.zh-CN.md",
    f"docs/work-items/{work_item}.ja.md",
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
]
contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")
archive_path = root / ".ai/work-items/archive" / f"{work_item}.archive.json"
archive = json.loads(archive_path.read_text(encoding="utf-8"))
archive["files"]["contractDigest"] = "sha256:" + hashlib.sha256(contract_path.read_bytes()).hexdigest()
archive_path.write_text(json.dumps(archive, indent=2) + "\n", encoding="utf-8")
contract_digest = archive["files"]["contractDigest"]
decision_dir = root / ".ai/decisions"
root_path = decision_dir / f"{work_item}.finalize.json"
root_receipt = json.loads(root_path.read_text(encoding="utf-8"))
root_receipt["contractDigest"] = contract_digest
root_path.write_text(json.dumps(root_receipt, indent=2) + "\n", encoding="utf-8")

def canonical(value: object) -> str:
    return "sha256:" + hashlib.sha256(
        json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    ).hexdigest()

transition_paths = sorted(decision_dir.glob(f"{work_item}.finalize.*.json"))
transitions = []
for path in transition_paths:
    envelope = json.loads(path.read_text(encoding="utf-8"))
    envelope["receipt"]["contractDigest"] = contract_digest
    transitions.append((envelope.get("sequence"), path, envelope))
previous_digest = canonical(root_receipt)
renamed = []
for sequence, path, envelope in sorted(transitions):
    envelope["predecessorReceiptDigest"] = previous_digest
    new_name = f"{work_item}.finalize.{canonical(envelope).removeprefix('sha256:')}.json"
    new_path = path.with_name(new_name)
    path.unlink()
    new_path.write_text(json.dumps(envelope, indent=2) + "\n", encoding="utf-8")
    renamed.append((sequence, new_path, envelope))
    previous_digest = canonical(envelope["receipt"])
close_path = decision_dir / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
head_sequence, head_path, head_envelope = renamed[-1]
close["resourceFinalizationHeadPath"] = f".ai/decisions/{head_path.name}"
close["resourceFinalizationHeadDigest"] = canonical(head_envelope["receipt"])
close_path.write_text(json.dumps(close, indent=2) + "\n", encoding="utf-8")
evidence = f".ai/evidence/{work_item}.verification.json"
finalize = f".ai/decisions/{work_item}.finalize.json"
close = f".ai/decisions/{work_item}.close.json"
for suffix, status in (
    ("", "In progress → Implemented after verified close"),
    (".zh-CN", "进行中 → 验证关闭后已实现"),
    (".ja", "In progress → verified close 後 Implemented"),
):
    parity = root / "docs/reference" / f"reference-parity{suffix}.md"
    row = (
        f"| WI-999 — fixture | {status} | "
        f"[Work Item](../work-items/{work_item}{suffix}.md); "
        f"archive `.ai/work-items/archive/{work_item}.contract.json`; "
        f"verification `{evidence}`; finalization `{finalize}`; close `{close}`. |\n"
    )
    parity.write_text(row, encoding="utf-8")
PY
python3 "$helper" --repo "$tmp/self-terminal" --check-all
python3 "$helper" --repo "$tmp/self-terminal" --work-item WI-999-closed-docs-fixture --check
perl -0pi -e 's/status: in_progress/status: implemented/' \
  "$tmp/self-terminal/docs/work-items/WI-999-closed-docs-fixture.md"
if python3 "$helper" --repo "$tmp/self-terminal" --check-all \
  >"$tmp/self-terminal-drift.out" 2>"$tmp/self-terminal-drift.err"; then
  echo 'self-terminal exception accepted invalid terminal metadata' >&2
  exit 1
fi
grep -Fq 'promotion required' "$tmp/self-terminal-drift.err"

# A valid local/no-resource Work Item has no provider finalization receipt.
# Promotion must accept both an absent and an explicit null resourceContext,
# while never inventing terminalFinalization references.
for variant in absent null; do
  cp -R "$tmp/unpromoted" "$tmp/no-resource-$variant"
  python3 - "$tmp/no-resource-$variant" "$variant" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
variant = sys.argv[2]
work_item = "WI-999-closed-docs-fixture"
contract_path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
if variant == "absent":
    contract.pop("resourceContext", None)
else:
    contract["resourceContext"] = None
contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")

archive_path = root / ".ai/work-items/archive" / f"{work_item}.archive.json"
archive = json.loads(archive_path.read_text(encoding="utf-8"))
archive["files"]["contractDigest"] = "sha256:" + hashlib.sha256(
    contract_path.read_bytes()
).hexdigest()
archive_path.write_text(json.dumps(archive, indent=2) + "\n", encoding="utf-8")

close_path = root / ".ai/decisions" / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
for field in (
    "resourceFinalizationSequence",
    "resourceFinalizationHeadPath",
    "resourceFinalizationHeadDigest",
):
    close.pop(field, None)
close_path.write_text(json.dumps(close, indent=2) + "\n", encoding="utf-8")
for path in (root / ".ai/decisions").glob(f"{work_item}.finalize*.json"):
    path.unlink()
PY
  python3 "$helper" --repo "$tmp/no-resource-$variant" --work-item WI-999-closed-docs-fixture
  python3 "$helper" --repo "$tmp/no-resource-$variant" --work-item WI-999-closed-docs-fixture --check
  for document in "$tmp/no-resource-$variant"/docs/work-items/WI-999-closed-docs-fixture*.md; do
    if grep -Fq 'terminalFinalization:' "$document"; then
      echo "no-resource promotion fabricated finalization for $variant" >&2
      exit 1
    fi
  done
done

# A resource-bound Work Item still requires its root finalization receipt.
cp -R "$tmp/unpromoted" "$tmp/resource-finalization-required"
python3 - "$tmp/resource-finalization-required" <<'PY'
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
for path in (root / ".ai/decisions").glob(f"{work_item}.finalize*.json"):
    if path.name == f"{work_item}.finalize.json":
        path.unlink()
        break
PY
if python3 "$helper" --repo "$tmp/resource-finalization-required" --work-item WI-999-closed-docs-fixture \
  >"$tmp/resource-finalization-required.out" 2>"$tmp/resource-finalization-required.err"; then
  echo 'resource-bound promotion accepted missing finalization' >&2
  exit 1
fi
grep -Fq 'must be a regular non-symlink file' "$tmp/resource-finalization-required.err"

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

# The Runtime's canonical close record binds verification evidence through the
# final Task Outcome report.  Structured human-decision references may contain
# the finalization/archive evidence without duplicating the verification path;
# promotion must accept that authoritative final-report binding while still
# rejecting a missing or malformed final report below.
cp -R "$tmp/unpromoted" "$tmp/final-report-binding"
python3 - "$tmp/final-report-binding" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
close_path = root / ".ai/decisions" / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
sequence_two = sorted((root / ".ai/decisions").glob(f"{work_item}.finalize.*.json"))[-1]
close["structuredDecision"]["evidenceRefs"] = [
    f".ai/decisions/{sequence_two.name}",
]
close_path.write_text(json.dumps(close, indent=2) + "\n", encoding="utf-8")
PY
if python3 "$helper" --repo "$tmp/final-report-binding" --work-item WI-999-closed-docs-fixture --check \
  >"$tmp/final-report-binding.out" 2>"$tmp/final-report-binding.err"; then
  echo 'promotion check accepted stale final-report-binding documentation' >&2
  exit 1
fi
grep -Fq 'promotion required' "$tmp/final-report-binding.err"
python3 "$helper" --repo "$tmp/final-report-binding" --work-item WI-999-closed-docs-fixture
python3 "$helper" --repo "$tmp/final-report-binding" --work-item WI-999-closed-docs-fixture --check

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

# Current Runtime cleanup-before-close path: close binds the deleted
# sequence-1 transition head while retaining the immutable root.
cp -R "$tmp/post-close-reconciliation" "$tmp/pre-close-cleanup"
python3 - "$tmp/pre-close-cleanup" <<'PY'
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

transition_paths = sorted(decision_dir.glob(f"{work_item}.finalize.*.json"))
assert len(transition_paths) == 1
transition_path = transition_paths[0]
transition = json.loads(transition_path.read_text(encoding="utf-8"))
close_path = decision_dir / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
close["resourceFinalizationSequence"] = 1
close["resourceFinalizationHeadPath"] = f".ai/decisions/{transition_path.name}"
close["resourceFinalizationHeadDigest"] = canonical_digest(transition["receipt"])
close["humanDecision"] = "confirmed"
close["structuredDecision"]["decision"] = "confirmed"
close["structuredDecision"]["evidenceRefs"] = [
    f".ai/evidence/{work_item}.verification.json",
]
write_json(close_path, close)
PY
if python3 "$helper" --repo "$tmp/pre-close-cleanup" --work-item WI-999-closed-docs-fixture --check \
  >"$tmp/pre-close-cleanup.out" 2>"$tmp/pre-close-cleanup.err"; then
  echo 'promotion check accepted stale current pre-close-cleanup documentation' >&2
  exit 1
fi
grep -Fq 'promotion required' "$tmp/pre-close-cleanup.err"
python3 "$helper" --repo "$tmp/pre-close-cleanup" --work-item WI-999-closed-docs-fixture
python3 "$helper" --repo "$tmp/pre-close-cleanup" --work-item WI-999-closed-docs-fixture --check

# An explicitly closed, unmerged failed delivery may have a truthful terminal
# abandoned receipt after its branch and worktree are removed.  Promotion must
# accept that non-merge terminal without projecting it as a successful merge.
cp -R "$tmp/unpromoted" "$tmp/abandoned-terminal"
python3 - "$tmp/abandoned-terminal" <<'PY'
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
root_receipt = json.loads(root_path.read_text(encoding="utf-8"))
root_receipt["before"] = {"branch": "present", "pullRequest": "unmerged", "worktree": "clean"}
root_receipt["after"] = {"branch": "deleted", "pullRequest": "unmerged", "worktree": "removed"}
root_receipt["pullRequest"]["mergeCommit"] = None
root_receipt["result"] = {
    "disposition": "abandoned",
    "failureCodes": ["unmerged_pull_request"],
    "unknownCodes": [],
}
write_json(root_path, root_receipt)
for path in decision_dir.glob(f"{work_item}.finalize.*.json"):
    path.unlink()

close_path = decision_dir / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
finalization_path = f".ai/decisions/{work_item}.finalize.json"
close["resourceFinalizationSequence"] = 0
close["resourceFinalizationHeadPath"] = finalization_path
close["resourceFinalizationHeadDigest"] = canonical_digest(root_receipt)
close["structuredDecision"]["evidenceRefs"] = [
    f".ai/evidence/{work_item}.verification.json",
    finalization_path,
]
write_json(close_path, close)
PY
if python3 "$helper" --repo "$tmp/abandoned-terminal" --work-item WI-999-closed-docs-fixture --check \
  >"$tmp/abandoned-terminal.out" 2>"$tmp/abandoned-terminal.err"; then
  echo 'promotion check accepted stale abandoned-terminal documentation' >&2
  exit 1
fi
grep -Fq 'promotion required' "$tmp/abandoned-terminal.err"
python3 "$helper" --repo "$tmp/abandoned-terminal" --work-item WI-999-closed-docs-fixture
python3 "$helper" --repo "$tmp/abandoned-terminal" --work-item WI-999-closed-docs-fixture --check

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

# A predecessor may also have a fully confirmed approved close when a
# successor recovery is recorded.  The valid recovery binding still makes it
# historical; promotion must not require rewriting or re-promoting it.
cp -R "$tmp/unpromoted" "$tmp/confirmed-recovered-predecessor"
python3 - "$tmp/confirmed-recovered-predecessor" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
repository_id = json.loads((root / ".ai/project.json").read_text())["repositoryId"]
(root / ".ai/decisions" / f"{work_item}.recovery.json").write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "workItemId": work_item,
            "predecessorWorkItemId": work_item,
            "successorWorkItemId": "WI-1001-successor",
            "decision": "successor",
            "repositoryId": repository_id,
            "reason": "The confirmed predecessor close remains immutable history.",
            "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
PY
python3 "$helper" --repo "$tmp/confirmed-recovered-predecessor" --check-all

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
expect_no_write_failure close-head "$tmp/close-head" 'close finalization head binding mismatch'

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

# Ordinary implementation Work Items do not acquire six derived-document
# obligations merely because they are closed.  Explicit invocation still
# validates the requested projection, and a docs-mode Contract remains
# mandatory even when one language is missing.
cp -R "$tmp/unpromoted" "$tmp/ordinary-no-docs"
python3 - "$tmp/ordinary-no-docs" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
repository_id = json.loads((root / ".ai/project.json").read_text(encoding="utf-8"))["repositoryId"]
work_item = "WI-999-closed-docs-fixture"
contract_path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["mode"] = "implementation"
contract["operation"] = "implementation.modify"
contract["scope"] = ["crates/example/src/lib.rs"]
# Keep the fixture's terminal evidence valid after changing the archived
# Contract.  This case models a no-resource implementation Work Item, so it
# has no provider-finalization receipts to preserve.
contract["resourceContext"] = None
contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")
archive_path = root / ".ai/work-items/archive" / f"{work_item}.archive.json"
archive = json.loads(archive_path.read_text(encoding="utf-8"))
archive["files"]["contractDigest"] = "sha256:" + hashlib.sha256(
    contract_path.read_bytes()
).hexdigest()
archive_path.write_text(json.dumps(archive, indent=2) + "\n", encoding="utf-8")
close_path = root / ".ai/decisions" / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
for field in (
    "resourceFinalizationSequence",
    "resourceFinalizationHeadPath",
    "resourceFinalizationHeadDigest",
):
    close.pop(field, None)
close_path.write_text(json.dumps(close, indent=2) + "\n", encoding="utf-8")
for path in (root / ".ai/decisions").glob(f"{work_item}.finalize*.json"):
    path.unlink()
for path in (root / "docs/work-items").glob(f"{work_item}*.md"):
    path.unlink()
for path in (root / "docs/reference").glob("reference-parity*.md"):
    path.write_text("", encoding="utf-8")
(root / ".ai/project").mkdir(parents=True, exist_ok=True)
(root / ".ai/project/documentation-policy.json").write_text(
    json.dumps(
        {
            "schemaVersion": 2,
            "repositoryId": repository_id,
            "defaultProjection": "derived",
            "effectiveFromContractCreatedAt": "2026-09-15T00:43:38Z",
            "requiredModes": ["docs", "documentation", "release"],
            "requiredOperations": ["documentation.modify", "release.publish"],
            "preserveExistingRegistrations": True,
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
(root / ".ai/project/capabilities.json").write_text(
    json.dumps(
        {
            "repositoryId": repository_id,
            "operationMappings": {
                "documentation.modify": ["documentation"],
                "implementation.modify": ["rust_implementation"],
                "release.publish": ["release_supply_chain"],
            },
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
PY
python3 "$helper" --repo "$tmp/ordinary-no-docs" --check-all >"$tmp/ordinary-no-docs.out"
python3 - "$tmp/ordinary-no-docs.out" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "current", report
assert report["checked"] == 0, report
assert report["excludedBeforePolicyEffectiveAt"] == 0, report
PY
if python3 "$helper" --repo "$tmp/ordinary-no-docs" \
  --work-item WI-999-closed-docs-fixture >"$tmp/ordinary-explicit.out" 2>"$tmp/ordinary-explicit.err"; then
  echo 'explicit documentation promotion silently skipped an ordinary Work Item' >&2
  exit 1
fi
grep -Fq 'must be a regular non-symlink file' "$tmp/ordinary-explicit.err"

# Legacy release-mode Contracts without an explicit release operation or an
# existing parity registration are historical evidence, not new projection
# obligations. A current Contract that explicitly declares release.publish
# still requires the projection.
cp -R "$tmp/ordinary-no-docs" "$tmp/legacy-release-mode"
python3 - "$tmp/legacy-release-mode" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
contract_path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["mode"] = "release"
contract["createdAt"] = "2026-01-01T00:00:00Z"
contract["scope"] = ["docs/reference/**"]
contract.pop("operation", None)
contract.pop("requestedOperation", None)
contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")
archive_path = root / ".ai/work-items/archive" / f"{work_item}.archive.json"
archive = json.loads(archive_path.read_text(encoding="utf-8"))
archive["files"]["contractDigest"] = "sha256:" + hashlib.sha256(
    contract_path.read_bytes()
).hexdigest()
archive_path.write_text(json.dumps(archive, indent=2) + "\n", encoding="utf-8")
(root / "docs/reference/reference-parity.md").write_text(
    "| WI-999 — legacy | Implemented | Historical registration. |\n",
    encoding="utf-8",
)
PY
python3 "$helper" --repo "$tmp/legacy-release-mode" --check-all >"$tmp/legacy-release-mode.out"
python3 - "$tmp/legacy-release-mode.out" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "current", report
assert report["checked"] == 0, report
assert report["excludedBeforePolicyEffectiveAt"] == 1, report
PY
if python3 "$helper" --repo "$tmp/legacy-release-mode" \
  --work-item WI-999-closed-docs-fixture --check \
  >"$tmp/legacy-release-explicit.out" 2>"$tmp/legacy-release-explicit.err"; then
  echo 'explicit historical promotion check skipped the requested Work Item' >&2
  exit 1
fi
grep -Fq 'must be a regular non-symlink file' "$tmp/legacy-release-explicit.err"

cp -R "$tmp/ordinary-no-docs" "$tmp/current-release-mode"
python3 - "$tmp/current-release-mode" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
contract_path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["createdAt"] = "2026-09-16T00:00:00Z"
contract["mode"] = "release"
contract["scope"] = ["src/example.rs"]
contract.pop("operation", None)
contract.pop("requestedOperation", None)
contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")
archive_path = root / ".ai/work-items/archive" / f"{work_item}.archive.json"
archive = json.loads(archive_path.read_text(encoding="utf-8"))
archive["files"]["contractDigest"] = "sha256:" + hashlib.sha256(
    contract_path.read_bytes()
).hexdigest()
archive_path.write_text(json.dumps(archive, indent=2) + "\n", encoding="utf-8")
PY
if python3 "$helper" --repo "$tmp/current-release-mode" --check-all \
  >"$tmp/current-release-mode.out" 2>"$tmp/current-release-mode.err"; then
  echo 'current release-mode Work Item silently skipped its required projection' >&2
  exit 1
fi
grep -Fq 'must be a regular non-symlink file' "$tmp/current-release-mode.err"

cp -R "$tmp/ordinary-no-docs" "$tmp/explicit-release-operation"
python3 - "$tmp/explicit-release-operation" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
contract_path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["createdAt"] = "2026-09-15T00:43:38Z"
contract["mode"] = "release"
contract["operation"] = "release.publish"
contract.pop("requestedOperation", None)
contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")
archive_path = root / ".ai/work-items/archive" / f"{work_item}.archive.json"
archive = json.loads(archive_path.read_text(encoding="utf-8"))
archive["files"]["contractDigest"] = "sha256:" + hashlib.sha256(
    contract_path.read_bytes()
).hexdigest()
archive_path.write_text(json.dumps(archive, indent=2) + "\n", encoding="utf-8")
PY
if python3 "$helper" --repo "$tmp/explicit-release-operation" --check-all \
  >"$tmp/explicit-release-operation.out" 2>"$tmp/explicit-release-operation.err"; then
  echo 'check-all skipped an explicit release.publish projection' >&2
  exit 1
fi
grep -Fq 'must be a regular non-symlink file' "$tmp/explicit-release-operation.err"

cp -R "$tmp/unpromoted" "$tmp/declared-docs-missing-language"
python3 - "$tmp/declared-docs-missing-language" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-999-closed-docs-fixture"
contract_path = root / ".ai/work-items/archive" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["mode"] = "docs"
contract["scope"] = [
    f"docs/work-items/{work_item}.md",
    f"docs/work-items/{work_item}.zh-CN.md",
    f"docs/work-items/{work_item}.ja.md",
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
]
contract["resourceContext"] = None
contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")
archive_path = root / ".ai/work-items/archive" / f"{work_item}.archive.json"
archive = json.loads(archive_path.read_text(encoding="utf-8"))
archive["files"]["contractDigest"] = "sha256:" + hashlib.sha256(
    contract_path.read_bytes()
).hexdigest()
archive_path.write_text(json.dumps(archive, indent=2) + "\n", encoding="utf-8")
close_path = root / ".ai/decisions" / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
for field in (
    "resourceFinalizationSequence",
    "resourceFinalizationHeadPath",
    "resourceFinalizationHeadDigest",
):
    close.pop(field, None)
close_path.write_text(json.dumps(close, indent=2) + "\n", encoding="utf-8")
for path in (root / ".ai/decisions").glob(f"{work_item}.finalize*.json"):
    path.unlink()
(root / "docs/work-items" / f"{work_item}.ja.md").unlink()
PY
expect_no_write_failure declared-docs-missing-language \
  "$tmp/declared-docs-missing-language" 'must be a regular non-symlink file'
before_failure=$(docs_digest "$tmp/declared-docs-missing-language")
if python3 "$helper" --repo "$tmp/declared-docs-missing-language" --check-all \
  >"$tmp/declared-docs-check-all.out" 2>"$tmp/declared-docs-check-all.err"; then
  echo 'check-all skipped a Work Item whose Contract declares documentation scope' >&2
  exit 1
fi
grep -Fq 'must be a regular non-symlink file' "$tmp/declared-docs-check-all.err"
after_failure=$(docs_digest "$tmp/declared-docs-missing-language")
test "$before_failure" = "$after_failure"

# Older native close records can bind a complete provider transition chain
# through their exact evidence reference without modern close-head fields or
# an in-Contract context. Exercise the real consumer; no historical files in
# this checkout are rewritten and fixture labels are not authentication.
python3 - "$helper" "$tmp/unpromoted" "$tmp" "$root" <<'PY_LEGACY_CONTEXT'
import copy
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

helper, original, temporary, repository = map(Path, sys.argv[1:])
wi = "WI-999-closed-docs-fixture"
contract_path = Path(f".ai/work-items/archive/{wi}.contract.json")
archive_path = Path(f".ai/work-items/archive/{wi}.archive.json")
decisions = Path(".ai/decisions")
root_path = decisions / f"{wi}.finalize.json"
close_path = decisions / f"{wi}.close.json"


def load(root, path):
    return json.loads((root / path).read_text())


def write(root, path, value):
    (root / path).write_text(json.dumps(value, indent=2) + "\n")


def canonical(value):
    return "sha256:" + hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def inventory(root):
    return {p.relative_to(root).as_posix(): p.read_bytes()
            for p in root.rglob("*") if p.is_file()}


def fixture(label, *, absent=False, mutate=lambda value: None):
    root = temporary / ("legacy-context-" + label)
    shutil.copytree(original, root)
    contract = load(root, contract_path)
    context = contract.pop("resourceContext")
    if not absent:
        contract["resourceContext"] = None
    write(root, contract_path, contract)
    digest = "sha256:" + hashlib.sha256((root / contract_path).read_bytes()).hexdigest()
    archive = load(root, archive_path)
    archive["files"]["contractDigest"] = digest
    write(root, archive_path, archive)
    receipts = [load(root, root_path)]
    for path in sorted((root / decisions).glob(f"{wi}.finalize.*.json"),
                       key=lambda p: json.loads(p.read_text())["sequence"]):
        receipts.append(json.loads(path.read_text())["receipt"])
        path.unlink()
    for receipt in receipts:
        receipt["contractDigest"] = digest
        receipt["branch"] = {"name": context["branch"], "remote": context["baseRemote"]}
        receipt["worktree"] = {"branch": context["branch"], "path": context["worktree"],
                               "worktreeId": "legacy-fixture-worktree"}
    receipts[0]["before"] = {"branch": "present", "pullRequest": "unmerged", "worktree": "clean"}
    receipts[0]["after"] = copy.deepcopy(receipts[0]["before"])
    receipts[0]["result"]["failureCodes"] = ["unmerged_pull_request"]
    receipts[1]["before"] = copy.deepcopy(receipts[0]["after"])
    receipts[2]["before"] = copy.deepcopy(receipts[1]["after"])
    mutate(receipts)
    write(root, root_path, receipts[0])
    previous = canonical(receipts[0])
    for sequence, receipt in enumerate(receipts[1:], 1):
        envelope = {"schemaVersion": 1, "sequence": sequence,
                    "transitionId": f"legacy-step-{sequence}",
                    "predecessorReceiptDigest": previous, "receipt": receipt}
        path = decisions / f"{wi}.finalize.{canonical(envelope)[7:]}.json"
        write(root, path, envelope)
        previous = canonical(receipt)
    close = load(root, close_path)
    for field in ("resourceFinalizationSequence", "resourceFinalizationHeadPath",
                  "resourceFinalizationHeadDigest"):
        close.pop(field)
    close["structuredDecision"]["evidenceRefs"] = [
        f".ai/evidence/{wi}.verification.json", path.as_posix()]
    write(root, close_path, close)
    return root


def invoke(root, *args):
    return subprocess.run([sys.executable, str(helper), "--repo", str(root),
                           "--work-item", wi, *args], capture_output=True, text=True)


for absent in (False, True):
    root = fixture(str(absent), absent=absent)
    before = inventory(root / ".ai")
    result = invoke(root)
    assert result.returncode == 0, ("legacy exact chain", result.stdout, result.stderr)
    assert inventory(root / ".ai") == before
    projected = inventory(root / "docs")
    assert invoke(root, "--check").returncode == 0
    assert invoke(root).returncode == 0
    assert inventory(root / "docs") == projected
    assert inventory(root / ".ai") == before

negative_count = 0


def reject(label, mutate_receipts=lambda value: None, mutate_root=lambda root: None):
    global negative_count
    root = fixture(label, mutate=mutate_receipts)
    mutate_root(root)
    before = inventory(root)
    result = invoke(root)
    assert result.returncode != 0, (label, result.stdout, result.stderr)
    assert inventory(root) == before, (label, "rejection wrote input or documents")
    negative_count += 1


for index, (receipt_index, key, value) in enumerate((
    (0, "repositoryId", "foreign"), (1, "workItemId", "WI-998-foreign"),
    (2, "contractDigest", "sha256:" + "0" * 64),
    (1, "resourceContext", {"provider": "foreign"}),
    (2, "worktree", {"worktreeId": "foreign"}),
    (0, "result", {"disposition": "deleted", "failureCodes": [], "unknownCodes": []}),
    (1, "result", {"disposition": "blocked", "failureCodes": [], "unknownCodes": []}),
    (2, "after", {"branch": "present", "pullRequest": "merged", "worktree": "clean"}),
    (1, "before", {"branch": "present", "pullRequest": "merged", "worktree": "clean"}),
)):
    reject(f"bound-receipt-{index}",
           lambda receipts, i=receipt_index, k=key, v=value: receipts[i].update({k: v}))


def missing_reference(root):
    close = load(root, close_path)
    close["structuredDecision"]["evidenceRefs"] = [f".ai/evidence/{wi}.verification.json"]
    write(root, close_path, close)


reject("missing-close-reference", mutate_root=missing_reference)
reject("missing-root", mutate_root=lambda root: (root / root_path).unlink())
reject("missing-transition", mutate_root=lambda root: next(
    p for p in (root / decisions).glob(f"{wi}.finalize.*.json")
    if json.loads(p.read_text())["sequence"] == 1).unlink())


def rewrite_transition(root, sequence, mutate, *, duplicate=False):
    path = next(p for p in (root / decisions).glob(f"{wi}.finalize.*.json")
                if json.loads(p.read_text())["sequence"] == sequence)
    envelope = json.loads(path.read_text())
    mutate(envelope)
    replacement = decisions / f"{wi}.finalize.{canonical(envelope)[7:]}.json"
    if not duplicate:
        path.unlink()
    write(root, replacement, envelope)
    close = load(root, close_path)
    close["structuredDecision"]["evidenceRefs"] = [
        replacement.as_posix() if ref == path.relative_to(root).as_posix() else ref
        for ref in close["structuredDecision"]["evidenceRefs"]]
    write(root, close_path, close)


for label, sequence, mutation, duplicate in (
    ("wrong-predecessor", 2, lambda e: e.update(predecessorReceiptDigest="sha256:" + "0" * 64), False),
    ("wrong-sequence", 2, lambda e: e.update(sequence=3), False),
    ("bool-sequence", 1, lambda e: e.update(sequence=True), False),
    ("duplicate-sequence", 1, lambda e: e.update(transitionId="different-transition"), True),
    ("bool-schema", 1, lambda e: e.update(schemaVersion=True), False),
    ("missing-transition-id", 2, lambda e: e.pop("transitionId"), False),
):
    reject(label, mutate_root=lambda root, s=sequence, m=mutation, d=duplicate:
           rewrite_transition(root, s, m, duplicate=d))


def unconfirmed_close(root):
    close = load(root, close_path)
    close["decisionState"] = "pending"
    write(root, close_path, close)


reject("unconfirmed-close", mutate_root=unconfirmed_close)
reject("root-already-merged", lambda receipts:
       receipts[0]["pullRequest"].update(mergeCommit="f" * 40))
reject("root-unknown", lambda receipts:
       receipts[0]["result"].update(unknownCodes=["unknown"]))
reject("missing-terminal", mutate_root=lambda root: next(
    p for p in (root / decisions).glob(f"{wi}.finalize.*.json")
    if json.loads(p.read_text())["sequence"] == 2).unlink())

# Exercise every currently selected real closed record in read-only mode.
# This catches main-only compatibility failures before PR integration.
sys.path.insert(0, str(helper.parent))
import promote_closed_work_item as promotion
selected, _ = promotion.closed_work_item_selection(repository)
assert selected, "real closed-record corpus is empty"
for work_item in selected:
    promotion.validate_terminal_evidence(repository, work_item)
print(f"legacy context: 2 positive, {negative_count} negative; {len(selected)} real closed records passed")
PY_LEGACY_CONTEXT

echo 'closed Work Item promotion regression passed'
