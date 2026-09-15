#!/usr/bin/env python3
"""Shared project policy for selecting required Work Item document projections."""

from __future__ import annotations

import fnmatch
import json
import re
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


POLICY_PATH = Path(".ai/project/documentation-policy.json")
CAPABILITIES_PATH = Path(".ai/project/capabilities.json")
PROJECT_PATH = Path(".ai/project.json")
DEFAULT_REQUIRED_MODES = ("docs", "documentation", "release")
DEFAULT_REQUIRED_OPERATIONS = ("documentation.modify", "release.publish")
PROJECTION_PATHS = (
    "docs/work-items/{work_item}.md",
    "docs/work-items/{work_item}.zh-CN.md",
    "docs/work-items/{work_item}.ja.md",
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
)
WORK_ITEM_PATTERN = re.compile(r"^WI-[0-9]+[A-Za-z]?(?:-[A-Za-z0-9][A-Za-z0-9-]*)?$")


class ProjectionPolicyError(ValueError):
    """Raised when project documentation policy is malformed or misbound."""


@dataclass(frozen=True)
class ProjectionPolicy:
    repository_id: str
    default_projection: str
    required_modes: tuple[str, ...]
    required_operations: tuple[str, ...]
    preserve_existing_registrations: bool
    effective_from_contract_created_at: datetime | None = None


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ProjectionPolicyError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _read_object(path: Path) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ProjectionPolicyError(f"{path.as_posix()}: must be a regular non-symlink file")
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=_reject_duplicate_keys
        )
    except (OSError, json.JSONDecodeError) as error:
        raise ProjectionPolicyError(f"{path.as_posix()}: malformed JSON: {error}") from error
    if not isinstance(value, dict):
        raise ProjectionPolicyError(f"{path.as_posix()}: expected a JSON object")
    return value


def _parse_timestamp(value: str, label: str) -> datetime:
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise ProjectionPolicyError(f"{label}: invalid timestamp") from error
    if parsed.tzinfo is None or parsed.utcoffset() is None:
        raise ProjectionPolicyError(f"{label}: timestamp must include a timezone")
    return parsed.astimezone(timezone.utc)


def load_projection_policy(repository: Path) -> ProjectionPolicy:
    """Load a repository-bound policy; absent policy defaults to derived pages."""
    project = _read_object(repository / PROJECT_PATH)
    repository_id = project.get("repositoryId")
    if not isinstance(repository_id, str) or not repository_id:
        raise ProjectionPolicyError(f"{PROJECT_PATH.as_posix()}: repositoryId is missing")

    path = repository / POLICY_PATH
    if not path.exists() and not path.is_symlink():
        return ProjectionPolicy(
            repository_id=repository_id,
            default_projection="derived",
            required_modes=DEFAULT_REQUIRED_MODES,
            required_operations=DEFAULT_REQUIRED_OPERATIONS,
            preserve_existing_registrations=True,
        )

    value = _read_object(path)
    base_fields = {
        "schemaVersion",
        "repositoryId",
        "defaultProjection",
        "requiredModes",
        "requiredOperations",
        "preserveExistingRegistrations",
    }
    schema_version = value.get("schemaVersion")
    if schema_version == 1:
        expected_fields = base_fields
        effective_from = None
    elif schema_version == 2:
        expected_fields = base_fields | {"effectiveFromContractCreatedAt"}
        raw_effective_from = value.get("effectiveFromContractCreatedAt")
        if not isinstance(raw_effective_from, str) or not raw_effective_from:
            raise ProjectionPolicyError(
                f"{POLICY_PATH.as_posix()}: effectiveFromContractCreatedAt must be a timestamp"
            )
        effective_from = _parse_timestamp(
            raw_effective_from,
            f"{POLICY_PATH.as_posix()}: effectiveFromContractCreatedAt",
        )
    else:
        raise ProjectionPolicyError(f"{POLICY_PATH.as_posix()}: unsupported schemaVersion")
    if set(value) != expected_fields:
        raise ProjectionPolicyError(
            f"{POLICY_PATH.as_posix()}: fields do not match schemaVersion {schema_version}"
        )
    if value.get("repositoryId") != repository_id:
        raise ProjectionPolicyError(f"{POLICY_PATH.as_posix()}: schema or repository identity mismatch")
    default_projection = value.get("defaultProjection")
    if default_projection not in {"derived", "required"}:
        raise ProjectionPolicyError(f"{POLICY_PATH.as_posix()}: invalid defaultProjection")

    def names(field: str) -> tuple[str, ...]:
        raw = value.get(field)
        if not isinstance(raw, list) or any(not isinstance(item, str) or not item for item in raw):
            raise ProjectionPolicyError(f"{POLICY_PATH.as_posix()}: {field} must be a string array")
        if len(raw) != len(set(raw)):
            raise ProjectionPolicyError(f"{POLICY_PATH.as_posix()}: {field} contains duplicates")
        return tuple(raw)

    required_modes = names("requiredModes")
    required_operations = names("requiredOperations")
    preserve = value.get("preserveExistingRegistrations")
    if not isinstance(preserve, bool):
        raise ProjectionPolicyError(
            f"{POLICY_PATH.as_posix()}: preserveExistingRegistrations must be boolean"
        )

    capabilities = _read_object(repository / CAPABILITIES_PATH)
    if capabilities.get("repositoryId") != repository_id:
        raise ProjectionPolicyError(f"{CAPABILITIES_PATH.as_posix()}: repository identity mismatch")
    mappings = capabilities.get("operationMappings")
    if not isinstance(mappings, dict):
        raise ProjectionPolicyError(f"{CAPABILITIES_PATH.as_posix()}: operationMappings is missing")
    for operation in required_operations:
        mapping = mappings.get(operation)
        if not isinstance(mapping, list) or not mapping or any(
            not isinstance(item, str) or not item for item in mapping
        ):
            raise ProjectionPolicyError(
                f"{CAPABILITIES_PATH.as_posix()}: required operation {operation!r} is unmapped"
            )

    return ProjectionPolicy(
        repository_id=repository_id,
        default_projection=default_projection,
        required_modes=required_modes,
        required_operations=required_operations,
        preserve_existing_registrations=preserve,
        effective_from_contract_created_at=effective_from,
    )


def contract_predates_projection_policy(
    contract: dict[str, Any], work_item_id: str, policy: ProjectionPolicy
) -> bool:
    """Keep a newly introduced repository policy from rewriting closed history."""
    effective_from = policy.effective_from_contract_created_at
    if effective_from is None:
        return False
    created_at = contract.get("createdAt")
    if not isinstance(created_at, str) or not created_at:
        raise ProjectionPolicyError(
            f"{work_item_id}: Contract createdAt is required by the effective projection policy"
        )
    created = _parse_timestamp(created_at, f"{work_item_id}: Contract createdAt")
    return created < effective_from


def requires_documentation_projection(
    contract: dict[str, Any] | None,
    work_item_id: str,
    policy: ProjectionPolicy,
    *,
    has_existing_registration: bool,
    additional_paths: tuple[str, ...] = (),
    acceptance_criteria: tuple[str, ...] = (),
) -> bool:
    """Select a projection from declared work, never from Work Item numbering."""
    if contract is None:
        if policy.effective_from_contract_created_at is not None:
            raise ProjectionPolicyError(
                f"{work_item_id}: Contract is required by the effective projection policy"
            )
        return policy.default_projection == "required" or has_existing_registration
    if contract_predates_projection_policy(contract, work_item_id, policy):
        return False
    if policy.default_projection == "required":
        return True

    mode = contract.get("mode")
    if mode is not None and not isinstance(mode, str):
        raise ProjectionPolicyError(f"{work_item_id}: Contract mode must be a string")
    if isinstance(mode, str) and mode in policy.required_modes:
        return True

    operation = contract.get("operation")
    requested_operation = contract.get("requestedOperation")
    for label, value in (("operation", operation), ("requestedOperation", requested_operation)):
        if value is not None and not isinstance(value, str):
            raise ProjectionPolicyError(f"{work_item_id}: Contract {label} must be a string")
    if operation and requested_operation and operation != requested_operation:
        raise ProjectionPolicyError(f"{work_item_id}: Contract operation fields conflict")
    effective_operation = operation or requested_operation
    if effective_operation in policy.required_operations:
        return True

    scope = contract.get("scope", [])
    if scope is not None and (
        not isinstance(scope, list) or any(not isinstance(item, str) or not item for item in scope)
    ):
        raise ProjectionPolicyError(f"{work_item_id}: Contract scope must be a string array")
    targets = tuple(path.format(work_item=work_item_id) for path in PROJECTION_PATHS)
    declared_paths = tuple(scope or ()) + additional_paths
    for item in declared_paths:
        normalized = item.replace("\\", "/").lstrip("./")
        if any(fnmatch.fnmatchcase(target, normalized) for target in targets):
            return True
    if any(
        any(marker in criterion.casefold() for marker in ("parity ledger", "parity registration"))
        for criterion in acceptance_criteria
    ):
        return True

    return policy.preserve_existing_registrations and has_existing_registration


def parity_registration_tokens(repository: Path) -> set[str]:
    """Read existing full or legacy-short registrations for promotion reuse."""
    tokens: set[str] = set()
    for suffix in ("", ".zh-CN", ".ja"):
        path = repository / f"docs/reference/reference-parity{suffix}.md"
        if path.is_symlink() or not path.is_file():
            continue
        for line in path.read_text(encoding="utf-8").splitlines():
            cells = line.split("|")
            if len(cells) < 3:
                continue
            token = cells[1].strip().split(maxsplit=1)[0] if cells[1].strip() else ""
            if WORK_ITEM_PATTERN.fullmatch(token):
                tokens.add(token)
    return tokens


def short_work_item_id(work_item_id: str) -> str:
    parts = work_item_id.split("-", 2)
    return "-".join(parts[:2]) if len(parts) >= 2 else work_item_id
