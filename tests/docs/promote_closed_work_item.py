#!/usr/bin/env python3
"""Promote structurally closed Work Item documentation from immutable evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any


WORK_ITEM_PATTERN = re.compile(r"^WI-[0-9]+[A-Za-z]?-[a-z0-9]+(?:-[a-z0-9]+)*$")
PROMOTION_MINIMUM = 253
TERMINAL_FIELDS = (
    "terminalArchive",
    "terminalVerification",
    "terminalFinalization",
    "terminalDecision",
)
LANGUAGES = (
    ("", "Implemented", "terminal lifecycle"),
    (".zh-CN", "已实现", "终态 lifecycle"),
    (".ja", "Implemented", "terminal lifecycle"),
)


class PromotionError(RuntimeError):
    """A fail-closed promotion validation error."""


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise PromotionError(f"malformed JSON: duplicate key {key!r}")
        result[key] = value
    return result


def regular_file(path: Path) -> None:
    try:
        mode = path.lstat().st_mode
    except OSError as error:
        raise PromotionError(f"{path}: must be a regular non-symlink file") from error
    if stat.S_ISLNK(mode) or not stat.S_ISREG(mode):
        raise PromotionError(f"{path}: must be a regular non-symlink file")


def read_json(path: Path) -> dict[str, Any]:
    regular_file(path)
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicate_keys)
    except PromotionError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise PromotionError(f"{path}: malformed JSON: {error}") from error
    if not isinstance(value, dict):
        raise PromotionError(f"{path}: malformed JSON: top level must be an object")
    return value


def canonical_digest(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def raw_digest(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PromotionError(message)


def valid_recovery_decision(repository: Path, work_item_id: str) -> bool:
    """Return whether a recovery receipt supersedes an immutable predecessor close.

    A recovered predecessor is intentionally not a normal ``closed`` item: its
    historical close bytes may be non-canonical and must remain untouched.
    Promotion therefore skips it here, while the governance inventory remains
    responsible for validating and reporting the recovery boundary.
    """
    decision_dir = repository / ".ai/decisions"
    recovery_paths = [decision_dir / f"{work_item_id}.recovery.json"]
    recovery_paths.extend(sorted(decision_dir.glob(f"{work_item_id}.recovery.*.json")))
    try:
        project = read_json(repository / ".ai/project.json")
    except PromotionError:
        return False
    for recovery_path in recovery_paths:
        if not recovery_path.exists() and not recovery_path.is_symlink():
            continue
        try:
            recovery = read_json(recovery_path)
        except PromotionError:
            continue
        evidence_refs = recovery.get("evidenceRefs")
        if not (
            recovery.get("schemaVersion") == 1
            and recovery.get("workItemId") == work_item_id
            and recovery.get("predecessorWorkItemId") == work_item_id
            and isinstance(recovery.get("successorWorkItemId"), str)
            and bool(recovery["successorWorkItemId"])
            and recovery.get("decision") in {"successor", "supersede"}
            and recovery.get("repositoryId") == project.get("repositoryId")
            and isinstance(evidence_refs, list)
            and bool(evidence_refs)
            and all(isinstance(item, str) and item for item in evidence_refs)
            and isinstance(recovery.get("reason"), str)
            and bool(recovery["reason"].strip())
        ):
            continue
        if recovery_path.name != f"{work_item_id}.recovery.json":
            expected = canonical_digest(recovery).removeprefix("sha256:")
            if recovery_path.name != f"{work_item_id}.recovery.{expected}.json":
                continue
        return True
    return False


@dataclass(frozen=True)
class TerminalEvidence:
    work_item_id: str
    repository_id: str
    archive_path: str
    evidence_path: str
    finalization_path: str
    close_path: str
    recovery_path: str | None


def receipt_identity(
    receipt: dict[str, Any],
    *,
    work_item_id: str,
    repository_id: str,
    contract_digest: str,
    base_revision: str,
) -> None:
    require(receipt.get("workItemId") == work_item_id, "finalization Work Item identity mismatch")
    require(receipt.get("repositoryId") == repository_id, "finalization repository identity mismatch")
    require(receipt.get("contractDigest") == contract_digest, "finalization Contract digest mismatch")
    pull_request = receipt.get("pullRequest")
    require(isinstance(pull_request, dict), "finalization pull request is missing")
    require(
        pull_request.get("baseRevision") == base_revision,
        "finalization pull-request base does not match archived Contract",
    )


def validate_terminal_evidence(repository: Path, work_item_id: str) -> TerminalEvidence:
    if not WORK_ITEM_PATTERN.fullmatch(work_item_id):
        raise PromotionError(f"invalid Work Item ID: {work_item_id}")
    project = read_json(repository / ".ai/project.json")
    repository_id = project.get("repositoryId")
    require(isinstance(repository_id, str) and repository_id, "repository identity is missing")

    archive_path = f".ai/work-items/archive/{work_item_id}.archive.json"
    contract_path = f".ai/work-items/archive/{work_item_id}.contract.json"
    evidence_path = f".ai/evidence/{work_item_id}.verification.json"
    close_path = f".ai/decisions/{work_item_id}.close.json"
    contract_file = repository / contract_path
    archive = read_json(repository / archive_path)
    contract = read_json(contract_file)
    evidence = read_json(repository / evidence_path)
    close = read_json(repository / close_path)

    require(
        archive.get("state") == "archived" and archive.get("workItemId") == work_item_id,
        "archive identity or state mismatch",
    )
    require(
        contract.get("workItemId") == work_item_id,
        "archived Contract Work Item identity mismatch",
    )
    require(
        contract.get("repositoryId") == repository_id,
        "repository identity mismatch in archived Contract",
    )
    base_revision = contract.get("baseRevision")
    require(isinstance(base_revision, str) and base_revision, "archived Contract base is missing")
    archive_files = archive.get("files")
    require(isinstance(archive_files, dict), "archive file manifest is missing")
    require(archive_files.get("contractPath") == contract_path, "archive Contract path mismatch")
    contract_digest = raw_digest(contract_file)
    require(
        archive_files.get("contractDigest") == contract_digest,
        "archive Contract digest mismatch",
    )

    require(evidence.get("workItemId") == work_item_id, "verification Work Item identity mismatch")
    require(evidence.get("repositoryId") == repository_id, "verification repository identity mismatch")
    require(evidence.get("passed") is True, "verification evidence is not passing")
    verification_receipt = evidence.get("receipt")
    require(isinstance(verification_receipt, dict), "verification receipt is missing")
    require(
        verification_receipt.get("workItemId") == work_item_id
        and verification_receipt.get("repositoryId") == repository_id
        and verification_receipt.get("passed") is True,
        "verification receipt identity or result mismatch",
    )

    root_path = repository / f".ai/decisions/{work_item_id}.finalize.json"
    root_receipt = read_json(root_path)
    receipt_identity(
        root_receipt,
        work_item_id=work_item_id,
        repository_id=repository_id,
        contract_digest=contract_digest,
        base_revision=base_revision,
    )
    transitions: dict[int, tuple[Path, dict[str, Any], dict[str, Any]]] = {}
    for path in sorted((repository / ".ai/decisions").glob(f"{work_item_id}.finalize.*.json")):
        envelope = read_json(path)
        sequence = envelope.get("sequence")
        require(
            isinstance(sequence, int) and sequence > 0,
            "finalization transition sequence is invalid",
        )
        require(sequence not in transitions, "finalization chain is ambiguous")
        expected_suffix = canonical_digest(envelope).removeprefix("sha256:")
        require(
            path.name == f"{work_item_id}.finalize.{expected_suffix}.json",
            "finalization chain digest or filename mismatch",
        )
        receipt = envelope.get("receipt")
        require(isinstance(receipt, dict), "finalization transition receipt is missing")
        receipt_identity(
            receipt,
            work_item_id=work_item_id,
            repository_id=repository_id,
            contract_digest=contract_digest,
            base_revision=base_revision,
        )
        transitions[sequence] = (path, envelope, receipt)

    reconciled_after_close = False
    if not transitions:
        # A provider may observe the merge and exact resource cleanup in one
        # atomic receipt.  Accept that terminal root only when its complete
        # deleted shape is explicit; partial or retained roots remain stopped.
        result = root_receipt.get("result")
        before = root_receipt.get("before")
        after = root_receipt.get("after")
        pull_request = root_receipt.get("pullRequest")
        require(
            isinstance(result, dict) and result.get("disposition") == "deleted",
            "direct terminal finalization must be deleted",
        )
        require(
            isinstance(before, dict)
            and before.get("pullRequest") == "merged"
            and before.get("branch") == "present"
            and before.get("worktree") == "clean",
            "direct terminal finalization before-state is incomplete",
        )
        require(
            isinstance(after, dict)
            and after.get("pullRequest") == "merged"
            and after.get("branch") == "deleted"
            and after.get("worktree") == "removed",
            "direct terminal finalization after-state is incomplete",
        )
        require(
            isinstance(pull_request, dict)
            and isinstance(pull_request.get("mergeCommit"), str)
            and bool(pull_request["mergeCommit"]),
            "direct terminal finalization merge commit is missing",
        )
        finalization_sequence = 0
        finalization_path = root_path.relative_to(repository).as_posix()
        finalization_digest = canonical_digest(root_receipt)
    else:
        root_result = root_receipt.get("result")
        root_before = root_receipt.get("before")
        root_after = root_receipt.get("after")
        root_was_retained = (
            isinstance(root_result, dict)
            and root_result.get("disposition") == "retained"
            and isinstance(root_before, dict)
            and root_before.get("pullRequest") == "merged"
            and isinstance(root_after, dict)
            and root_after.get("pullRequest") == "merged"
        )
        if sorted(transitions) == [1] and root_was_retained:
            # A pre-v0.2.34 Runtime could close after recording a retained
            # root.  The Rust Runtime now rejects that order, but it must
            # provide a narrow, append-only cleanup reconciliation for such
            # immutable historical records.  The current Runtime may also
            # record the deleted transition before close. Both shapes share
            # the same strict transition validation below; only the close
            # binding distinguishes historical reconciliation from the
            # current terminal sequence-1 path.
            _, envelope, receipt = transitions[1]
            require(
                envelope.get("predecessorReceiptDigest") == canonical_digest(root_receipt),
                "post-close reconciliation predecessor digest mismatch",
            )
            require(
                receipt.get("before") == root_after,
                "post-close reconciliation before-state mismatch",
            )
            result = receipt.get("result")
            after = receipt.get("after")
            require(
                isinstance(result, dict) and result.get("disposition") == "deleted",
                "post-close reconciliation must be deleted",
            )
            require(
                isinstance(after, dict)
                and after.get("pullRequest") == "merged"
                and after.get("branch") == "deleted"
                and after.get("worktree") == "removed",
                "post-close reconciliation cleanup state is incomplete",
            )
            transition_path = transitions[1][0].relative_to(repository).as_posix()
            transition_digest = canonical_digest(receipt)
            close_sequence = close.get("resourceFinalizationSequence")
            bound_close_path = close.get("resourceFinalizationHeadPath")
            close_digest = close.get("resourceFinalizationHeadDigest")
            if (
                close_sequence == 0
                and bound_close_path == root_path.relative_to(repository).as_posix()
                and close_digest == canonical_digest(root_receipt)
            ):
                # Legacy post-close reconciliation: close remains bound to
                # the retained root and the later transition is historical.
                finalization_sequence = 0
                finalization_path = root_path.relative_to(repository).as_posix()
                finalization_digest = canonical_digest(root_receipt)
                reconciled_after_close = True
            elif (
                close_sequence == 1
                and bound_close_path == transition_path
                and close_digest == transition_digest
            ):
                # Current Runtime flow: cleanup is observed before close, so
                # close binds the deleted sequence-1 head.
                finalization_sequence = 1
                finalization_path = transition_path
                finalization_digest = transition_digest
                reconciled_after_close = False
            else:
                raise PromotionError("sequence-1 close finalization head binding mismatch")
        else:
            require(sorted(transitions) == [1, 2], "finalization chain must have unique sequences 1 and 2")
            reconciled_after_close = False
            previous_digest = canonical_digest(root_receipt)
            merge_commit: str | None = None
            for sequence in (1, 2):
                _, envelope, receipt = transitions[sequence]
                require(
                    envelope.get("predecessorReceiptDigest") == previous_digest,
                    "finalization predecessor digest mismatch",
                )
                pull_request = receipt["pullRequest"]
                current_merge_commit = pull_request.get("mergeCommit")
                require(
                    isinstance(current_merge_commit, str) and current_merge_commit,
                    "finalization merge commit is missing",
                )
                if merge_commit is None:
                    merge_commit = current_merge_commit
                require(current_merge_commit == merge_commit, "finalization merge identity mismatch")
                previous_digest = canonical_digest(receipt)

            head_path, _, head_receipt = transitions[2]
            result = head_receipt.get("result")
            require(
                isinstance(result, dict) and result.get("disposition") == "deleted",
                "sequence-2 finalization is not deleted",
            )
            finalization_sequence = 2
            finalization_path = head_path.relative_to(repository).as_posix()
            finalization_digest = canonical_digest(head_receipt)
    close_finalization_sequence = close.get("resourceFinalizationSequence")
    close_finalization_path = close.get("resourceFinalizationHeadPath")
    close_finalization_digest = close.get("resourceFinalizationHeadDigest")
    if reconciled_after_close:
        require(
            close_finalization_sequence == 0
            and close_finalization_path == root_path.relative_to(repository).as_posix()
            and close_finalization_digest == canonical_digest(root_receipt),
            "close must remain bound to the retained root during reconciliation",
        )
    else:
        require(
            close_finalization_sequence == finalization_sequence
            and close_finalization_path == finalization_path
            and close_finalization_digest == finalization_digest,
            "close finalization head binding mismatch",
        )
    require(
        close.get("workItemId") == work_item_id
        and close.get("repositoryId") == repository_id,
        "close identity mismatch",
    )
    require(
        close.get("state") == "closed"
        and close.get("decisionState") == "confirmed"
        and close.get("humanDecision") in {"approved", "confirmed"},
        "close is not a confirmed approved/confirmed decision",
    )
    structured = close.get("structuredDecision")
    require(
        isinstance(structured, dict)
        and structured.get("decision") in {"approved", "confirmed"},
        "structured close decision is missing",
    )
    structured_refs = structured.get("evidenceRefs")
    require(
        isinstance(structured_refs, list)
        and bool(structured_refs)
        and all(isinstance(reference, str) and reference.strip() for reference in structured_refs),
        "structured close decision references are incomplete",
    )
    final_report = close.get("finalReport")
    bindings = final_report.get("bindings") if isinstance(final_report, dict) else None
    require(
        isinstance(bindings, dict)
        and bindings.get("workItemId") == work_item_id
        and bindings.get("repositoryId") == repository_id
        and evidence_path in bindings.get("evidenceRefs", []),
        "close final report bindings are incomplete",
    )

    recovery_path = f".ai/decisions/{work_item_id}.recovery.json"
    recovery_file = repository / recovery_path
    if recovery_file.exists() or recovery_file.is_symlink():
        recovery = read_json(recovery_file)
        require(
            recovery.get("workItemId") == work_item_id
            and recovery.get("predecessorWorkItemId") == work_item_id
            and recovery.get("repositoryId") == repository_id,
            "recovery identity mismatch",
        )
        # Retry receipts describe an earlier failed attempt and do not
        # supersede a confirmed terminal close. They remain governed history,
        # but must not be projected as a successor recovery reference.
        if recovery.get("decision") in {"successor", "supersede"}:
            pass
        elif recovery.get("decision") == "retry":
            recovery_path = None
        else:
            raise PromotionError("recovery identity mismatch")
    else:
        recovery_path = None

    return TerminalEvidence(
        work_item_id=work_item_id,
        repository_id=repository_id,
        archive_path=contract_path,
        evidence_path=evidence_path,
        finalization_path=finalization_path,
        close_path=close_path,
        recovery_path=recovery_path,
    )


def promoted_frontmatter(text: str, evidence: TerminalEvidence) -> str:
    require(text.startswith("---\n"), "Work Item document frontmatter is missing")
    end = text.find("\n---\n", 4)
    require(end != -1, "Work Item document frontmatter is malformed")
    lines = text[4:end].splitlines()
    values = {
        "status": "implemented",
        "lastVerifiedBy": evidence.work_item_id,
        "terminalArchive": evidence.archive_path,
        "terminalVerification": evidence.evidence_path,
        "terminalFinalization": evidence.finalization_path,
        "terminalDecision": evidence.close_path,
    }
    counts: dict[str, int] = {}
    retained: list[str] = []
    insert_at: int | None = None
    for line in lines:
        match = re.match(r"^([A-Za-z][A-Za-z0-9]*):", line)
        key = match.group(1) if match else None
        if key in values:
            counts[key] = counts.get(key, 0) + 1
            if key in TERMINAL_FIELDS:
                continue
            retained.append(f"{key}: {values[key]}")
            if key == "lastVerifiedBy":
                insert_at = len(retained)
        else:
            retained.append(line)
    require(counts.get("status") == 1, "frontmatter status must appear exactly once")
    require(
        counts.get("lastVerifiedBy") == 1,
        "frontmatter lastVerifiedBy must appear exactly once",
    )
    require(
        all(counts.get(field, 0) <= 1 for field in TERMINAL_FIELDS),
        "terminal frontmatter field is duplicated",
    )
    require(insert_at is not None, "frontmatter terminal insertion point is missing")
    terminal_lines = [f"{field}: {values[field]}" for field in TERMINAL_FIELDS]
    retained[insert_at:insert_at] = terminal_lines
    return "---\n" + "\n".join(retained) + text[end:]


def promoted_parity(
    text: str,
    *,
    suffix: str,
    status: str,
    label: str,
    evidence: TerminalEvidence,
) -> str:
    short = evidence.work_item_id.split("-", 2)[:2]
    short_id = "-".join(short)
    pattern = re.compile(rf"^\|\s*{re.escape(short_id)}(?=\s|—|\|).*$", re.IGNORECASE)
    lines = text.splitlines(keepends=True)
    indexes = [index for index, line in enumerate(lines) if pattern.match(line.rstrip("\n"))]
    require(len(indexes) == 1, f"expected exactly one parity row for {short_id}")
    index = indexes[0]
    cells = [cell.strip() for cell in lines[index].rstrip("\n").split("|")[1:-1]]
    require(len(cells) >= 3, f"parity row for {short_id} is malformed")
    existing = lines[index]
    # A recovery retry can share the predecessor's numeric parity row.  In
    # that case the row intentionally carries both terminal projections and
    # both immutable evidence lineages; normal promotion must validate the
    # retry references without erasing the predecessor recovery boundary.
    recovery_marker = "recovered" in cells[1].casefold() or "已恢复" in cells[1]
    if recovery_marker:
        required_retry_references = (
            f"[Work Item](../work-items/{evidence.work_item_id}{suffix}.md)",
            f"`{evidence.archive_path}`",
            f"`{evidence.evidence_path}`",
            f"`{evidence.finalization_path}`",
            f"`{evidence.close_path}`",
        )
        require(
            all(reference in existing for reference in required_retry_references),
            "mixed recovery/retry parity row is missing retry evidence",
        )
        return "".join(lines)
    evidence_parts = [
        f"[Work Item](../work-items/{evidence.work_item_id}{suffix}.md)",
        f"{label}: archive `{evidence.archive_path}`",
        f"verification `{evidence.evidence_path}`",
        f"finalization `{evidence.finalization_path}`",
        f"close `{evidence.close_path}`",
    ]
    if evidence.recovery_path is not None:
        evidence_parts.append(f"recovery `{evidence.recovery_path}`")
    newline = "\n" if lines[index].endswith("\n") else ""
    lines[index] = f"| {cells[0]} | {status} | {'; '.join(evidence_parts)}. |{newline}"
    return "".join(lines)


def planned_changes(repository: Path, evidence: TerminalEvidence) -> dict[Path, str]:
    changes: dict[Path, str] = {}
    for suffix, status, label in LANGUAGES:
        document = repository / f"docs/work-items/{evidence.work_item_id}{suffix}.md"
        parity = repository / f"docs/reference/reference-parity{suffix}.md"
        regular_file(document)
        regular_file(parity)
        changes[document] = promoted_frontmatter(document.read_text(encoding="utf-8"), evidence)
        changes[parity] = promoted_parity(
            parity.read_text(encoding="utf-8"),
            suffix=suffix,
            status=status,
            label=label,
            evidence=evidence,
        )
    return changes


def bounded_documentation_projection(repository: Path, work_item_id: str) -> bool:
    """Return whether a closed Work Item is a bounded self-projection task.

    Documentation promotion is intentionally a terminal projection boundary:
    its own planning pages are registered before close, so rewriting them after
    close would create an unbounded chain of documentation-only successors.
    The exception is deliberately narrow and derived from the immutable
    archived Contract scope; arbitrary documentation-only Work Items do not
    qualify.  Terminal evidence is still validated by the caller.
    """
    contract_path = repository / ".ai/work-items/archive" / f"{work_item_id}.contract.json"
    try:
        contract = read_json(contract_path)
    except PromotionError:
        return False
    scope = contract.get("scope")
    if not isinstance(scope, list) or not scope:
        return False
    paths = [item for item in scope if isinstance(item, str)]
    if len(paths) != len(scope) or any(
        not item
        or item.startswith(("/", "\\"))
        or ".." in Path(item).parts
        or "*" in item
        for item in paths
    ):
        return False
    parity_paths = {
        "docs/reference/reference-parity.md",
        "docs/reference/reference-parity.zh-CN.md",
        "docs/reference/reference-parity.ja.md",
    }
    own_documents = {
        f"docs/work-items/{work_item_id}.md",
        f"docs/work-items/{work_item_id}.zh-CN.md",
        f"docs/work-items/{work_item_id}.ja.md",
    }
    if not parity_paths.issubset(paths) or not own_documents.issubset(paths):
        return False
    for item in paths:
        if item in parity_paths:
            continue
        if not item.startswith("docs/work-items/") or not item.endswith((".md", ".zh-CN.md", ".ja.md")):
            return False
    return True


def self_projection_pending(
    repository: Path,
    work_item_id: str,
    changes: dict[Path, str],
    stale: list[Path],
) -> bool:
    """Validate the exact pre-archive shape of a self-projection task.

    Only the current Work Item's three pages and three parity rows may remain
    conditional.  Their bodies/rows must otherwise equal the deterministic
    promoted form, preventing this exception from hiding arbitrary drift.
    """
    if not bounded_documentation_projection(repository, work_item_id):
        return False
    allowed = {
        repository / f"docs/work-items/{work_item_id}{suffix}.md"
        for suffix in ("", ".zh-CN", ".ja")
    }
    allowed.update(
        repository / f"docs/reference/reference-parity{suffix}.md"
        for suffix in ("", ".zh-CN", ".ja")
    )
    if not stale or any(path not in allowed for path in stale):
        return False

    for path in stale:
        current = path.read_text(encoding="utf-8")
        expected = changes[path]
        if path.parent.name == "work-items":
            if not current.startswith("---\n") or "\n---\n" not in current[4:]:
                return False
            end = current.find("\n---\n", 4)
            current_frontmatter = current[4:end].splitlines()
            expected_end = expected.find("\n---\n", 4)
            expected_frontmatter = expected[4:expected_end].splitlines()
            current_values: dict[str, list[str]] = {}
            expected_values: dict[str, list[str]] = {}
            for line in current_frontmatter:
                key = line.split(":", 1)[0] if ":" in line else ""
                current_values.setdefault(key, []).append(line)
            for line in expected_frontmatter:
                key = line.split(":", 1)[0] if ":" in line else ""
                expected_values.setdefault(key, []).append(line)
            if current_values.get("status") != ["status: in_progress"]:
                return False
            if current_values.get("lastVerifiedBy") != [f"lastVerifiedBy: {work_item_id}"]:
                return False
            if any(current_values.get(field) for field in TERMINAL_FIELDS):
                return False
            for key in set(current_values) | set(expected_values):
                if key in {"status", "lastVerifiedBy", *TERMINAL_FIELDS}:
                    continue
                if current_values.get(key) != expected_values.get(key):
                    return False
            if current[end:] != expected[expected_end:]:
                return False
        else:
            current_lines = current.splitlines(keepends=True)
            expected_lines = expected.splitlines(keepends=True)
            short_id = "-".join(work_item_id.split("-", 2)[:2])
            row_pattern = re.compile(rf"^\|\s*{re.escape(short_id)}(?=\s|—|\|)", re.IGNORECASE)
            indexes = [index for index, line in enumerate(current_lines) if row_pattern.match(line)]
            if len(indexes) != 1 or len(expected_lines) != len(current_lines):
                return False
            index = indexes[0]
            cells = [cell.strip() for cell in current_lines[index].rstrip("\n").split("|")[1:-1]]
            if len(cells) < 3:
                return False
            suffix = (
                ".zh-CN"
                if path.name.endswith(".zh-CN.md")
                else (".ja" if path.name.endswith(".ja.md") else "")
            )
            required_references = (
                f"[Work Item](../work-items/{work_item_id}{suffix}.md)",
                f"archive `.ai/work-items/archive/{work_item_id}.contract.json`",
                f"verification `.ai/evidence/{work_item_id}.verification.json`",
                f"finalization `.ai/decisions/{work_item_id}.finalize.json`",
                f"close `.ai/decisions/{work_item_id}.close.json`",
            )
            if any(reference not in current_lines[index] for reference in required_references):
                return False
            expected_status = (
                "进行中 → 验证关闭后已实现"
                if path.name.endswith(".zh-CN.md")
                else (
                    "In progress → verified close 後 Implemented"
                    if path.name.endswith(".ja.md")
                    else "In progress → Implemented after verified close"
                )
            )
            if cells[1] != expected_status:
                return False
            if current_lines[:index] != expected_lines[:index] or current_lines[index + 1 :] != expected_lines[index + 1 :]:
                return False
    return True


def write_all(changes: dict[Path, str]) -> None:
    pending: list[tuple[Path, Path]] = []
    try:
        for path, text in changes.items():
            descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
            temporary_path = Path(temporary)
            with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
                stream.write(text)
                stream.flush()
                os.fsync(stream.fileno())
            os.chmod(temporary_path, stat.S_IMODE(path.stat().st_mode))
            pending.append((path, temporary_path))
        for path, temporary_path in pending:
            os.replace(temporary_path, path)
    finally:
        for _, temporary_path in pending:
            try:
                temporary_path.unlink()
            except FileNotFoundError:
                pass


def promote(repository: Path, work_item_id: str, *, check: bool) -> dict[str, Any]:
    evidence = validate_terminal_evidence(repository, work_item_id)
    changes = planned_changes(repository, evidence)
    stale = [path for path, expected in changes.items() if path.read_text(encoding="utf-8") != expected]
    if check and stale and self_projection_pending(repository, work_item_id, changes, stale):
        return {
            "changedPaths": [path.relative_to(repository).as_posix() for path in stale],
            "mode": "check",
            "state": "self_terminal",
            "terminalEvidence": {
                "archive": evidence.archive_path,
                "close": evidence.close_path,
                "finalization": evidence.finalization_path,
                "verification": evidence.evidence_path,
            },
            "workItemId": work_item_id,
        }
    if check and stale:
        paths = ", ".join(path.relative_to(repository).as_posix() for path in stale)
        raise PromotionError(f"{work_item_id}: promotion required for {paths}")
    if not check and stale:
        write_all({path: changes[path] for path in stale})
    return {
        "changedPaths": [path.relative_to(repository).as_posix() for path in stale],
        "mode": "check" if check else "write",
        "state": "promoted" if stale else "current",
        "terminalEvidence": {
            "archive": evidence.archive_path,
            "close": evidence.close_path,
            "finalization": evidence.finalization_path,
            "verification": evidence.evidence_path,
        },
        "workItemId": work_item_id,
    }


def closed_work_items(repository: Path) -> list[str]:
    decisions = repository / ".ai/decisions"
    result: list[str] = []
    for close in sorted(decisions.glob("WI-*.close.json")):
        regular_file(close)
        work_item_id = close.name.removesuffix(".close.json")
        match = re.match(r"^WI-([0-9]+)", work_item_id)
        if match and int(match.group(1)) >= PROMOTION_MINIMUM:
            # Recovery is a separate terminal projection for an immutable
            # predecessor.  Do not ask the normal promotion path to invent an
            # approved close for it; the successor owns the future promotion.
            # A valid successor/supersede receipt makes the predecessor an
            # immutable historical projection, regardless of whether an
            # earlier Runtime already recorded a confirmed close.  The
            # recovery binding, not the shape of the predecessor close, owns
            # this exception.  Retry and invalid/foreign receipts continue
            # through normal promotion validation.
            if valid_recovery_decision(repository, work_item_id):
                continue
            result.append(work_item_id)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True, type=Path)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--work-item")
    group.add_argument("--check-all", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    repository = args.repo.resolve()
    try:
        if args.check_all:
            reports = [promote(repository, work_item_id, check=True) for work_item_id in closed_work_items(repository)]
            print(json.dumps({"checked": len(reports), "reports": reports, "state": "current"}, indent=2))
        else:
            print(json.dumps(promote(repository, args.work_item, check=args.check), indent=2))
    except PromotionError as error:
        print(f"closed Work Item promotion failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
