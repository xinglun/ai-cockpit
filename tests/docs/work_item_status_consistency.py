#!/usr/bin/env python3
"""Fail closed when terminal Work Item documentation drifts from repository truth."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


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
            match = re.match(r"^\|\s*(WI-[0-9]+[A-Za-z]?)(?=\s|—|\|)", line, re.IGNORECASE)
            if not match:
                continue
            cells = [cell.strip() for cell in line.split("|")[1:-1]]
            if len(cells) < 2:
                continue
            status = vocabulary.get(cells[1])
            if status is None and "→" in cells[1] and any(
                marker in cells[1].lower()
                for marker in ("close", "关闭", "verified close 後")
            ):
                status = "conditional"
            if status is None:
                continue
            work_item = match.group(1).upper()
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


def valid_close(path: Path, work_item_id: str, repository_id: str) -> bool:
    value = load_regular_json(path)
    return bool(
        value
        and value.get("workItemId") == work_item_id
        and value.get("repositoryId") == repository_id
        and value.get("state") == "closed"
        and value.get("decisionState") == "confirmed"
        and value.get("humanDecision") in {"approved", "superseded"}
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
        if not valid_contract(archive_contract, work_item_id, repository_id):
            continue
        close = repository / ".ai/decisions" / f"{work_item_id}.close.json"
        recovery = repository / ".ai/decisions" / f"{work_item_id}.recovery.json"
        has_close = valid_close(close, work_item_id, repository_id)
        has_recovery = valid_recovery(recovery, work_item_id, repository_id)
        if not (has_close or has_recovery):
            continue

        parity = rows.get(short, [])
        if "conditional" in parity:
            errors.append(
                f"{english.relative_to(repository)}: terminal Work Item retains conditional parity status"
            )
            continue
        if len(parity) != 3 or len(set(parity)) != 1:
            continue

        expected = parity[0]
        allowed = {"historical", "recovered"} if expected == "recovered" else {"implemented"}
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
