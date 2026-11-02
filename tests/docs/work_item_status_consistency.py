#!/usr/bin/env python3
"""Fail closed when terminal Work Item documentation drifts from repository truth."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

from work_item_projection_policy import (
    ProjectionPolicyError,
    contract_predates_projection_policy,
    load_projection_policy,
)


PARITY_DOCUMENTS = (
    ("docs/reference/reference-parity.md", {"Implemented": "implemented", "Recovered": "recovered"}),
    ("docs/reference/reference-parity.zh-CN.md", {"已实现": "implemented", "已恢复": "recovered"}),
    ("docs/reference/reference-parity.ja.md", {"Implemented": "implemented", "実装済み": "implemented", "Recovered": "recovered", "復旧済み": "recovered"}),
)

def load_regular_json(path: Path) -> dict[str, Any] | None:
    if path.is_symlink() or not path.is_file():
        return None
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    return value if isinstance(value, dict) else None


def frontmatter(path: Path) -> dict[str, str] | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if not text.startswith("---\n"):
        return None
    parts = text.split("---\n", 2)
    if len(parts) != 3:
        return None
    result: dict[str, str] = {}
    for line in parts[1].splitlines():
        if ":" not in line or line[:1].isspace():
            continue
        key, value = line.split(":", 1)
        result[key] = value.strip()
    return result


def short_id(work_item_id: str) -> str | None:
    match = re.match(r"^(WI-[0-9]+[A-Za-z]?)($|-)", work_item_id, re.IGNORECASE)
    return match.group(1).upper() if match else None


def parity_work_item_id(line: str) -> str | None:
    """Resolve a parity row's full id while retaining legacy short rows."""
    cells = [cell.strip() for cell in line.split("|")[1:-1]]
    if not cells:
        return None
    match = re.match(
        r"^(WI-[0-9]+[A-Za-z]?(?:-[A-Za-z0-9][A-Za-z0-9-]*)?)(?=\s|—|\||$)",
        cells[0],
        re.IGNORECASE,
    )
    if match is None:
        return None
    value = match.group(1)
    return value if re.match(r"^WI-[0-9]+[A-Za-z]?-[A-Za-z0-9]", value) else value.upper()


def parity_statuses_for_work_item(
    rows: dict[str, list[str]], work_item_id: str
) -> list[str]:
    """Prefer an exact full-id projection and fall back to its short id."""
    return rows.get(work_item_id) or rows.get(short_id(work_item_id) or "", [])


def parity_statuses(repository: Path) -> tuple[dict[str, list[str]], list[str]]:
    rows: dict[str, list[str]] = {}
    errors: list[str] = []
    for relative, vocabulary in PARITY_DOCUMENTS:
        path = repository / relative
        if path.is_symlink() or not path.is_file():
            errors.append(f"{relative}: parity document must be a regular file")
            continue
        seen: set[str] = set()
        for line in path.read_text(encoding="utf-8").splitlines():
            work_item = parity_work_item_id(line)
            if work_item is None:
                continue
            cells = [cell.strip() for cell in line.split("|")[1:-1]]
            if len(cells) < 2:
                continue
            status = vocabulary.get(cells[1])
            if status is None:
                # A recovery retry may share the predecessor's numeric parity
                # row.  Keep one human-readable row while retaining both
                # terminal projections in its status cell.
                has_implemented = any(
                    token in cells[1]
                    for token in vocabulary
                    if vocabulary[token] == "implemented"
                )
                has_recovered = any(
                    token in cells[1]
                    for token in vocabulary
                    if vocabulary[token] == "recovered"
                )
                if has_implemented and has_recovered:
                    status = "mixed"
            if status is None and "→" in cells[1] and any(
                marker in cells[1].lower()
                for marker in ("close", "关闭", "verified close 後", "検証済みクローズ後")
            ):
                status = "conditional"
            if status is None:
                continue
            if work_item in seen:
                errors.append(f"{relative}: duplicate parity row for {work_item}")
                continue
            seen.add(work_item)
            rows.setdefault(work_item, []).append(status)
    return rows, errors


def valid_contract(path: Path, work_item_id: str, repository_id: str) -> bool:
    value = load_regular_json(path)
    return bool(
        value
        and value.get("workItemId") == work_item_id
        and value.get("repositoryId") == repository_id
    )


def bounded_documentation_projection(repository: Path, work_item_id: str) -> bool:
    """Recognize the narrow self-projection boundary used by doc promotions.

    A documentation-promotion Work Item registers its own tri-language pages
    and parity ledgers before close.  Requiring those same pages to be rewritten
    after close would create an infinite chain.  Only an exact docs-only scope
    with the three own pages and three parity ledgers receives this bounded
    projection treatment; mixed or wildcard scopes remain fail-closed.
    """
    contract = load_regular_json(
        repository / ".ai/work-items/archive" / f"{work_item_id}.contract.json"
    )
    if not contract:
        return False
    scope = contract.get("scope")
    if not isinstance(scope, list) or not scope or any(not isinstance(item, str) for item in scope):
        return False
    if any(
        not item
        or item.startswith(("/", "\\"))
        or ".." in Path(item).parts
        or "*" in item
        for item in scope
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
    if not parity_paths.issubset(scope) or not own_documents.issubset(scope):
        return False
    return all(
        item in parity_paths
        or (item.startswith("docs/work-items/") and item.endswith(".md"))
        for item in scope
    )


def valid_close(path: Path, work_item_id: str, repository_id: str) -> bool:
    value = load_regular_json(path)
    return bool(
        value
        and value.get("workItemId") == work_item_id
        and value.get("repositoryId") == repository_id
        and value.get("state") == "closed"
        and value.get("decisionState") == "confirmed"
        and value.get("humanDecision") in {"approved", "confirmed", "superseded"}
    )


def valid_recovery(path: Path, work_item_id: str, repository_id: str) -> bool:
    value = load_regular_json(path)
    return bool(
        value
        and value.get("schemaVersion") == 1
        and value.get("workItemId") == work_item_id
        and value.get("predecessorWorkItemId") == work_item_id
        and value.get("repositoryId") == repository_id
        and value.get("decision") in {"successor", "supersede"}
        and isinstance(value.get("successorWorkItemId"), str)
        and value["successorWorkItemId"].strip()
    )


def canonical_json_digest(value: Any) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def raw_file_digest(path: Path) -> str | None:
    if path.is_symlink() or not path.is_file():
        return None
    try:
        return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError:
        return None


def digest_named_recovery(path: Path, value: dict[str, Any]) -> bool:
    expected_name = f"{value.get('workItemId')}.recovery.{canonical_json_digest(value)[7:]}.json"
    return path.name == expected_name


def archived_recovery_bundle(
    repository: Path, work_item_id: str, repository_id: str
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]] | None:
    """Read one immutable archive bundle without trusting a partial projection."""
    archive = repository / ".ai/work-items/archive"
    manifest_path = archive / f"{work_item_id}.archive.json"
    manifest = load_regular_json(manifest_path)
    if (
        not manifest
        or manifest.get("workItemId") != work_item_id
        or manifest.get("state") != "archived"
        or not isinstance(manifest.get("files"), dict)
    ):
        return None
    files = manifest["files"]
    records: dict[str, dict[str, Any]] = {}
    for key in ("contract", "summary", "outcome"):
        relative = f".ai/work-items/archive/{work_item_id}.{key}.json"
        path = repository / relative
        value = load_regular_json(path)
        if (
            not value
            or files.get(f"{key}Path") != relative
            or files.get(f"{key}Digest") != raw_file_digest(path)
        ):
            return None
        records[key] = value
    contract = records["contract"]
    summary = records["summary"]
    outcome = records["outcome"]
    if (
        contract.get("workItemId") != work_item_id
        or contract.get("repositoryId") != repository_id
        or summary.get("workItemId") != work_item_id
        or summary.get("repositoryId") != repository_id
        or outcome.get("workItemId") != work_item_id
    ):
        return None
    return manifest, contract, summary, outcome


def recovery_binds_archive_bundle_digests(
    repository: Path,
    work_item_id: str,
    recovery: dict[str, Any],
    bundle: tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]],
) -> bool:
    manifest, contract, summary, outcome = bundle
    archive = repository / ".ai/work-items/archive"
    manifest_path = archive / f"{work_item_id}.archive.json"
    expected = {
        "predecessorContractDigest": canonical_json_digest(contract),
        "predecessorSummaryDigest": canonical_json_digest(summary),
        "predecessorOutcomeDigest": canonical_json_digest(outcome),
    }
    events_path = archive / f"{work_item_id}.events.jsonl"
    events_digest = raw_file_digest(events_path)
    if events_digest is not None:
        expected["predecessorEventsDigest"] = events_digest
    if any(recovery.get(key) != value for key, value in expected.items()):
        return False
    if events_digest is None and recovery.get("predecessorEventsDigest") is not None:
        return False
    manifest_digest = recovery.get("predecessorArchiveManifestDigest")
    if manifest_digest is not None and manifest_digest != raw_file_digest(manifest_path):
        return False
    return manifest.get("workItemId") == work_item_id

def recovery_binds_archive_bundle(
    repository: Path,
    work_item_id: str,
    recovery: dict[str, Any],
    bundle: tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]],
) -> bool:
    references = recovery.get("evidenceRefs")
    return (
        recovery_binds_archive_bundle_digests(repository, work_item_id, recovery, bundle)
        and isinstance(references, list)
        and all(isinstance(reference, str) for reference in references)
        and f".ai/work-items/archive/{work_item_id}.archive.json" in references
    )



def ordinary_cleanup_tuple_matches(
    repository: Path,
    work_item_id: str,
    repository_id: str,
    contract: dict[str, Any],
    close: dict[str, Any],
) -> bool:
    """Check consistency of the native ordinary-close tuple; not issuer authority.

    The captured cleanup HEAD is the governance commit at close, not necessarily
    the executed source or the later commit carrying this close record. The close
    writer and verification executor are independent Runtime identities.
    """
    binding = close.get("ordinaryCleanupBinding")
    if not isinstance(binding, dict):
        return False
    manifest_path = repository / f".ai/work-items/archive/{work_item_id}.archive.json"
    digest = lambda value: isinstance(value, str) and re.fullmatch(r"sha256:[0-9a-f]{64}", value) is not None
    if (
        type(binding.get("schemaVersion")) is not int
        or binding.get("schemaVersion") != 1
        or close.get("ordinaryCleanupBindingDigest") != canonical_json_digest(binding)
        or binding.get("workItemId") != work_item_id
        or binding.get("repositoryId") != repository_id
        or binding.get("contractDigest") != canonical_json_digest(contract)
        or not digest(binding.get("archiveManifestDigest"))
        or binding.get("archiveManifestDigest") != raw_file_digest(manifest_path)
        or not isinstance(binding.get("headRevision"), str)
        or re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", binding["headRevision"]) is None
        or any(not isinstance(binding.get(key), str) or not binding[key].strip()
               for key in ("branch", "branchRef", "capturedAt", "worktreeGitDir", "worktreeId", "worktreePath", "runtimeVersion"))
        or binding.get("branchRef") != "refs/heads/" + binding["branch"]
        or not digest(binding.get("runtimeDigest"))
    ):
        return False
    # Older legitimate resource/ordinary records need not have top-level writer
    # metadata. If present it must agree; never substitute executor identity.
    for key in ("runtimeVersion", "runtimeDigest"):
        if close.get(key) is not None and close[key] != binding[key]:
            return False
    return True


def _verified_native_close_record(
    repository: Path,
    work_item_id: str,
    repository_id: str,
    bundle: tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]],
) -> dict[str, Any] | None:
    _, contract, summary, _ = bundle
    verification_relative = f".ai/evidence/{work_item_id}.verification.json"
    verification = load_regular_json(repository / verification_relative)
    if not verification:
        return None
    contract_digest = verification.get("contractDigest")
    snapshot_digest = verification.get("repositorySnapshotDigest")
    receipt = verification.get("receipt")
    plan_receipt = receipt.get("planReceipt") if isinstance(receipt, dict) else None
    if (
        verification.get("passed") is not True
        or verification.get("workItemId") != work_item_id
        or verification.get("repositoryId") != repository_id
        or not isinstance(contract_digest, str)
        or contract_digest
        not in {
            summary.get("checkpointContractDigest"),
            summary.get("preflightContractDigest"),
        }
        or not isinstance(snapshot_digest, str)
        or snapshot_digest
        not in {
            summary.get("checkpointRepositorySnapshotDigest"),
            summary.get("preflightRepositorySnapshotDigest"),
        }
        or not isinstance(receipt, dict)
        or receipt.get("passed") is not True
        or receipt.get("workItemId") != work_item_id
        or receipt.get("repositoryId") != repository_id
        or verification.get("receiptDigest") != canonical_json_digest(receipt)
        or not isinstance(plan_receipt, dict)
        or plan_receipt.get("workItemId") != work_item_id
        or plan_receipt.get("repositoryId") != repository_id
        or plan_receipt.get("repositorySnapshotDigest") != snapshot_digest
    ):
        return None
    close_relative = f".ai/decisions/{work_item_id}.close.json"
    close = load_regular_json(repository / close_relative)
    if not close or not valid_close(repository / close_relative, work_item_id, repository_id):
        return None
    final_report = close.get("finalReport")
    bindings = final_report.get("bindings") if isinstance(final_report, dict) else None
    structured = close.get("structuredDecision")
    return close if (
        isinstance(final_report, dict)
        and close.get("finalReportDigest") == canonical_json_digest(final_report)
        and final_report.get("status") == "verified"
        and final_report.get("humanStatusColor") == "green"
        and isinstance(bindings, dict)
        and bindings.get("workItemId") == work_item_id
        and bindings.get("repositoryId") == repository_id
        and bindings.get("repositorySnapshotDigest") == snapshot_digest
        and verification_relative in bindings.get("evidenceRefs", [])
        and isinstance(structured, dict)
        and structured.get("decision") in {"approved", "confirmed"}
        and close.get("humanDecision") == structured.get("decision")
        and (
            not any(key in close for key in ("ordinaryCleanupBinding", "ordinaryCleanupBindingDigest"))
            or ordinary_cleanup_tuple_matches(repository, work_item_id, repository_id, contract, close)
        )
    ) else None

def terminal_successor_is_verified_and_closed(
    repository: Path,
    work_item_id: str,
    repository_id: str,
    bundle: tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]],
) -> bool:
    close = _verified_native_close_record(repository, work_item_id, repository_id, bundle)
    return bool(
        close
        and f".ai/evidence/{work_item_id}.verification.json"
        in close["structuredDecision"].get("evidenceRefs", [])
    )



def valid_terminal_successor_lineage(
    repository: Path, work_item_id: str, repository_id: str
) -> bool:
    """Follow one bounded, immutable successor lineage to a closed terminal node."""
    current = work_item_id
    visited: set[str] = set()
    for _ in range(64):
        if current in visited:
            return False
        visited.add(current)
        bundle = archived_recovery_bundle(repository, current, repository_id)
        if bundle is None:
            return False
        candidates: list[tuple[Path, dict[str, Any], str, tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]]]] = []
        for path in recovery_candidates(repository, current):
            recovery = load_regular_json(path)
            if (
                not recovery
                or not valid_recovery(path, current, repository_id)
                or recovery.get("decision") not in {"successor", "supersede"}
                or (path.name != f"{current}.recovery.json" and not digest_named_recovery(path, recovery))
                or not recovery_binds_archive_bundle(repository, current, recovery, bundle)
            ):
                continue
            successor = recovery.get("successorWorkItemId")
            if not isinstance(successor, str) or not successor or successor in visited:
                continue
            successor_bundle = archived_recovery_bundle(repository, successor, repository_id)
            if successor_bundle is None:
                continue
            successor_contract = successor_bundle[1]
            if (
                successor_contract.get("predecessorWorkItemId") != current
                or successor_contract.get("recoveryDecisionPath")
                != path.relative_to(repository).as_posix()
            ):
                continue
            candidates.append((path, recovery, successor, successor_bundle))
        if len(candidates) != 1:
            return False
        _, _, successor, successor_bundle = candidates[0]
        if terminal_successor_is_verified_and_closed(
            repository, successor, repository_id, successor_bundle
        ):
            return True
        current = successor
    return False


def valid_terminal_supersession(
    repository: Path, work_item_id: str, repository_id: str
) -> bool:
    """Validate a terminal successor before accepting conditional historical parity.

    The predecessor stays byte-for-byte immutable. Its exact archived bundle,
    the digest-named supersede decision, and the successor's archived,
    verified, and confirmed-close chain must all agree.
    """
    predecessor_archive_path = (
        repository / ".ai/work-items/archive" / f"{work_item_id}.archive.json"
    )
    predecessor_archive = load_regular_json(predecessor_archive_path)
    if (
        not predecessor_archive
        or predecessor_archive.get("workItemId") != work_item_id
        or predecessor_archive.get("state") != "archived"
    ):
        return False
    predecessor_files = predecessor_archive.get("files")
    if not isinstance(predecessor_files, dict):
        return False
    predecessor_paths = {
        "contract": f".ai/work-items/archive/{work_item_id}.contract.json",
        "summary": f".ai/work-items/archive/{work_item_id}.summary.json",
        "events": f".ai/work-items/archive/{work_item_id}.events.jsonl",
        "outcome": f".ai/work-items/archive/{work_item_id}.outcome.json",
    }
    predecessor_digests = {
        "contract": "predecessorContractDigest",
        "summary": "predecessorSummaryDigest",
        "events": "predecessorEventsDigest",
        "outcome": "predecessorOutcomeDigest",
    }

    for path in recovery_candidates(repository, work_item_id):
        recovery = load_regular_json(path)
        if (
            not recovery
            or not valid_recovery(path, work_item_id, repository_id)
            or recovery.get("decision") != "supersede"
            or not digest_named_recovery(path, recovery)
            or recovery.get("predecessorArchiveManifestDigest")
            != raw_file_digest(predecessor_archive_path)
        ):
            continue

        archive_bundle_valid = True
        for key, relative in predecessor_paths.items():
            record_path = repository / relative
            raw_digest = raw_file_digest(record_path)
            record = load_regular_json(record_path) if key != "events" else None
            expected_recovery_digest = (
                raw_digest if key == "events" else canonical_json_digest(record)
                if record is not None
                else None
            )
            manifest_path = predecessor_files.get(f"{key}Path")
            manifest_digest = predecessor_files.get(f"{key}Digest")
            recovery_digest = recovery.get(predecessor_digests[key])
            if (
                manifest_path != relative
                or raw_digest is None
                or manifest_digest != raw_digest
                or expected_recovery_digest is None
                or recovery_digest != expected_recovery_digest
            ):
                archive_bundle_valid = False
                break
        predecessor_contract = load_regular_json(
            repository / predecessor_paths["contract"]
        )
        if (
            not archive_bundle_valid
            or not predecessor_contract
            or predecessor_contract.get("workItemId") != work_item_id
            or predecessor_contract.get("repositoryId") != repository_id
        ):
            continue

        successor_id = recovery.get("successorWorkItemId")
        if not isinstance(successor_id, str) or not successor_id:
            continue
        successor_contract_path = (
            repository / ".ai/work-items/archive" / f"{successor_id}.contract.json"
        )
        successor_summary_path = (
            repository / ".ai/work-items/archive" / f"{successor_id}.summary.json"
        )
        successor_outcome_path = (
            repository / ".ai/work-items/archive" / f"{successor_id}.outcome.json"
        )
        successor_archive_path = (
            repository / ".ai/work-items/archive" / f"{successor_id}.archive.json"
        )
        verification_relative = f".ai/evidence/{successor_id}.verification.json"
        close_relative = f".ai/decisions/{successor_id}.close.json"
        recovery_relative = path.relative_to(repository).as_posix()
        required_references = {
            f".ai/work-items/archive/{work_item_id}.archive.json",
            close_relative,
            verification_relative,
        }
        recovery_refs = recovery.get("evidenceRefs")
        if (
            not isinstance(recovery_refs, list)
            or any(not isinstance(item, str) for item in recovery_refs)
            or not required_references.issubset(set(recovery_refs))
        ):
            continue

        successor_contract = load_regular_json(successor_contract_path)
        successor_archive = load_regular_json(successor_archive_path)
        successor_summary = load_regular_json(successor_summary_path)
        successor_recovery_relative = (
            successor_contract.get("recoveryDecisionPath")
            if successor_contract
            else None
        )
        successor_recovery_path = (
            repository / successor_recovery_relative
            if isinstance(successor_recovery_relative, str)
            else None
        )
        successor_recovery = (
            load_regular_json(successor_recovery_path)
            if successor_recovery_path is not None
            else None
        )
        recovery_chain_matches = bool(
            successor_recovery_path is not None
            and successor_recovery is not None
            and successor_recovery_path.parent == repository / ".ai/decisions"
            and valid_recovery(successor_recovery_path, work_item_id, repository_id)
            and digest_named_recovery(successor_recovery_path, successor_recovery)
            and successor_recovery.get("decision") in {"successor", "supersede"}
            and successor_recovery.get("successorWorkItemId") == successor_id
            and all(
                successor_recovery.get(key) == recovery.get(key)
                for key in (
                    "predecessorContractDigest",
                    "predecessorSummaryDigest",
                    "predecessorEventsDigest",
                    "predecessorOutcomeDigest",
                )
            )
            and successor_recovery.get("predecessorArchiveManifestDigest")
            in {None, recovery.get("predecessorArchiveManifestDigest")}
        )
        if (
            not successor_contract
            or successor_contract.get("workItemId") != successor_id
            or successor_contract.get("repositoryId") != repository_id
            or successor_contract.get("predecessorWorkItemId") != work_item_id
            or not recovery_chain_matches
            or not successor_archive
            or successor_archive.get("workItemId") != successor_id
            or successor_archive.get("state") != "archived"
            or not successor_summary
            or successor_summary.get("workItemId") != successor_id
            or successor_summary.get("repositoryId") != repository_id
        ):
            continue

        successor_files = successor_archive.get("files")
        if not isinstance(successor_files, dict) or any(
            successor_files.get(f"{key}Path") != relative
            or successor_files.get(f"{key}Digest") != raw_file_digest(repository / relative)
            for key, relative in (
                ("contract", f".ai/work-items/archive/{successor_id}.contract.json"),
                ("summary", f".ai/work-items/archive/{successor_id}.summary.json"),
                ("outcome", f".ai/work-items/archive/{successor_id}.outcome.json"),
            )
        ):
            continue

        verification = load_regular_json(repository / verification_relative)
        if not verification:
            continue
        contract_digest = verification.get("contractDigest")
        snapshot_digest = verification.get("repositorySnapshotDigest")
        receipt = verification.get("receipt")
        plan_receipt = receipt.get("planReceipt") if isinstance(receipt, dict) else None
        if (
            verification.get("passed") is not True
            or verification.get("workItemId") != successor_id
            or verification.get("repositoryId") != repository_id
            or not isinstance(contract_digest, str)
            or contract_digest
            not in {
                successor_summary.get("checkpointContractDigest"),
                successor_summary.get("preflightContractDigest"),
            }
            or not isinstance(snapshot_digest, str)
            or snapshot_digest
            not in {
                successor_summary.get("checkpointRepositorySnapshotDigest"),
                successor_summary.get("preflightRepositorySnapshotDigest"),
            }
            or not isinstance(receipt, dict)
            or receipt.get("passed") is not True
            or receipt.get("workItemId") != successor_id
            or receipt.get("repositoryId") != repository_id
            or verification.get("receiptDigest") != canonical_json_digest(receipt)
            or not isinstance(plan_receipt, dict)
            or plan_receipt.get("workItemId") != successor_id
            or plan_receipt.get("repositoryId") != repository_id
            or plan_receipt.get("repositorySnapshotDigest") != snapshot_digest
        ):
            continue

        close = load_regular_json(repository / close_relative)
        if not close or not valid_close(repository / close_relative, successor_id, repository_id):
            continue
        final_report = close.get("finalReport")
        bindings = final_report.get("bindings") if isinstance(final_report, dict) else None
        structured = close.get("structuredDecision")
        if (
            not isinstance(final_report, dict)
            or close.get("finalReportDigest") != canonical_json_digest(final_report)
            or final_report.get("status") != "verified"
            or final_report.get("humanStatusColor") != "green"
            or not isinstance(bindings, dict)
            or bindings.get("workItemId") != successor_id
            or bindings.get("repositoryId") != repository_id
            or verification_relative not in bindings.get("evidenceRefs", [])
            or not isinstance(structured, dict)
            or structured.get("decision") != "approved"
            or verification_relative not in structured.get("evidenceRefs", [])
        ):
            continue
        return True
    return False


def recovery_candidates(repository: Path, work_item_id: str) -> list[Path]:
    """Return the base and digest-suffixed recovery records in stable order."""
    decision_dir = repository / ".ai/decisions"
    return [decision_dir / f"{work_item_id}.recovery.json"] + sorted(
        decision_dir.glob(f"{work_item_id}.recovery.*.json")
    )


def has_valid_recovery(repository: Path, work_item_id: str, repository_id: str) -> bool:
    """Accept only a valid base record or a content-bound digest-suffixed record."""
    for path in recovery_candidates(repository, work_item_id):
        if not valid_recovery(path, work_item_id, repository_id):
            continue
        if path.name != f"{work_item_id}.recovery.json":
            value = load_regular_json(path)
            if value is None:
                continue
            digest = hashlib.sha256(
                json.dumps(
                    value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
                ).encode("utf-8")
            ).hexdigest()
            if path.name != f"{work_item_id}.recovery.{digest}.json":
                continue
        return True
    return False



def valid_retirement_receipt(
    repo: Path,
    work_item: str,
    receipt_path: Path,
    receipt: dict[str, Any],
) -> bool:
    """Validate Runtime retirement as historical cleanup, never verification."""

    try:
        project = load_regular_json(repo / ".ai/project.json")
        manifest_path = repo / ".ai/work-items/archive" / f"{work_item}.archive.json"
        manifest = load_regular_json(manifest_path)
    except ValueError:
        return False
    if project is None or manifest is None:
        return False
    digest_pattern = re.compile(r"sha256:[0-9a-f]{64}")
    nonempty = lambda key: isinstance(receipt.get(key), str) and bool(
        receipt[key].strip()
    )
    disposition = receipt.get("disposition")
    if disposition not in {"integrated", "replaced"}:
        return False
    successor = receipt.get("successorWorkItemId")
    if disposition == "replaced" and (not isinstance(successor, str) or not successor):
        return False
    if disposition == "integrated" and successor is not None:
        return False
    if not (
        receipt.get("schemaVersion") == 1
        and receipt.get("decisionId") == "work-item-retirement"
        and receipt.get("workItemId") == work_item
        and receipt.get("repositoryId") == project.get("repositoryId")
        and receipt.get("originalBytesPreserved") is True
        and receipt.get("verificationClaim") == "not_verified"
        and all(nonempty(key) for key in ("actor", "authoritySource", "reason", "recordedAt"))
        and digest_pattern.fullmatch(str(receipt.get("runtimeDigest", "")))
        and nonempty("runtimeVersion")
        and digest_pattern.fullmatch(str(receipt.get("archiveManifestDigest", "")))
    ):
        return False
    if canonical_json_digest(manifest) != receipt.get("archiveManifestDigest"):
        return False
    if not (
        manifest.get("workItemId") == work_item
        and manifest.get("state") in {"retired", "replaced"}
        and manifest.get("retirementDisposition") == disposition
        and manifest.get("retirementReceiptPath")
        == str(receipt_path.relative_to(repo))
        and manifest.get("closeRequired") is False
        and manifest.get("historicalEvidence") is True
    ):
        return False
    artifacts = receipt.get("artifacts")
    if not isinstance(artifacts, dict):
        return False
    for suffix in ("contract", "summary", "outcome"):
        artifact = artifacts.get(suffix)
        relative = f".ai/work-items/archive/{work_item}.{suffix}.json"
        path = repo / relative
        if not isinstance(artifact, dict) or artifact.get("path") != relative:
            return False
        digest = artifact.get("digest")
        if not isinstance(digest, str) or not digest_pattern.fullmatch(digest):
            return False
        if not path.is_file() or path.is_symlink():
            return False
        if "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            return False
    return True

def _native_idempotent_resource_close(
    repository: Path, work_item_id: str, repository_id: str,
    contract: dict[str, Any], close: dict[str, Any], root: dict[str, Any],
) -> bool:
    """Read the exact schema-v1 already-deleted observation, never delete again.

    The native protocol explicitly permits fully_deleted(before) as well as
    fresh cleanup (cockpit-protocol validate_resource_finalization_receipt).
    Reuse the existing concrete identity validator; keep this adapter confined
    to a terminal root, frozen by its actual close head and without transitions.
    The caller already checks the complete archive/formal/report/close tuple.
    """
    from promote_closed_work_item import PromotionError, receipt_identity
    context = contract.get("resourceContext")
    required_context = {"baseBranch", "baseRemote", "branch", "provider", "pullRequest", "worktree"}
    if (not isinstance(context, dict) or set(context) != required_context
            or any(not isinstance(value, str) or not value.strip()
                   or value in {"unknown", "pending"} or value.startswith("pending:")
                   for value in context.values())):
        return False
    root_relative = f".ai/decisions/{work_item_id}.finalize.json"
    fully_deleted = {"branch": "deleted", "pullRequest": "merged", "worktree": "removed"}
    pull_request = root.get("pullRequest")
    if (type(root.get("schemaVersion")) is not int or root.get("schemaVersion") != 1
            or root.get("postMergeSync") is not None
            or root.get("before") != fully_deleted or root.get("after") != fully_deleted
            or root.get("result") != {"disposition": "deleted", "failureCodes": [], "unknownCodes": []}
            or not isinstance(pull_request, dict)
            or type(pull_request.get("number")) is not int or pull_request["number"] <= 0
            or any(not isinstance(root.get(key), str) or not root[key].strip()
                   for key in ("actor", "authoritySource", "reason", "timestamp", "operationId", "receiptId", "runtimeVersion"))
            or not isinstance(root.get("runtimeDigest"), str)
            or re.fullmatch(r"sha256:[0-9a-f]{64}", root["runtimeDigest"]) is None
            or type(close.get("resourceFinalizationSequence")) is not int
            or close["resourceFinalizationSequence"] != 0
            or close.get("resourceFinalizationHeadPath") != root_relative
            or close.get("resourceFinalizationHeadDigest") != canonical_json_digest(root)
            or root_relative not in close["structuredDecision"].get("evidenceRefs", [])
            or any((repository / ".ai/decisions").glob(f"{work_item_id}.finalize.*.json"))):
        return False
    for key in ("baseRevision", "headRevision", "mergeCommit"):
        value = pull_request.get(key)
        if not isinstance(value, str) or re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", value) is None:
            return False
    if (not isinstance(root.get("branch"), dict) or not isinstance(root.get("worktree"), dict)
            or root["branch"].get("headRevision") != pull_request["headRevision"]
            or root["worktree"].get("headRevision") != pull_request["headRevision"]):
        return False
    base = contract.get("baseRevision")
    if not isinstance(base, str) or re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", base) is None:
        return False
    try:
        receipt_identity(
            root, work_item_id=work_item_id, repository_id=repository_id,
            contract_digest=raw_file_digest(repository / f".ai/work-items/archive/{work_item_id}.contract.json"),
            base_revision=base, resource_context=context,
        )
    except (PromotionError, KeyError, TypeError, ValueError, OSError):
        return False
    return True


def ordinary_closed_successor_state(
    repository: Path,
    predecessor: str,
    repository_id: str,
    recovery_path: Path,
    recovery: dict[str, Any],
) -> tuple[str, str] | None:
    """Describe one native-shaped ordinary-close lineage, never an issuer claim.

    An already verified/approved predecessor stays completed while its child
    is pending. A child reaches `closed` only with its own complete native
    archive/verification/actual-close bindings. Canonical first receipts are
    supported as written by Runtime; no alias or synthetic supersede is needed.
    """
    def regular(relative: str) -> Path | None:
        candidate = Path(relative)
        if candidate.is_absolute() or ".." in candidate.parts:
            return None
        path = repository / candidate
        try:
            if not path.is_file() or path.is_symlink():
                return None
            for parent in path.parents:
                if parent == repository:
                    break
                if parent.is_symlink():
                    return None
            else:
                return None
            if path.stat().st_size > 1_048_576:
                return None
        except OSError:
            return None
        return path

    def strict(relative: str) -> dict[str, Any] | None:
        path = regular(relative)
        if path is None:
            return None
        def unique(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
            result: dict[str, Any] = {}
            for key, value in pairs:
                if key in result:
                    raise ValueError("duplicate governance key")
                result[key] = value
            return result
        try:
            value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique)
        except (OSError, UnicodeError, ValueError):
            return None
        return value if isinstance(value, dict) else None

    def bundle(wi: str) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]] | None:
        manifest = strict(f".ai/work-items/archive/{wi}.archive.json")
        if (not manifest or manifest.get("workItemId") != wi
                or ("repositoryId" in manifest and manifest["repositoryId"] != repository_id)
                or manifest.get("state") != "archived"
                or not isinstance(manifest.get("files"), dict)):
            return None
        files = manifest["files"]
        records: dict[str, dict[str, Any]] = {}
        for key, ending in (("contract", "contract.json"), ("summary", "summary.json"),
                            ("outcome", "outcome.json"), ("events", "events.jsonl")):
            relative = f".ai/work-items/archive/{wi}.{ending}"
            path = regular(relative)
            if (path is None or files.get(key + "Path") != relative
                    or files.get(key + "Digest") != raw_file_digest(path)):
                return None
            if key != "events":
                value = strict(relative)
                if not value or value.get("workItemId") != wi:
                    return None
                records[key] = value
        if any(records[key].get("repositoryId") != repository_id for key in ("contract", "summary")):
            return None
        return manifest, records["contract"], records["summary"], records["outcome"]

    def complete(wi: str, archived: tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]], selected_recovery: str | None = None, selected_successor: str | None = None) -> str | None:
        verification = strict(f".ai/evidence/{wi}.verification.json")
        close = strict(f".ai/decisions/{wi}.close.json")
        if not verification or not close:
            return None
        structured = close.get("structuredDecision")
        if (close.get("humanDecision") not in {"approved", "confirmed"} or not isinstance(structured, dict)
                or structured.get("decision") != close.get("humanDecision")
                or any(not isinstance(structured.get(key), str) or not structured[key].strip()
                       for key in ("actor", "authoritySource", "reason", "decidedAt"))):
            return None
        receipt = verification.get("receipt")
        plan = receipt.get("planReceipt") if isinstance(receipt, dict) else None
        summary = archived[2]
        runtime = verification.get("runtimeDigest")
        if (not isinstance(runtime, str) or not re.fullmatch(r"sha256:[0-9a-f]{64}", runtime)
                or not isinstance(receipt, dict) or receipt.get("runtimeDigest") != runtime
                or verification.get("contractDigest") != canonical_json_digest(archived[1])
                or type(summary.get("checkpointCount")) is not int
                or summary.get("checkpointCount") != 1
                or summary.get("state") != "finish_ready"
                or summary.get("preflightState") != "green"
                or not isinstance(plan, dict)):
            return None
        bound_close = _verified_native_close_record(repository, wi, repository_id, archived)
        if bound_close is None or bound_close != close:
            return None
        references = structured["evidenceRefs"]
        if any(key in close for key in ("ordinaryCleanupBinding", "ordinaryCleanupBindingDigest")):
            if not ordinary_cleanup_tuple_matches(repository, wi, repository_id, archived[1], close):
                return None
            if f".ai/evidence/{wi}.verification.json" in references:
                return "direct"
            # Native close accepts digest_json(envelope), whereas its archive
            # approval binds exact manifest bytes. Do not substitute raw JSON
            # formatting bytes for the canonical verification envelope digest.
            if (canonical_json_digest(verification) in references
                    and raw_file_digest(repository / f".ai/work-items/archive/{wi}.archive.json") in references):
                return "direct"
            # This is correspondence to an already recorded native human
            # decision, not authentication or a grant from a citation JSON.
            for reference in references:
                if (not isinstance(reference, str)
                        or not reference.startswith(f".ai/evidence/{wi}.human-close-")
                        or not reference.endswith(".json")):
                    continue
                citation = strict(reference)
                bindings = citation.get("bindings") if citation else None
                statement = citation.get("statement") if citation else None
                if (not citation or type(citation.get("schemaVersion")) is not int
                        or citation["schemaVersion"] != 1 or not isinstance(bindings, dict)
                        or not isinstance(statement, str) or not statement.strip()):
                    continue
                statement_digest = hashlib.sha256(statement.encode("utf-8")).hexdigest()
                if (citation.get("actor") == structured["actor"]
                        and citation.get("authoritySource") == structured["authoritySource"]
                        and citation.get("statementUtf8Sha256") == statement_digest
                        and "sha256:" + statement_digest in structured["authoritySource"]
                        and bindings.get("workItemId") == wi
                        and bindings.get("repositoryId") == repository_id
                        and bindings.get("contractDigest") == canonical_json_digest(archived[1])
                        and "sha256:" + str(bindings.get("archiveRawSha256")) == raw_file_digest(
                            repository / f".ai/work-items/archive/{wi}.archive.json")):
                    return "direct"
            if (selected_recovery is not None and selected_successor is not None
                    and selected_recovery in references
                    and f".ai/decisions/{selected_successor}.close.json" in references):
                return "successor_close"
            return None
        # Resource closes bind the native finalization head/root, not a new
        # ordinary path requirement. Reuse the complete existing chain validator.
        from promote_closed_work_item import PromotionError, validate_terminal_evidence
        try:
            terminal = validate_terminal_evidence(repository, wi)
        except (PromotionError, OSError, ValueError, KeyError, TypeError):
            root = strict(f".ai/decisions/{wi}.finalize.json")
            return "resource" if root and _native_idempotent_resource_close(
                repository, wi, repository_id, archived[1], close, root
            ) else None
        if terminal.finalization_path is not None:
            # The complete native validator already checked the exact close
            # head path/sequence/digest. Human citations need not repeat it.
            return "resource"
        return None

    predecessor_bundle = bundle(predecessor)
    if predecessor_bundle is None:
        return None
    try:
        relative = recovery_path.relative_to(repository).as_posix()
    except ValueError:
        return None
    if recovery_path.parent != repository / ".ai/decisions":
        return None
    exact_recovery = strict(relative)
    if exact_recovery != recovery or not isinstance(exact_recovery, dict):
        return None
    required_fields = {
        "schemaVersion", "decisionId", "decision", "workItemId", "repositoryId",
        "predecessorWorkItemId", "predecessorContractDigest", "predecessorSummaryDigest",
        "successorWorkItemId", "runtimeVersion", "runtimeDigest", "actor",
        "authoritySource", "reason", "evidenceRefs", "policyRefs", "decidedAt", "resumeCondition",
    }
    allowed_fields = required_fields | {
        "predecessorOutcomeDigest", "predecessorEventsDigest", "predecessorArchiveManifestDigest",
        "predecessorFinalizationContractDigest", "successorBindingMode", "currentContractDigest",
        "predecessorVerificationEvidenceDigest",
    }
    if (not required_fields <= set(recovery) or set(recovery) - allowed_fields
            or not isinstance(recovery.get("evidenceRefs"), list)
            or not recovery["evidenceRefs"]
            or any(not isinstance(value, str) or not value.strip() for value in recovery["evidenceRefs"])
            or not isinstance(recovery.get("policyRefs"), list)
            or any(not isinstance(value, str) for value in recovery["policyRefs"])
            or recovery.get("predecessorFinalizationContractDigest") is not None):
        return None
    for key in ("predecessorContractDigest", "predecessorSummaryDigest", "predecessorOutcomeDigest",
                "predecessorEventsDigest", "predecessorArchiveManifestDigest"):
        value = recovery.get(key)
        if value is not None and (not isinstance(value, str) or not re.fullmatch(r"sha256:[0-9a-f]{64}", value)):
            return None
    canonical = f".ai/decisions/{predecessor}.recovery.json"
    if relative != canonical and not digest_named_recovery(recovery_path, exact_recovery):
        return None
    successor = recovery.get("successorWorkItemId")
    if (type(recovery.get("schemaVersion")) is not int or recovery.get("schemaVersion") != 1
            or recovery.get("decisionId") != "work-item-recovery"
            or recovery.get("decision") != "successor"
            or recovery.get("workItemId") != predecessor
            or recovery.get("predecessorWorkItemId") != predecessor
            or recovery.get("repositoryId") != repository_id
            or not isinstance(successor, str)
            or not re.fullmatch(r"WI-[0-9]+[A-Za-z]?(?:-[A-Za-z0-9-]+)?", successor)
            or successor == predecessor
            or any(not isinstance(recovery.get(key), str) or not recovery[key].strip()
                   for key in ("actor", "authoritySource", "reason", "decidedAt", "resumeCondition", "runtimeVersion"))
            or not isinstance(recovery.get("runtimeDigest"), str)
            or not re.fullmatch(r"sha256:[0-9a-f]{64}", recovery["runtimeDigest"])
            or any(recovery.get(key) is not None for key in
                   ("successorBindingMode", "currentContractDigest", "predecessorVerificationEvidenceDigest"))
            or not recovery_binds_archive_bundle_digests(repository, predecessor, recovery, predecessor_bundle)):
        return None
    parent_close_binding = complete(predecessor, predecessor_bundle, relative, successor)
    if parent_close_binding is None:
        return None
    recovery_references_parent_close = (
        f".ai/decisions/{predecessor}.close.json" in recovery["evidenceRefs"]
    )
    recovery_references_parent_archive = (
        f".ai/work-items/archive/{predecessor}.archive.json" in recovery["evidenceRefs"]
    )
    # No competing selected successor or supersede may be hidden by this path.
    for other_path in recovery_candidates(repository, predecessor):
        if other_path == recovery_path:
            continue
        try:
            other_relative = other_path.relative_to(repository).as_posix()
        except ValueError:
            return None
        try:
            other_path.lstat()
        except FileNotFoundError:
            if other_relative == canonical:
                continue  # only the genuinely absent canonical candidate is optional
            return None
        except OSError:
            return None
        other = strict(other_relative)
        if (other is None
                or other.get("workItemId") != predecessor
                or other.get("predecessorWorkItemId") != predecessor
                or other.get("repositoryId") != repository_id
                or other.get("decision") not in {"retry", "successor", "supersede"}
                or (other_relative != canonical and not digest_named_recovery(other_path, other))
                or other.get("decision") in {"successor", "supersede"}):
            return None
    locations = []
    for location in ("active", "archive"):
        prefix = f".ai/work-items/{location}/{successor}"
        c_path, s_path = repository / (prefix + ".contract.json"), repository / (prefix + ".summary.json")
        if c_path.exists() or c_path.is_symlink() or s_path.exists() or s_path.is_symlink():
            contract, summary = strict(prefix + ".contract.json"), strict(prefix + ".summary.json")
            if not contract or not summary:
                return None
            locations.append((location, contract, summary))
    if len(locations) != 1:
        return None
    location, contract, summary = locations[0]
    if any(record.get("workItemId") != successor or record.get("repositoryId") != repository_id
           or record.get("predecessorWorkItemId") != predecessor
           or record.get("predecessorContractDigest") != recovery["predecessorContractDigest"]
           or record.get("recoveryDecisionPath") != relative
           for record in (contract, summary)):
        return None
    close_path = repository / f".ai/decisions/{successor}.close.json"
    retirement_relative = f".ai/decisions/{successor}.retirement.json"
    retirement_path = repository / retirement_relative
    manifest_relative = f".ai/work-items/archive/{successor}.archive.json"
    manifest = strict(manifest_relative) if location == "archive" else None
    if location == "archive" and (
        retirement_path.exists() or retirement_path.is_symlink()
        or (manifest and manifest.get("state") in {"retired", "replaced"})
    ):
        # A retired child cannot supply a successful close. Its exact native
        # retirement permits only the predecessor's already accepted result
        # to remain closed, with the child reported separately as nonverified.
        retirement = strict(retirement_relative)
        if (parent_close_binding == "successor_close"
                or close_path.exists() or close_path.is_symlink() or not manifest or not retirement
                or type(retirement.get("schemaVersion")) is not int
                or retirement.get("disposition") != "replaced"
                or manifest.get("state") != "replaced"
                or not valid_retirement_receipt(repository, successor, retirement_path, retirement)
                or retirement.get("contractDigest") != canonical_json_digest(contract)
                or retirement.get("summaryDigest") != canonical_json_digest(summary)
                or manifest.get("successorWorkItemId") != retirement.get("successorWorkItemId")):
            return None
        files = manifest.get("files")
        if not isinstance(files, dict):
            return None
        for key in ("contract", "summary", "outcome"):
            artifact_relative = f".ai/work-items/archive/{successor}.{key}.json"
            artifact_path = regular(artifact_relative)
            artifact = strict(artifact_relative)
            if (artifact_path is None or not artifact or artifact.get("workItemId") != successor
                    or files.get(key + "Path") != artifact_relative
                    or files.get(key + "Digest") != raw_file_digest(artifact_path)):
                return None
        if not (recovery_references_parent_close and recovery_references_parent_archive):
            # A historical recovery may cite only external observations and
            # cannot cite a future close. All four parent digests still match.
            # This narrow case is only for a complete nonverified retirement
            # was just checked above, not an ordinary pending/closed child.
            parent_close = strict(f".ai/decisions/{predecessor}.close.json")
            if not parent_close:
                return None
            try:
                recovery_at, decision_at, recorded_at = (
                    datetime.fromisoformat(value.replace("Z", "+00:00"))
                    for value in (
                        recovery["decidedAt"],
                        parent_close["structuredDecision"]["decidedAt"],
                        parent_close.get("recordedAt"),
                    )
                )
            except (AttributeError, TypeError, ValueError):
                return None
            if (any(value.tzinfo is None for value in (recovery_at, decision_at, recorded_at))
                    or not recovery_at < decision_at <= recorded_at):
                return None
        return "retired", successor
    references = recovery["evidenceRefs"]
    artifact_paths = {
        f".ai/work-items/archive/{predecessor}.{suffix}"
        for suffix in ("contract.json", "summary.json", "outcome.json", "events.jsonl")
    }
    artifact_set_bound = artifact_paths <= set(references)
    resource_archive_bound = (
        parent_close_binding == "resource" and recovery_references_parent_archive
        and f".ai/work-items/archive/{predecessor}.outcome.json" in references
    )
    same_native_attempt_bound = False
    if parent_close_binding == "successor_close":
        attempts = [reference for reference in references if re.fullmatch(
            rf"\.ai/evidence/{re.escape(predecessor)}\.verification-attempt\.[0-9a-f]{{64}}\.json", reference
        )]
        if len(attempts) == 1:
            attempt = strict(attempts[0])
            formal = strict(f".ai/evidence/{predecessor}.verification.json")
            expected_id = "sha256:" + attempts[0].rsplit(".", 2)[-2]
            same_native_attempt_bound = bool(
                attempt and formal
                and type(attempt.get("schemaVersion")) is int and attempt["schemaVersion"] == 1
                and type(attempt.get("protocolVersion")) is int and attempt["protocolVersion"] == 1
                and attempt.get("kind") == "verification_attempt"
                and attempt.get("attemptId") == expected_id
                and attempt.get("state") == "execution_completed"
                and attempt.get("passed") is True
                and attempt.get("workItemId") == predecessor
                and attempt.get("repositoryId") == repository_id
                and attempt.get("contractDigest") == canonical_json_digest(predecessor_bundle[1])
                and attempt.get("repositorySnapshotDigest") == formal.get("repositorySnapshotDigest")
                and attempt.get("runtimeDigest") == formal.get("runtimeDigest")
                and isinstance(attempt.get("receipt"), dict)
                and canonical_json_digest(attempt["receipt"]) == formal.get("receiptDigest")
                and f".ai/work-items/archive/{predecessor}.outcome.json" in references
            )
    if not ((recovery_references_parent_close and recovery_references_parent_archive)
            or artifact_set_bound or resource_archive_bound or same_native_attempt_bound):
        return None  # opaque strings/URLs alone never establish a normal binding
    verification_path = repository / f".ai/evidence/{successor}.verification.json"
    verification_exists = verification_path.exists() or verification_path.is_symlink()
    verification = strict(f".ai/evidence/{successor}.verification.json") if verification_exists else None
    if verification_exists:
        receipt = verification.get("receipt") if verification else None
        plan = receipt.get("planReceipt") if isinstance(receipt, dict) else None
        if (not verification or type(verification.get("passed")) is not bool
                or verification.get("workItemId") != successor
                or verification.get("repositoryId") != repository_id
                or verification.get("contractDigest") != canonical_json_digest(contract)
                or verification.get("repositorySnapshotDigest") not in {
                    summary.get("checkpointRepositorySnapshotDigest"), summary.get("preflightRepositorySnapshotDigest")}
                or not isinstance(receipt, dict) or verification.get("receiptDigest") != canonical_json_digest(receipt)
                or receipt.get("workItemId") != successor or receipt.get("repositoryId") != repository_id
                or receipt.get("passed") is not verification["passed"]
                or receipt.get("runtimeDigest") != verification.get("runtimeDigest")
                or not isinstance(plan, dict) or plan.get("workItemId") != successor
                or plan.get("repositoryId") != repository_id
                or plan.get("repositorySnapshotDigest") != verification.get("repositorySnapshotDigest")):
            return None
    if location == "active":
        if parent_close_binding == "successor_close":
            return None  # this native decision explicitly relies on a closed child
        if close_path.exists() or close_path.is_symlink():
            return None
        if any(str(record.get(key, "")).lower() in {"archived", "closed", "complete", "completed", "implemented", "recovered", "terminal"}
               for record in (contract, summary)
               for key in ("state", "lifecycleState", "lifecyclePhase", "archiveState", "closeState")):
            return None
        return "pending", successor
    successor_bundle = bundle(successor)
    if successor_bundle is None or not verification or verification.get("passed") is not True:
        return None
    if close_path.exists() or close_path.is_symlink():
        return ("closed", successor) if complete(successor, successor_bundle) else None
    # A close explicitly citing its selected child cannot be satisfied by pending.
    if parent_close_binding == "successor_close":
        return None
    # A genuinely archived/verified child still awaiting its human close is pending.
    return "pending", successor


def ordinary_closed_projection_continuation(
    repository: Path, predecessor: str, repository_id: str
) -> str | None:
    """Return the exact docs-only successor owning this conditional projection."""
    for recovery_path in recovery_candidates(repository, predecessor):
        recovery = load_regular_json(recovery_path)
        if not recovery:
            continue
        state = ordinary_closed_successor_state(repository, predecessor, repository_id, recovery_path, recovery)
        if state is None or state[0] == "retired":
            continue  # retirement never grants a docs-only continuation exemption
        _, successor = state
        if not bounded_documentation_projection(repository, successor):
            # This helper requires an archived docs-only Contract; an active
            # child is checked against the identical exact scope shape below.
            contract_path = repository / f".ai/work-items/active/{successor}.contract.json"
        else:
            contract_path = repository / f".ai/work-items/archive/{successor}.contract.json"
        contract = load_regular_json(contract_path)
        if not contract:
            continue
        own = {f"docs/work-items/{successor}{suffix}.md" for suffix in ("", ".zh-CN", ".ja")}
        predecessor_docs = {f"docs/work-items/{predecessor}{suffix}.md" for suffix in ("", ".zh-CN", ".ja")}
        ledgers = {path for path, _ in PARITY_DOCUMENTS}
        scope = contract.get("scope")
        if (isinstance(scope, list) and all(isinstance(item, str) for item in scope)
                and own | predecessor_docs | ledgers <= set(scope)
                and all(not any(mark in item for mark in ("*", "?", "[", "]", "..", "\\"))
                        and (item in ledgers or (item.startswith("docs/work-items/") and item.endswith(".md")))
                        for item in scope)):
            return successor
    return None

def valid_pending_successor(
    repository: Path, work_item_id: str, repository_id: str
) -> bool:
    """Recognize an identity-bound successor without upgrading it to terminal.

    A successor decision authorizes continued work only.  It is not evidence
    that the archived predecessor has been superseded or closed.
    """
    if valid_close(
        repository / ".ai/decisions" / f"{work_item_id}.close.json",
        work_item_id,
        repository_id,
    ):
        return False
    for path in recovery_candidates(repository, work_item_id):
        recovery = load_regular_json(path)
        if (
            not recovery
            or not valid_recovery(path, work_item_id, repository_id)
            or recovery.get("decision") != "successor"
        ):
            continue
        if path.name != f"{work_item_id}.recovery.json" and not digest_named_recovery(
            path, recovery
        ):
            continue
        successor_id = recovery.get("successorWorkItemId")
        if not isinstance(successor_id, str) or not successor_id:
            continue
        relative_path = path.relative_to(repository).as_posix()
        for location in ("active", "archive"):
            contract = load_regular_json(
                repository / ".ai/work-items" / location / f"{successor_id}.contract.json"
            )
            summary = load_regular_json(
                repository / ".ai/work-items" / location / f"{successor_id}.summary.json"
            )
            if (
                contract
                and summary
                and contract.get("workItemId") == successor_id
                and contract.get("repositoryId") == repository_id
                and contract.get("predecessorWorkItemId") == work_item_id
                and contract.get("predecessorContractDigest")
                == recovery.get("predecessorContractDigest")
                and contract.get("recoveryDecisionPath") == relative_path
                and summary.get("workItemId") == successor_id
                and summary.get("repositoryId") == repository_id
                and summary.get("predecessorWorkItemId") == work_item_id
                and summary.get("predecessorContractDigest")
                == recovery.get("predecessorContractDigest")
                and summary.get("recoveryDecisionPath") == relative_path
            ):
                return True
    return False


def verifier_is_authoritative(
    repository: Path, verifier: str, work_item_id: str, repository_id: str
) -> bool | None:
    if not verifier.startswith("WI-"):
        return None
    work_item_short = short_id(work_item_id)
    if verifier == work_item_id or (
        work_item_short is not None and verifier.startswith(f"{work_item_short}-")
    ):
        contract = repository / ".ai/work-items/archive" / f"{work_item_id}.contract.json"
        return valid_contract(contract, work_item_id, repository_id)
    if not re.fullmatch(r"WI-[0-9]+[A-Za-z]?(?:-[A-Za-z0-9-]+)?", verifier):
        return False
    for location in ("active", "archive"):
        contract = repository / ".ai/work-items" / location / f"{verifier}.contract.json"
        if valid_contract(contract, verifier, repository_id):
            return True
    return False


def check(repository: Path) -> list[str]:
    errors: list[str] = []
    project = load_regular_json(repository / ".ai/project.json")
    repository_id = project.get("repositoryId") if project else None
    if not isinstance(repository_id, str) or not repository_id:
        return [".ai/project.json: repositoryId is missing or invalid"]
    try:
        projection_policy = load_projection_policy(repository)
    except ProjectionPolicyError as error:
        return [str(error)]

    rows, parity_errors = parity_statuses(repository)
    errors.extend(parity_errors)
    work_item_directory = repository / "docs/work-items"
    for english in sorted(work_item_directory.glob("WI-*.md")):
        if english.name.endswith((".zh-CN.md", ".ja.md")):
            continue
        english_fields = frontmatter(english)
        if not english_fields or "workItemId" not in english_fields:
            continue
        work_item_id = english_fields["workItemId"]
        short = short_id(work_item_id)
        if short is None:
            errors.append(f"{english.relative_to(repository)}: invalid workItemId {work_item_id}")
            continue

        documents = (
            english,
            english.with_name(f"{english.stem}.zh-CN.md"),
            english.with_name(f"{english.stem}.ja.md"),
        )
        fields: list[dict[str, str]] = []
        missing = False
        for document in documents:
            value = frontmatter(document)
            if value is None:
                errors.append(f"{document.relative_to(repository)}: missing or malformed frontmatter")
                missing = True
            else:
                fields.append(value)
        if missing:
            continue
        ids = [value.get("workItemId") for value in fields]
        if ids != [work_item_id] * 3:
            errors.append(f"{english.relative_to(repository)}: three-language workItemId mismatch")
            continue
        statuses = [value.get("status") for value in fields]
        if len(set(statuses)) != 1:
            errors.append(f"{english.relative_to(repository)}: three-language status mismatch: {statuses}")
            continue
        verifiers = [value.get("lastVerifiedBy") for value in fields]
        if len(set(verifiers)) != 1:
            errors.append(f"{english.relative_to(repository)}: three-language lastVerifiedBy mismatch: {verifiers}")
            continue

        archive_contract = repository / ".ai/work-items/archive" / f"{work_item_id}.contract.json"
        contract = load_regular_json(archive_contract)
        if (
            not contract
            or contract.get("workItemId") != work_item_id
            or contract.get("repositoryId") != repository_id
        ):
            continue
        close = repository / ".ai/decisions" / f"{work_item_id}.close.json"
        has_close = valid_close(close, work_item_id, repository_id)
        has_recovery = has_valid_recovery(repository, work_item_id, repository_id)
        if not (has_close or has_recovery):
            continue

        try:
            if contract_predates_projection_policy(
                contract, work_item_id, projection_policy
            ):
                # Keep the historical three-language record structurally
                # consistent, but do not reinterpret its parity status under
                # a repository projection policy introduced later.
                continue
        except ProjectionPolicyError as error:
            errors.append(f"{english.relative_to(repository)}: {error}")
            continue

        parity = parity_statuses_for_work_item(rows, work_item_id)
        if "conditional" in parity:
            if bounded_documentation_projection(repository, work_item_id):
                # The current documentation-promotion Work Item is itself the
                # bounded projection boundary; its pre-archive conditional
                # row is intentional and must not spawn another successor.
                continue
            if valid_terminal_supersession(repository, work_item_id, repository_id):
                verifier = verifiers[0]
                verifier_authority = (
                    verifier_is_authoritative(
                        repository, verifier, work_item_id, repository_id
                    )
                    if isinstance(verifier, str)
                    else False
                )
                if verifier_authority is False:
                    errors.append(
                        f"{english.relative_to(repository)}: lastVerifiedBy {verifier!r} "
                        "does not bind an active or archived Contract"
                    )
                # Preserve the historical conditional row. Only a digest-bound
                # successor with passing verification and confirmed close
                # resolves it; no documentation-only successor is required.
                continue
            ordinary_successor = ordinary_closed_projection_continuation(repository, work_item_id, repository_id)
            if ordinary_successor is not None:
                # Only this new docs-only continuation exemption requires a
                # complete uniform projection; legacy status handling below
                # remains unchanged outside this narrow branch.
                if len(parity) != 3 or len(set(parity)) != 1:
                    errors.append(f"{english.relative_to(repository)}: missing or mixed three-language parity status")
                    continue
                # A valid unrelated WI is not this projection's verifier.
                verifier = verifiers[0]
                if verifier not in {work_item_id, ordinary_successor}:
                    errors.append(f"{english.relative_to(repository)}: lastVerifiedBy {verifier!r} is not the predecessor or its exact docs successor")
                continue
            if valid_pending_successor(repository, work_item_id, repository_id):
                # A successor is a continuation, not a confirmed close. Keep
                # the conditional human projection until terminal evidence is
                # independently established.
                continue
            errors.append(
                f"{english.relative_to(repository)}: terminal Work Item retains conditional parity status"
            )
            continue
        if len(parity) != 3 or len(set(parity)) != 1:
            continue

        expected = parity[0]
        allowed = (
            {"historical", "recovered", "implemented"}
            if expected == "mixed"
            else ({"historical", "recovered"} if expected == "recovered" else {"implemented"})
        )
        english_text = english.read_text(encoding="utf-8")
        if (
            expected == "implemented"
            and has_recovery
            and re.search(r"\bimmutable\s+recovery\b", english_text, re.IGNORECASE)
        ):
            allowed.add("recovered")
        if statuses[0] not in allowed:
            errors.append(
                f"{english.relative_to(repository)}: status {statuses[0]!r}; expected one of "
                f"{','.join(sorted(allowed))} "
                "from authoritative parity and terminal decision"
            )
        verifier = verifiers[0]
        verifier_authority = (
            verifier_is_authoritative(repository, verifier, work_item_id, repository_id)
            if isinstance(verifier, str)
            else False
        )
        if verifier_authority is False:
            errors.append(
                f"{english.relative_to(repository)}: lastVerifiedBy {verifier!r} "
                "does not bind an active or archived Contract"
            )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True, type=Path)
    args = parser.parse_args()
    errors = check(args.repo.resolve())
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("work item status consistency passed")
    return 0


if __name__ == "__main__":
    getattr(sys, "exit")(main())
