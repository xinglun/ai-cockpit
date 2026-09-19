#!/usr/bin/env python3
"""Fail closed when terminal Work Item documentation drifts from repository truth."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
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
    ("docs/reference/reference-parity.ja.md", {"Implemented": "implemented", "Recovered": "recovered"}),
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
        r"^(WI-[0-9]+[A-Za-z]?(?:-[A-Za-z0-9][A-Za-z0-9-]*)?)(?=\s|—|\|)",
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
                for marker in ("close", "关闭", "verified close 後")
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


def recovery_binds_archive_bundle(
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
    references = recovery.get("evidenceRefs")
    return (
        isinstance(references, list)
        and all(isinstance(reference, str) for reference in references)
        and f".ai/work-items/archive/{work_item_id}.archive.json" in references
        and manifest.get("workItemId") == work_item_id
    )


def terminal_successor_is_verified_and_closed(
    repository: Path,
    work_item_id: str,
    repository_id: str,
    bundle: tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]],
) -> bool:
    _, _, summary, _ = bundle
    verification_relative = f".ai/evidence/{work_item_id}.verification.json"
    verification = load_regular_json(repository / verification_relative)
    if not verification:
        return False
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
        return False
    close_relative = f".ai/decisions/{work_item_id}.close.json"
    close = load_regular_json(repository / close_relative)
    if not close or not valid_close(repository / close_relative, work_item_id, repository_id):
        return False
    final_report = close.get("finalReport")
    bindings = final_report.get("bindings") if isinstance(final_report, dict) else None
    structured = close.get("structuredDecision")
    return bool(
        isinstance(final_report, dict)
        and close.get("finalReportDigest") == canonical_json_digest(final_report)
        and final_report.get("status") == "verified"
        and final_report.get("humanStatusColor") == "green"
        and isinstance(bindings, dict)
        and bindings.get("workItemId") == work_item_id
        and bindings.get("repositoryId") == repository_id
        and verification_relative in bindings.get("evidenceRefs", [])
        and isinstance(structured, dict)
        and structured.get("decision") == "approved"
        and verification_relative in structured.get("evidenceRefs", [])
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
