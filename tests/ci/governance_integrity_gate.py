#!/usr/bin/env python3
"""Fail-closed governance inventory and documentation-integrity gate."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


PARITY_DOCS = (
    ("docs/reference/reference-parity.md", "Implemented"),
    ("docs/reference/reference-parity.zh-CN.md", "已实现"),
    ("docs/reference/reference-parity.ja.md", "Implemented"),
)
PENDING_PARITY_REGISTRY = "docs/reference/pending-parity-registry.json"
RECORD_SUFFIXES = ("contract", "summary", "archive", "outcome")
WORK_ITEM_DOCUMENTS = (
    ("", "English"),
    (".ja", "Japanese"),
    (".zh-CN", "Simplified Chinese"),
)


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"{path}: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"{path}: expected a JSON object")
    return value


def current_release(repo: Path) -> str:
    fixture_metadata = repo / "cargo-metadata.json"
    if fixture_metadata.is_file():
        metadata = load_json(fixture_metadata)
    else:
        completed = subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
            cwd=repo,
            check=True,
            capture_output=True,
            text=True,
        )
        metadata = json.loads(completed.stdout)
    versions = {
        package["version"]
        for package in metadata.get("packages", [])
        if package.get("source") is None and isinstance(package.get("version"), str)
    }
    if len(versions) != 1:
        raise ValueError(f"workspace versions are not singular: {sorted(versions)}")
    return versions.pop()


def record_id(path: Path, suffix: str) -> str:
    ending = f".{suffix}.json"
    return path.name[: -len(ending)]


def short_id(work_item: str) -> str:
    match = re.match(r"^(WI-[0-9]+[A-Za-z]?)", work_item, re.IGNORECASE)
    return match.group(1).upper() if match else work_item


def created_at(path: Path) -> datetime | None:
    try:
        value = load_json(path).get("createdAt")
        if not isinstance(value, str):
            return None
        return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)
    except (ValueError, TypeError):
        return None


def archived_work_item_timestamp(repo: Path, work_item: str) -> datetime | None:
    for suffix in ("contract", "archive"):
        timestamp = created_at(
            repo / ".ai/work-items/archive" / f"{work_item}.{suffix}.json"
        )
        if timestamp is not None:
            return timestamp
    return None


def repository_default_branch(repo: Path, remote: str) -> str | None:
    event_path = os.environ.get("GITHUB_EVENT_PATH")
    if event_path:
        try:
            event = load_json(Path(event_path))
            repository = event.get("repository")
            if isinstance(repository, dict):
                value = repository.get("default_branch")
                if isinstance(value, str) and value:
                    return value
        except ValueError:
            pass
    remote_head = subprocess.run(
        [
            "git",
            "symbolic-ref",
            "--quiet",
            "--short",
            f"refs/remotes/{remote}/HEAD",
        ],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    prefix = f"{remote}/"
    if remote_head.startswith(prefix) and len(remote_head) > len(prefix):
        return remote_head[len(prefix) :]
    if os.environ.get("GITHUB_EVENT_NAME") == "pull_request":
        value = os.environ.get("GITHUB_BASE_REF")
        if value:
            return value
    return None


def repository_phase(repo: Path, base_branch: str) -> str:
    ref = os.environ.get("GITHUB_REF", "")
    if ref.startswith("refs/tags/"):
        return "release_tag"
    branch = subprocess.run(
        ["git", "symbolic-ref", "--quiet", "--short", "HEAD"],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    current_branch = branch.stdout.strip()
    if current_branch:
        return "default_branch" if current_branch == base_branch else "feature_branch"
    if os.environ.get("GITHUB_EVENT_NAME") == "pull_request":
        return "pull_request"
    if os.environ.get("GITHUB_REF") == f"refs/heads/{base_branch}":
        return "default_branch"
    return "unknown"


def release_tag_proves_merged_head(repo: Path, head_revision: str) -> bool:
    """Prove a pre-merge branch head is contained by the immutable release tag.

    A release-tag source checkout is detached, so the normal feature-branch
    phase cannot be used.  The tag itself is only an allowed transitional
    boundary when it resolves to the checked-out commit and the recorded PR
    head is an ancestor of that commit.  The later release policy gate still
    proves that the tagged commit is on the protected default branch.
    """
    ref = os.environ.get("GITHUB_REF", "")
    if not ref.startswith("refs/tags/"):
        return False
    current = os.environ.get("GITHUB_SHA")
    if not isinstance(current, str) or re.fullmatch(r"[0-9a-f]{40}", current) is None:
        current = subprocess.run(
            ["git", "rev-parse", "HEAD^{commit}"],
            cwd=repo,
            check=False,
            capture_output=True,
            text=True,
        ).stdout.strip()
    if re.fullmatch(r"[0-9a-f]{40}", current) is None:
        return False
    tag_name = ref.removeprefix("refs/tags/")
    tagged = subprocess.run(
        ["git", "rev-parse", f"refs/tags/{tag_name}^{{commit}}"],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if tagged != current:
        return False
    return (
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", head_revision, current],
            cwd=repo,
            check=False,
            capture_output=True,
        ).returncode
        == 0
    )


def checked_out_review_head(repo: Path, phase: str) -> str | None:
    """Resolve the provider-reviewed head represented by this checkout."""
    if phase == "pull_request":
        parents = subprocess.run(
            ["git", "rev-list", "--parents", "-n", "1", "HEAD"],
            cwd=repo,
            check=False,
            capture_output=True,
            text=True,
        ).stdout.split()
        spec = "HEAD^2" if len(parents) >= 3 else "HEAD"
    elif phase == "feature_branch":
        spec = "HEAD"
    else:
        return None
    resolved = subprocess.run(
        ["git", "rev-parse", "--verify", f"{spec}^{{commit}}"],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    value = resolved.stdout.strip()
    return value if re.fullmatch(r"[0-9a-f]{40}", value) else None


def finalization_head_matches_checkout(
    repo: Path,
    phase: str,
    work_item: str,
    recorded_head: str,
) -> bool:
    """Bind finalization to the reviewed head with bounded governance append drift."""
    reviewed_head = checked_out_review_head(repo, phase)
    resolved_recorded = resolve_commit_revision(repo, recorded_head)
    if reviewed_head is None or resolved_recorded is None:
        return False
    if resolved_recorded == reviewed_head:
        return True
    if subprocess.run(
        ["git", "merge-base", "--is-ancestor", resolved_recorded, reviewed_head],
        cwd=repo,
        check=False,
        capture_output=True,
    ).returncode != 0:
        return False
    changes = subprocess.run(
        ["git", "diff", "--name-status", resolved_recorded, reviewed_head, "--"],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    if changes.returncode != 0:
        return False
    canonical = f".ai/decisions/{work_item}.finalize.json"
    transition_prefix = f".ai/decisions/{work_item}.finalize."
    post_finalize_paths = {
        # Pending parity registration is a bounded governance transition. It
        # may be appended after the reviewed head so a merged/closed Work Item
        # can remain visible as awaiting parity completion; it must not be
        # accompanied by implementation or arbitrary documentation drift.
        PENDING_PARITY_REGISTRY,
        f".ai/evidence/{work_item}/quality-route-post-finalize.json",
        f".ai/evidence/{work_item}/repository-gates-post-finalize.json",
        f".ai/decisions/{work_item}.close.json",
    }
    for line in changes.stdout.splitlines():
        status, _, relative = line.partition("\t")
        if relative == PENDING_PARITY_REGISTRY:
            if status not in {"A", "M"}:
                return False
            continue
        transition = (
            relative.startswith(transition_prefix)
            and relative.endswith(".json")
            and len(relative.removeprefix(transition_prefix)[:-5]) == 64
            and all(
                character in "0123456789abcdef"
                for character in relative.removeprefix(transition_prefix)[:-5]
            )
        )
        if status != "A" or (
            relative != canonical
            and not transition
            and relative not in post_finalize_paths
        ):
            return False
    return True


def default_branch_contains_reviewed_head(repo: Path, head_revision: str) -> bool:
    """Return true only when the exact reviewed PR head is in the checkout.

    This is deliberately narrower than a provider merge lookup: a merge (or
    an equivalent reviewed integration) is proven only when the immutable PR
    head is an ancestor of the synchronized default-branch checkout. If a
    provider used squash/rebase and the exact head cannot be proven, the gate
    stays unknown and the normal explicit finalization/close path remains
    required.
    """
    resolved_head = resolve_commit_revision(repo, head_revision)
    if resolved_head is None:
        return False
    current = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    if current.returncode != 0:
        return False
    return (
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", resolved_head, current.stdout.strip()],
            cwd=repo,
            check=False,
            capture_output=True,
        ).returncode
        == 0
    )


def resolve_commit_revision(repo: Path, revision: str) -> str | None:
    """Resolve a provider receipt revision without accepting ambiguous text.

    Provider APIs and local Git commands occasionally emit an abbreviated
    commit.  The receipt still has to bind one exact object: Git's
    ``^{commit}`` resolution rejects ambiguous or non-commit names, and the
    returned object name is always the canonical forty-character SHA.
    """
    if not isinstance(revision, str) or re.fullmatch(r"[0-9a-f]{7,40}", revision) is None:
        return None
    if len(revision) == 40:
        # Existing fixture and provider records already carry a canonical
        # object name.  Keep this path compatible with detached synthetic
        # repositories used by the regression corpus.
        return revision
    resolved = subprocess.run(
        ["git", "rev-parse", "--verify", f"{revision}^{{commit}}"],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    value = resolved.stdout.strip()
    if resolved.returncode != 0 or re.fullmatch(r"[0-9a-f]{40}", value) is None:
        return None
    return value


def stale_awaiting_merge_close(repo: Path, value: dict[str, Any], phase: str) -> bool:
    """Detect a merged exact PR head whose pre-merge receipt was never closed."""
    if phase != "default_branch":
        return False
    after = value.get("after")
    pull_request = value.get("pullRequest")
    result = value.get("result")
    if not all(isinstance(item, dict) for item in (after, pull_request, result)):
        return False
    return (
        after.get("pullRequest") == "unmerged"
        and pull_request.get("mergeCommit") is None
        and result.get("disposition") == "blocked"
        and result.get("failureCodes") == ["unmerged_pull_request"]
        and default_branch_contains_reviewed_head(repo, pull_request.get("headRevision", ""))
    )


def premerge_finalize_state(
    repo: Path, work_item: str, value: dict[str, Any]
) -> tuple[bool, str]:
    after = value.get("after")
    branch_identity = value.get("branch")
    pull_request = value.get("pullRequest")
    resource_context = value.get("resourceContext")
    result = value.get("result")
    worktree = value.get("worktree")
    if not all(
        isinstance(item, dict)
        for item in (
            after,
            branch_identity,
            pull_request,
            resource_context,
            result,
            worktree,
        )
    ):
        return False, "unknown"
    base_branch = pull_request.get("baseBranch")
    base_remote = pull_request.get("baseRemote")
    if (
        not isinstance(base_branch, str)
        or not base_branch
        or not isinstance(base_remote, str)
        or not base_remote
    ):
        return False, "unknown"
    reason = value.get("reason")
    reason_valid = isinstance(reason, str) and bool(reason.strip())
    phase = "unknown"
    contract_path = (
        repo / ".ai/work-items/archive" / f"{work_item}.contract.json"
    )
    evidence_path = repo / ".ai/evidence" / f"{work_item}.verification.json"
    try:
        project = load_json(repo / ".ai/project.json")
        contract = load_json(contract_path)
        evidence = load_json(evidence_path)
        contract_digest = "sha256:" + hashlib.sha256(contract_path.read_bytes()).hexdigest()
    except (ValueError, OSError):
        return False, phase
    repository_id = project.get("repositoryId")
    contract_context = contract.get("resourceContext")
    default_branch = repository_default_branch(repo, base_remote)
    declared_base_branch = (
        contract_context.get("baseBranch")
        if isinstance(contract_context, dict)
        else None
    )
    # A pull-request merge checkout may be detached and may not contain
    # origin/HEAD or the event payload/base-ref environment variables.  The
    # Contract's immutable resource context is the only safe fallback: it is
    # compared byte-for-byte below and cannot silently broaden the PR identity.
    effective_default_branch = default_branch or (
        declared_base_branch if isinstance(declared_base_branch, str) and declared_base_branch else None
    )
    phase = (
        repository_phase(repo, effective_default_branch)
        if effective_default_branch
        else "unknown"
    )
    runtime_digest = value.get("runtimeDigest")
    runtime_version = value.get("runtimeVersion")
    base_revision = pull_request.get("baseRevision")
    head_revision = pull_request.get("headRevision")
    resolved_head = resolve_commit_revision(repo, head_revision)
    resolved_branch_head = resolve_commit_revision(repo, branch_identity.get("headRevision"))
    resolved_worktree_head = resolve_commit_revision(repo, worktree.get("headRevision"))
    pull_request_url = pull_request.get("url")
    valid = (
        value.get("workItemId") == work_item
        and isinstance(repository_id, str)
        and value.get("repositoryId") == repository_id
        and evidence.get("repositoryId") == repository_id
        and isinstance(runtime_digest, str)
        and re.fullmatch(r"sha256:[0-9a-f]{64}", runtime_digest) is not None
        and runtime_digest == evidence.get("runtimeDigest")
        and isinstance(runtime_version, str)
        and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", runtime_version) is not None
        and runtime_version == evidence.get("runtimeVersion")
        and value.get("contractDigest") == contract_digest
        and contract.get("workItemId") == work_item
        and isinstance(contract_context, dict)
        and resource_context == contract_context
        and effective_default_branch is not None
        and base_branch == effective_default_branch
        and (default_branch is not None or base_branch == declared_base_branch)
        and resource_context.get("baseBranch") == base_branch
        and resource_context.get("baseRemote") == base_remote
        and resource_context.get("provider") == value.get("provider")
        and resource_context.get("pullRequest") == pull_request_url
        and isinstance(pull_request_url, str)
        and bool(pull_request_url)
        and branch_identity.get("name") == resource_context.get("branch")
        and branch_identity.get("remote") == base_remote
        and worktree.get("branch") == branch_identity.get("name")
        and worktree.get("path") == resource_context.get("worktree")
        and worktree.get("worktreeId") == work_item
        and resolved_head is not None
        and resolved_branch_head == resolved_head
        and resolved_worktree_head == resolved_head
        and isinstance(base_revision, str)
        and re.fullmatch(r"[0-9a-f]{40}", base_revision) is not None
        and base_revision == contract.get("baseRevision")
        and after.get("branch") == "present"
        and after.get("worktree") == "clean"
        and after.get("pullRequest") == "unmerged"
        and pull_request.get("mergeCommit") is None
        and result.get("disposition") == "blocked"
        and result.get("failureCodes") == ["unmerged_pull_request"]
        and result.get("unknownCodes") == []
        and reason_valid
        and (
            (
                phase in {"feature_branch", "pull_request"}
                and finalization_head_matches_checkout(
                    repo, phase, work_item, head_revision
                )
            )
            or (
                phase == "release_tag"
                and release_tag_proves_merged_head(repo, head_revision)
            )
        )
    )
    return valid, phase


def parity_rows(repo: Path) -> tuple[dict[str, dict[str, str]], list[dict[str, str]]]:
    rows: dict[str, dict[str, str]] = {}
    findings: list[dict[str, str]] = []
    for relative, implemented in PARITY_DOCS:
        path = repo / relative
        if not path.is_file():
            findings.append(
                {
                    "workItemId": "repository",
                    "code": "missing_parity_document",
                    "path": relative,
                    "severity": "error",
                }
            )
            continue
        for line in path.read_text(encoding="utf-8").splitlines():
            match = re.match(
                r"^\|\s*(WI-[0-9]+[A-Za-z]?)(?=\s|—|\|)", line, re.IGNORECASE
            )
            if match:
                rows.setdefault(match.group(1).upper(), {})[relative] = line
    return rows, findings


def _reject_duplicate_json_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


def load_pending_parity_registry(repo: Path) -> list[dict[str, Any]]:
    path = repo / PENDING_PARITY_REGISTRY
    if path.is_symlink():
        raise ValueError("pending parity registry must be a regular file")
    if not path.exists():
        return []
    if not path.is_file():
        raise ValueError("pending parity registry must be a regular file")
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=_reject_duplicate_json_keys,
        )
    except (OSError, json.JSONDecodeError, ValueError) as error:
        raise ValueError(f"invalid pending parity registry JSON: {error}") from error
    if not isinstance(value, dict) or set(value) != {"entries", "schemaVersion"}:
        raise ValueError("pending parity registry fields are invalid")
    if value["schemaVersion"] != 1 or not isinstance(value["entries"], list):
        raise ValueError("pending parity registry schema is invalid")
    entries: list[dict[str, Any]] = []
    work_items: set[str] = set()
    pull_requests: set[tuple[str, int, str]] = set()
    expected_entry_fields = {
        "baseRevision",
        "createdAt",
        "expectedRecords",
        "headRevision",
        "parityRows",
        "provider",
        "pullRequest",
        "registryBaseRevision",
        "repositoryId",
        "state",
        "workItemId",
    }
    for entry in value["entries"]:
        if not isinstance(entry, dict) or set(entry) != expected_entry_fields:
            raise ValueError("pending parity entry fields are invalid")
        work_item = entry["workItemId"]
        provider = entry["provider"]
        pull_request = entry["pullRequest"]
        if (
            not isinstance(work_item, str)
            or re.fullmatch(r"WI-[0-9]+[A-Za-z]?-[A-Za-z0-9][A-Za-z0-9-]*", work_item)
            is None
            or provider != "github"
            or not isinstance(pull_request, dict)
            or set(pull_request) != {"number", "url"}
            or not isinstance(pull_request["number"], int)
            or isinstance(pull_request["number"], bool)
            or pull_request["number"] <= 0
            or not isinstance(pull_request["url"], str)
            or re.fullmatch(
                rf"https://github\.com/[^/]+/[^/]+/pull/{pull_request['number']}",
                pull_request["url"],
            )
            is None
        ):
            raise ValueError("pending parity entry identity is invalid")
        pull_request_identity = (
            provider,
            pull_request["number"],
            pull_request["url"],
        )
        if work_item in work_items or pull_request_identity in pull_requests:
            raise ValueError("pending parity entry identity is duplicated")
        work_items.add(work_item)
        pull_requests.add(pull_request_identity)
        entries.append(entry)
    return entries


def _regular_repository_file(repo: Path, relative: str) -> Path | None:
    candidate = Path(relative)
    if (
        not relative
        or candidate.is_absolute()
        or ".." in candidate.parts
        or candidate.as_posix() != relative
    ):
        return None
    path = repo / candidate
    if path.is_symlink() or not path.is_file():
        return None
    try:
        path.resolve().relative_to(repo)
    except ValueError:
        return None
    return path


def _parity_row_precedes_record(
    repo: Path,
    parity_relative: str,
    row: str,
    record_relative: str,
) -> bool:
    """Prove the lifecycle row was registered before evidence appeared.

    The parity ledger is a projection, so a later commit may legitimately add
    evidence/decision links to a row that was already registered before the
    Work Item was archived. Blaming only the current, enriched line would
    incorrectly classify that append as stale. We require the current row to
    be unique and complete, then inspect its line history for the first commit
    carrying the same Work Item/status registration key. A row introduced only
    after the evidence path remains fail-closed.
    """

    parity_path = repo / parity_relative
    matching_lines = [
        index
        for index, value in enumerate(
            parity_path.read_text(encoding="utf-8").splitlines(), start=1
        )
        if value == row
    ]
    if len(matching_lines) != 1:
        return False
    record = subprocess.run(
        [
            "git",
            "log",
            "-1",
            "--diff-filter=A",
            "--format=%H",
            "HEAD",
            "--",
            record_relative,
        ],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    record_revision = record.stdout.strip()
    if re.fullmatch(r"[0-9a-f]{40}", record_revision) is None:
        return False
    cells = [cell.strip() for cell in row.strip().strip("|").split("|")]
    if len(cells) < 3 or not cells[0] or not cells[1]:
        return False
    registration_pattern = (
        rf"^\|[[:space:]]*{re.escape(cells[0])}[[:space:]]*\|[[:space:]]*"
        rf"{re.escape(cells[1])}[[:space:]]*\|"
    )
    history = subprocess.run(
        [
            "git",
            "log",
            "--format=%H",
            "--reverse",
            "--follow",
            f"-G{registration_pattern}",
            "--",
            parity_relative,
        ],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    if history.returncode != 0:
        return False
    for candidate in history.stdout.splitlines():
        candidate = candidate.strip()
        if (
            re.fullmatch(r"[0-9a-f]{40}", candidate) is None
            or candidate == record_revision
        ):
            continue
        if (
            subprocess.run(
                ["git", "merge-base", "--is-ancestor", candidate, record_revision],
                cwd=repo,
                check=False,
                capture_output=True,
            ).returncode
            == 0
        ):
            return True
    return False


def _active_parity_projection_declared(
    repo: Path,
    work_item: str,
) -> bool:
    """Select the active parity control from declared scope or observed paths."""

    try:
        contract = load_json(
            repo / ".ai/work-items/active" / f"{work_item}.contract.json"
        )
        summary = load_json(
            repo / ".ai/work-items/active" / f"{work_item}.summary.json"
        )
    except ValueError:
        # Malformed active governance records must not disable the control.
        return True
    declared_paths: list[Any] = []
    for value in (contract.get("scope"), summary.get("changedPaths")):
        if isinstance(value, list):
            declared_paths.extend(value)
    path_declared = any(
        isinstance(relative, str)
        and relative.startswith("docs/reference/reference-parity")
        for relative in declared_paths
    )
    acceptance = contract.get("acceptanceCriteria")
    acceptance_declared = isinstance(acceptance, list) and any(
        isinstance(criterion, str)
        and (
            "parity ledger" in criterion.casefold()
            or "parity registration" in criterion.casefold()
        )
        for criterion in acceptance
    )
    return path_declared or acceptance_declared


def _work_item_document_issue(
    repo: Path,
    work_item: str,
    suffix: str,
) -> tuple[str, str] | None:
    """Return a stable finding code/path when a projection is not trustworthy."""

    relative = f"docs/work-items/{work_item}{suffix}.md"
    path = repo / relative
    if path.is_symlink() or not path.exists():
        return "missing_work_item_document", relative
    if not path.is_file():
        return "invalid_work_item_document", relative
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError):
        return "invalid_work_item_document", relative
    if not text.startswith("---\n") or "\n---\n" not in text:
        return "invalid_work_item_document", relative
    if not re.search(
        rf"(?m)^workItemId:\s*{re.escape(work_item)}\s*$",
        text,
    ):
        return "invalid_work_item_document", relative
    return None


def _pending_append_is_bounded(
    repo: Path,
    head_revision: str,
    registry_base_revision: str,
    base_branch: str,
) -> bool:
    if (
        re.fullmatch(r"[0-9a-f]{40}", head_revision) is None
        or re.fullmatch(r"[0-9a-f]{40}", registry_base_revision) is None
    ):
        return False
    ancestor = subprocess.run(
        ["git", "merge-base", "--is-ancestor", head_revision, registry_base_revision],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    if ancestor.returncode != 0:
        return False
    phase = repository_phase(repo, base_branch)
    reviewed_revision = "HEAD^2" if phase == "pull_request" else "HEAD"
    if phase not in {"feature_branch", "pull_request"}:
        return False
    reviewed = subprocess.run(
        ["git", "rev-parse", "--verify", f"{reviewed_revision}^{{commit}}"],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    parents = subprocess.run(
        ["git", "rev-list", "--parents", "-n", "1", reviewed],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.split()
    if len(parents) != 2 or parents[1] != registry_base_revision:
        return False
    changed = subprocess.run(
        [
            "git",
            "diff",
            "--name-status",
            f"{registry_base_revision}..{reviewed}",
            "--",
        ],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    if changed.returncode != 0:
        return False
    return changed.stdout.splitlines() in (
        [f"A\t{PENDING_PARITY_REGISTRY}"],
        [f"M\t{PENDING_PARITY_REGISTRY}"],
    )


def validate_pending_parity_entry(
    repo: Path,
    entry: dict[str, Any],
    work_item: str,
    rows: dict[str, dict[str, str]],
    lifecycle_state: str,
) -> bool:
    expected_records = entry["expectedRecords"]
    expected_paths = {
        "contract": f".ai/work-items/archive/{work_item}.contract.json",
        "finalize": f".ai/decisions/{work_item}.finalize.json",
        "verification": f".ai/evidence/{work_item}.verification.json",
    }
    if not isinstance(expected_records, dict) or expected_records != expected_paths:
        return False
    record_paths = {
        key: _regular_repository_file(repo, relative)
        for key, relative in expected_paths.items()
    }
    if any(path is None for path in record_paths.values()):
        return False
    try:
        project = load_json(repo / ".ai/project.json")
        contract = load_json(record_paths["contract"])
        evidence = load_json(record_paths["verification"])
        finalize = load_json(record_paths["finalize"])
    except (ValueError, TypeError):
        return False
    pull_request = entry["pullRequest"]
    finalize_pull_request = finalize.get("pullRequest")
    finalize_context = finalize.get("resourceContext")
    if not isinstance(finalize_pull_request, dict) or not isinstance(
        finalize_context, dict
    ):
        return False
    base_revision = entry["baseRevision"]
    head_revision = entry["headRevision"]
    registry_base_revision = entry["registryBaseRevision"]
    created_at_value = entry["createdAt"]
    try:
        created_at_time = datetime.fromisoformat(
            created_at_value.removesuffix("Z") + "+00:00"
        )
    except (AttributeError, TypeError, ValueError):
        return False
    if created_at_time.tzinfo is None or not created_at_value.endswith("Z"):
        return False
    if (
        entry["repositoryId"] != project.get("repositoryId")
        or entry["workItemId"] != work_item
        or entry["state"] != "in_progress"
        or lifecycle_state != "awaiting_merge_close"
        or re.fullmatch(r"[0-9a-f]{40}", base_revision) is None
        or contract.get("repositoryId") != entry["repositoryId"]
        or base_revision != contract.get("baseRevision")
        or base_revision != finalize_pull_request.get("baseRevision")
        or pull_request["number"] != finalize_pull_request.get("number")
        or pull_request["url"] != finalize_pull_request.get("url")
        or entry["provider"] != finalize.get("provider")
        or entry["provider"] != finalize_context.get("provider")
        or pull_request["url"] != finalize_context.get("pullRequest")
        or contract.get("workItemId") != work_item
        or contract.get("resourceContext") != finalize_context
        or evidence.get("workItemId") != work_item
        or evidence.get("repositoryId") != entry["repositoryId"]
        or evidence.get("passed") is not True
        or finalize.get("workItemId") != work_item
        or finalize.get("repositoryId") != entry["repositoryId"]
        or finalize.get("runtimeVersion") != evidence.get("runtimeVersion")
        or finalize.get("runtimeDigest") != evidence.get("runtimeDigest")
        or finalize.get("contractDigest")
        != "sha256:" + hashlib.sha256(record_paths["contract"].read_bytes()).hexdigest()
        or finalize_pull_request.get("headRevision") != head_revision
        or not isinstance(finalize.get("branch"), dict)
        or finalize["branch"].get("headRevision") != head_revision
        or not isinstance(finalize.get("worktree"), dict)
        or finalize["worktree"].get("headRevision") != head_revision
        or not _pending_append_is_bounded(
            repo,
            head_revision,
            registry_base_revision,
            str(finalize_pull_request.get("baseBranch", "")),
        )
    ):
        return False
    parity_rows_value = entry["parityRows"]
    expected_parity_paths = [relative for relative, _ in PARITY_DOCS]
    if not isinstance(parity_rows_value, list) or len(parity_rows_value) != 3:
        return False
    if [item.get("path") for item in parity_rows_value if isinstance(item, dict)] != expected_parity_paths:
        return False
    for item, (relative, _) in zip(parity_rows_value, PARITY_DOCS, strict=True):
        if not isinstance(item, dict) or set(item) != {"path", "row"}:
            return False
        row = item["row"]
        pending_status = "进行中" if relative.endswith(".zh-CN.md") else "In progress"
        if (
            not isinstance(row, str)
            or not row.startswith(f"| {short_id(work_item)} ")
            or f"| {pending_status} |" not in row
            or expected_paths["verification"] not in row
            or expected_paths["finalize"] not in row
        ):
            return False
    return not rows.get(short_id(work_item))


def finding(
    work_item: str, code: str, path: str, severity: str = "error"
) -> dict[str, str]:
    return {"workItemId": work_item, "code": code, "path": path, "severity": severity}


def valid_recovery_decision(
    repo: Path, work_item: str, value: dict[str, Any]
) -> bool:
    """Validate the minimum identity of a predecessor recovery receipt.

    Recovery is a terminal inventory state for the predecessor, not a green
    completion claim.  Keep this check intentionally small and explicit: the
    Runtime owns the detailed receipt schema, while this static gate must only
    accept a repository-local, predecessor-bound successor link.
    """
    try:
        project = load_json(repo / ".ai/project.json")
    except ValueError:
        return False
    repository_id = project.get("repositoryId")
    evidence_refs = value.get("evidenceRefs")
    decision = value.get("decision")
    successor = value.get("successorWorkItemId")
    successor_shape_valid = (
        (decision == "retry" and (successor is None or (isinstance(successor, str) and bool(successor))))
        or (decision in {"supersede", "successor"} and isinstance(successor, str) and bool(successor))
    )
    return (
        value.get("schemaVersion") == 1
        and value.get("workItemId") == work_item
        and value.get("predecessorWorkItemId") == work_item
        and successor_shape_valid
        and isinstance(repository_id, str)
        and value.get("repositoryId") == repository_id
        and isinstance(evidence_refs, list)
        and bool(evidence_refs)
        and all(isinstance(item, str) and item for item in evidence_refs)
        and isinstance(value.get("reason"), str)
        and bool(value["reason"].strip())
    )


def recovery_decision_candidate(
    repo: Path, work_item: str
) -> tuple[Path, dict[str, Any]] | None:
    """Resolve the valid head of an append-only recovery decision chain.

    Runtime recovery receipts are immutable: a retry may remain at the
    canonical ``.recovery.json`` path while a later successor/supersession
    decision is written under a digest-suffixed path. The gate must inspect
    that chain rather than treating the earlier retry as the terminal record.
    Invalid candidates are never promoted; a deterministic preference for a
    supersession, then successor, then the latest decision timestamp keeps
    the projection stable when a predecessor has more than one valid receipt.
    """
    decisions = repo / ".ai/decisions"
    canonical = decisions / f"{work_item}.recovery.json"
    paths = [canonical]
    paths.extend(sorted(decisions.glob(f"{work_item}.recovery.*.json")))
    candidates: list[tuple[Path, dict[str, Any]]] = []
    for path in paths:
        if not path.is_file() or path.is_symlink():
            continue
        try:
            value = load_json(path)
        except ValueError:
            continue
        if valid_recovery_decision(repo, work_item, value):
            candidates.append((path, value))
    if not candidates:
        return None
    return max(
        candidates,
        key=lambda item: (
            item[1].get("decision") == "supersede",
            item[1].get("decision") == "successor",
            item[1].get("decidedAt", ""),
            str(item[0]),
        ),
    )


def valid_close_decision(repo: Path, work_item: str, value: dict[str, Any]) -> bool:
    try:
        project = load_json(repo / ".ai/project.json")
    except ValueError:
        return False
    structured = value.get("structuredDecision")
    if not isinstance(structured, dict):
        return False
    nonempty = lambda key: isinstance(structured.get(key), str) and bool(
        structured[key].strip()
    )
    return (
        value.get("workItemId") == work_item
        and value.get("state") == "closed"
        and value.get("decisionState") == "confirmed"
        and value.get("humanDecision") in {"approved", "superseded"}
        and structured.get("decision") in {"approved", "superseded"}
        and all(nonempty(key) for key in ("actor", "authoritySource", "reason", "decidedAt"))
        and isinstance(structured.get("evidenceRefs"), list)
        and bool(structured["evidenceRefs"])
        and isinstance(structured.get("policyRefs"), list)
        and (structured.get("resumeCondition") is None or nonempty("resumeCondition"))
        and isinstance(value.get("repositoryId"), str)
        and value.get("repositoryId") == project.get("repositoryId")
    )


def archive_digests_valid(repo: Path, work_item: str) -> bool:
    manifest_path = repo / ".ai/work-items/archive" / f"{work_item}.archive.json"
    try:
        manifest = load_json(manifest_path)
    except ValueError:
        return False
    files = manifest.get("files")
    if not isinstance(files, dict):
        return True
    for name in ("contract", "summary", "outcome"):
        expected = files.get(f"{name}Digest")
        if not isinstance(expected, str):
            return False
        path = repo / ".ai/work-items/archive" / f"{work_item}.{name}.json"
        if not path.is_file() or path.is_symlink():
            return False
        if "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--report", required=True)
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    report_path = Path(args.report)
    findings: list[dict[str, str]] = []
    inventory: list[dict[str, Any]] = []

    try:
        release = current_release(repo)
    except (ValueError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        release = "unknown"
        findings.append(finding("repository", "unknown_problem", "cargo-metadata"))
        metadata_error = str(error)
    else:
        metadata_error = None

    rows, parity_findings = parity_rows(repo)
    findings.extend(parity_findings)
    try:
        pending_entries = load_pending_parity_registry(repo)
    except ValueError:
        pending_entries = []
        findings.append(
            finding(
                "repository",
                "invalid_pending_parity_registry",
                PENDING_PARITY_REGISTRY,
            )
        )
    pending_by_work_item = {
        entry["workItemId"]: entry for entry in pending_entries
    }
    locations: dict[str, set[str]] = {}
    for location in ("active", "archive"):
        directory = repo / ".ai/work-items" / location
        for path in sorted(directory.glob("*.contract.json")):
            locations.setdefault(record_id(path, "contract"), set()).add(location)

    release_tokens = {f"v{release}", f"v{release.replace('.', '-')}"}
    release_contract_dates = []
    for work_item, work_item_locations in locations.items():
        if "archive" not in work_item_locations:
            continue
        if any(token.lower() in work_item.lower() for token in release_tokens):
            timestamp = archived_work_item_timestamp(repo, work_item)
            if timestamp is not None:
                release_contract_dates.append(timestamp)
    cycle_start = min(release_contract_dates) if release_contract_dates else None
    classifications: dict[str, str] = {}
    for work_item, work_item_locations in locations.items():
        location = "active" if "active" in work_item_locations else "archive"
        contract_created_at = (
            None
            if location == "active"
            else archived_work_item_timestamp(repo, work_item)
        )
        if location == "active":
            classifications[work_item] = "current_active"
        elif any(token.lower() in work_item.lower() for token in release_tokens):
            classifications[work_item] = "current_release_cycle"
        elif cycle_start is not None and contract_created_at is not None and contract_created_at >= cycle_start:
            classifications[work_item] = "current_release_cycle"
        elif contract_created_at is None:
            classifications[work_item] = "legacy"
        else:
            classifications[work_item] = "historical"

    full_ids_by_short: dict[str, list[str]] = {}
    for work_item in locations:
        full_ids_by_short.setdefault(short_id(work_item), []).append(work_item)
    for registered_id, full_ids in sorted(full_ids_by_short.items()):
        if len(full_ids) > 1:
            severity = (
                "error"
                if any(classifications[item].startswith("current") for item in full_ids)
                else "historical"
            )
            for work_item in sorted(full_ids):
                findings.append(
                    finding(work_item, "ambiguous_short_id", "docs/reference", severity)
                )
    known_short_ids = set(full_ids_by_short)
    for registered_id, translations in rows.items():
        if registered_id not in known_short_ids:
            path = sorted(translations)[0] if translations else "docs/reference"
            findings.append(finding(registered_id, "missing_work_item", path))
    for pending_work_item in sorted(pending_by_work_item):
        pending_locations = locations.get(pending_work_item)
        if pending_locations is None:
            findings.append(
                finding(
                    pending_work_item,
                    "pending_parity_work_item_missing",
                    PENDING_PARITY_REGISTRY,
                )
            )
        elif pending_locations != {"archive"} or classifications[
            pending_work_item
        ] in {"historical", "legacy"}:
            findings.append(
                finding(
                    pending_work_item,
                    "invalid_pending_parity_registration",
                    PENDING_PARITY_REGISTRY,
                )
            )

    for work_item in sorted(locations):
        work_item_locations = locations[work_item]
        if work_item_locations == {"active", "archive"}:
            findings.append(finding(work_item, "duplicate_work_item", ".ai/work-items"))
        location = "active" if "active" in work_item_locations else "archive"
        classification = classifications[work_item]

        record: dict[str, Any] = {
            "classification": classification,
            "location": location,
            "workItemId": work_item,
        }
        if classification in {"historical", "legacy"}:
            record["exemption"] = (
                "missing_created_at" if classification == "legacy" else "before_current_release_cycle"
            )
            inventory.append(record)
            continue

        base = repo / ".ai/work-items" / location
        required_suffixes = ("contract", "summary") if location == "active" else RECORD_SUFFIXES
        for suffix in required_suffixes:
            relative = f".ai/work-items/{location}/{work_item}.{suffix}.json"
            if not (base / f"{work_item}.{suffix}.json").is_file():
                findings.append(finding(work_item, f"missing_{suffix}", relative))

        parity_projection = (
            _active_parity_projection_declared(repo, work_item)
            if location == "active"
            else short_id(work_item) in rows
        )
        if parity_projection:
            for document_suffix, _language in WORK_ITEM_DOCUMENTS:
                issue = _work_item_document_issue(repo, work_item, document_suffix)
                if issue is not None:
                    findings.append(finding(work_item, issue[0], issue[1]))

        if location == "active" and not parity_projection:
            record["lifecycleState"] = "active_non_parity"
        elif location == "active":
            work_item_rows = rows.get(short_id(work_item), {})
            expected_records = (
                f".ai/work-items/archive/{work_item}.contract.json",
                f".ai/evidence/{work_item}.verification.json",
                f".ai/decisions/{work_item}.finalize.json",
                f".ai/decisions/{work_item}.close.json",
            )
            prearchive_valid = True
            for parity_doc, _ in PARITY_DOCS:
                line = work_item_rows.get(parity_doc)
                if line is None:
                    prearchive_valid = False
                    findings.append(
                        finding(
                            work_item,
                            "missing_prearchive_parity_entry",
                            parity_doc,
                        )
                    )
                    continue
                expected_status = (
                    "进行中 → 验证关闭后已实现"
                    if parity_doc.endswith(".zh-CN.md")
                    else (
                        "In progress → verified close 後 Implemented"
                        if parity_doc.endswith(".ja.md")
                        else "In progress → Implemented after verified close"
                    )
                )
                if f"| {expected_status} |" not in line or any(
                    f"`{relative}`" not in line for relative in expected_records
                ):
                    prearchive_valid = False
                    findings.append(
                        finding(
                            work_item,
                            "invalid_prearchive_parity_registration",
                            parity_doc,
                        )
                    )
            record["lifecycleState"] = (
                "prearchive_parity_registered"
                if prearchive_valid
                else "prearchive_parity_incomplete"
            )

        if location == "archive":
            if not archive_digests_valid(repo, work_item):
                findings.append(
                    finding(
                        work_item,
                        "archive_digest_mismatch",
                        f".ai/work-items/archive/{work_item}.archive.json",
                    )
                )
            recovery_candidate = recovery_decision_candidate(repo, work_item)
            recovery_path = (
                recovery_candidate[0]
                if recovery_candidate is not None
                else repo / ".ai/decisions" / f"{work_item}.recovery.json"
            )
            recovery_value = (
                recovery_candidate[1] if recovery_candidate is not None else {}
            )
            recovery_receipt_valid = recovery_candidate is not None

            outcome_path = base / f"{work_item}.outcome.json"
            if outcome_path.is_file():
                try:
                    outcome = load_json(outcome_path)
                except ValueError:
                    outcome = {}
                outcome_identity_valid = outcome.get("workItemId") == work_item
                outcome_green = outcome.get("decisionState") == "green"
                # A valid recovery explicitly preserves a blocked/non-green
                # predecessor.  It is inventory-recovered, never promoted to
                # green.  Without that receipt, the historical Outcome must
                # remain green for a normal archived Work Item.
                if not outcome_identity_valid or (
                    not outcome_green and not recovery_receipt_valid
                ):
                    findings.append(
                        finding(
                            work_item,
                            "invalid_outcome",
                            str(outcome_path.relative_to(repo)),
                        )
                    )
            evidence = f".ai/evidence/{work_item}.verification.json"
            evidence_path = repo / evidence
            if not evidence_path.is_file() or evidence_path.is_symlink():
                findings.append(finding(work_item, "missing_evidence", evidence))
            else:
                try:
                    evidence_value = load_json(evidence_path)
                except ValueError:
                    evidence_value = {}
                try:
                    project_value = load_json(repo / ".ai/project.json")
                except ValueError:
                    project_value = {}
                runtime_digest = evidence_value.get("runtimeDigest")
                runtime_shape_valid = (
                    "runtimeDigest" not in evidence_value
                    or (
                        isinstance(runtime_digest, str)
                        and re.fullmatch(r"sha256:[0-9a-f]{64}", runtime_digest)
                        and isinstance(evidence_value.get("runtimeVersion"), str)
                        and bool(evidence_value["runtimeVersion"])
                    )
                )
                if (
                    evidence_value.get("workItemId") != work_item
                    or evidence_value.get("passed") is not True
                    or evidence_value.get("repositoryId", project_value.get("repositoryId"))
                    != project_value.get("repositoryId")
                    or not runtime_shape_valid
                ):
                    findings.append(finding(work_item, "invalid_evidence", evidence))
            decision_paths = (
                f".ai/decisions/{work_item}.close.json",
                f".ai/decisions/{work_item}.recovery.json",
            )
            decision = None
            close_path = repo / ".ai/decisions" / f"{work_item}.close.json"
            # A recovery receipt explains a predecessor's history; it must not
            # shadow a later valid close decision for the same Work Item.
            if close_path.is_file() and not close_path.is_symlink():
                decision = str(close_path.relative_to(repo))
                try:
                    decision_value = load_json(close_path)
                except ValueError:
                    decision_value = {}
                if valid_close_decision(repo, work_item, decision_value):
                    record["decisionPath"] = decision
                    record["lifecycleState"] = "closed"
                elif recovery_receipt_valid:
                    # A predecessor may already contain an immutable, but
                    # non-canonical, close receipt when a later recovery
                    # explicitly supersedes it.  The recovery receipt is the
                    # authoritative terminal projection in that case; do not
                    # reclassify the predecessor as invalid merely because
                    # its historical close cannot be rewritten.
                    decision = str(recovery_path.relative_to(repo))
                    record["decisionPath"] = decision
                    record["lifecycleState"] = "recovered"
                else:
                    record["lifecycleState"] = "closure_invalid"
                    findings.append(finding(work_item, "invalid_terminal_decision", decision))
            elif recovery_path.is_file() and not recovery_path.is_symlink():
                if recovery_receipt_valid:
                    decision = str(recovery_path.relative_to(repo))
                    record["decisionPath"] = decision
                    record["lifecycleState"] = "recovered"
                else:
                    record["lifecycleState"] = "closure_invalid"
                    findings.append(
                        finding(
                            work_item,
                            "invalid_terminal_decision",
                            str(recovery_path.relative_to(repo)),
                        )
                    )
            if decision is None:
                finalize_path = f".ai/decisions/{work_item}.finalize.json"
                if (repo / finalize_path).is_file():
                    try:
                        finalize_value = load_json(repo / finalize_path)
                    except ValueError:
                        finalize_value = {}
                    finalize_valid, phase = premerge_finalize_state(
                        repo, work_item, finalize_value
                    )
                    if stale_awaiting_merge_close(repo, finalize_value, phase):
                        decision = finalize_path
                        record["decisionPath"] = decision
                        record["lifecycleState"] = "stale_awaiting_merge_close"
                        findings.append(
                            finding(work_item, "stale_awaiting_merge_close", finalize_path)
                        )
                    else:
                        if finalize_valid and phase in {
                            "feature_branch",
                            "pull_request",
                            "release_tag",
                        }:
                            decision = finalize_path
                            record["decisionPath"] = decision
                            record["lifecycleState"] = "awaiting_merge_close"
                        else:
                            record["lifecycleState"] = "closure_missing"
                            code = (
                                "premerge_finalize_not_applicable"
                                if finalize_valid
                                else "invalid_premerge_finalize"
                            )
                            findings.append(finding(work_item, code, finalize_path))
                            findings.append(
                                finding(
                                    work_item,
                                    "missing_terminal_decision",
                                    decision_paths[0],
                                )
                            )
                else:
                    record["lifecycleState"] = "closure_missing"
                    findings.append(
                        finding(work_item, "missing_terminal_decision", decision_paths[0])
                    )
            work_item_rows = rows.get(short_id(work_item), {})
            pending_entry = pending_by_work_item.get(work_item)
            pending_valid = False
            if pending_entry is not None:
                try:
                    pending_finalize = load_json(
                        repo
                        / ".ai/decisions"
                        / f"{work_item}.finalize.json"
                    )
                except ValueError:
                    pending_phase = "unknown"
                else:
                    pending_pull_request = pending_finalize.get("pullRequest")
                    pending_base_branch = (
                        pending_pull_request.get("baseBranch")
                        if isinstance(pending_pull_request, dict)
                        else None
                    )
                    pending_phase = (
                        repository_phase(repo, pending_base_branch)
                        if isinstance(pending_base_branch, str)
                        and pending_base_branch
                        else "unknown"
                    )
                pending_stale = (
                    bool(work_item_rows)
                    or pending_phase == "default_branch"
                    or record.get("lifecycleState")
                    in {"closed", "recovered", "stale_awaiting_merge_close"}
                )
                if pending_stale:
                    findings.append(
                        finding(
                            work_item,
                            "stale_pending_parity_registration",
                            PENDING_PARITY_REGISTRY,
                        )
                    )
                else:
                    pending_valid = validate_pending_parity_entry(
                        repo,
                        pending_entry,
                        work_item,
                        rows,
                        str(record.get("lifecycleState", "unknown")),
                    )
                    if pending_valid:
                        record["lifecycleState"] = "pending_parity_registration"
                        record["pendingParityRegistryPath"] = PENDING_PARITY_REGISTRY
                    else:
                        findings.append(
                            finding(
                                work_item,
                                "invalid_pending_parity_registration",
                                PENDING_PARITY_REGISTRY,
                            )
                        )
            for parity_doc, implemented in PARITY_DOCS:
                line = work_item_rows.get(parity_doc)
                if line is None:
                    if not pending_valid:
                        findings.append(
                            finding(work_item, "missing_parity_entry", parity_doc)
                        )
                    continue
                if evidence not in line:
                    findings.append(finding(work_item, "missing_parity_evidence", parity_doc))
                if decision is not None and decision not in line:
                    findings.append(finding(work_item, "missing_parity_decision", parity_doc))
                lifecycle_status = (
                    "进行中 → 验证关闭后已实现"
                    if parity_doc.endswith(".zh-CN.md")
                    else (
                        "In progress → verified close 後 Implemented"
                        if parity_doc.endswith(".ja.md")
                        else "In progress → Implemented after verified close"
                    )
                )
                if f"| {lifecycle_status} |" in line:
                    lifecycle_records = (
                        f".ai/work-items/archive/{work_item}.contract.json",
                        evidence,
                        f".ai/decisions/{work_item}.finalize.json",
                        f".ai/decisions/{work_item}.close.json",
                    )
                    if any(f"`{relative}`" not in line for relative in lifecycle_records):
                        findings.append(
                            finding(
                                work_item,
                                "invalid_prearchive_parity_registration",
                                parity_doc,
                            )
                        )
                    elif not _parity_row_precedes_record(
                        repo,
                        parity_doc,
                        line,
                        evidence,
                    ):
                        findings.append(
                            finding(
                                work_item,
                                "stale_prearchive_parity_registration",
                                parity_doc,
                            )
                        )
                status_tokens = (implemented,)
                if record.get("lifecycleState") == "recovered":
                    recovery_status = "已恢复" if parity_doc.endswith(".zh-CN.md") else "Recovered"
                    status_tokens = (implemented, recovery_status)
                elif record.get("lifecycleState") == "awaiting_merge_close":
                    pending_status = "进行中" if parity_doc.endswith(".zh-CN.md") else "In progress"
                    status_tokens = (implemented, pending_status)
                if not any(token in line for token in status_tokens):
                    findings.append(finding(work_item, "stale_parity_status", parity_doc))
        inventory.append(record)

    problems: list[dict[str, str]] = []
    for path in sorted((repo / ".ai/problems").glob("*.json")):
        try:
            problem = load_json(path)
        except ValueError:
            findings.append(finding("repository", "unknown_problem", str(path.relative_to(repo))))
            continue
        item = {
            "id": str(problem.get("id", path.stem)),
            "state": str(problem.get("state", "unknown")),
            "workItemId": str(problem.get("workItemId", "unknown")),
        }
        problems.append(item)
        if item["state"] == "unknown":
            findings.append(finding(item["workItemId"], "unknown_problem", str(path.relative_to(repo))))

    findings = sorted(
        {json.dumps(item, sort_keys=True): item for item in findings}.values(),
        key=lambda item: (item["workItemId"], item["code"], item["path"]),
    )
    legacy_warnings = [item for item in findings if item["severity"] == "historical"]
    findings = [item for item in findings if item["severity"] != "historical"]
    inventory.sort(key=lambda item: item["workItemId"])
    problems.sort(key=lambda item: (item["workItemId"], item["id"]))
    blocking_findings = [item for item in findings if item["severity"] == "error"]
    report: dict[str, Any] = {
        "currentCycleStart": cycle_start.isoformat().replace("+00:00", "Z") if cycle_start else None,
        "currentRelease": release,
        "findings": findings,
        "inventory": inventory,
        "legacyWarnings": legacy_warnings,
        "problems": problems,
        "schemaVersion": 1,
        "state": "passed" if not blocking_findings else "failed",
    }
    if metadata_error is not None:
        report["metadataError"] = metadata_error
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if blocking_findings:
        for item in blocking_findings:
            print(f"{item['workItemId']}: {item['code']}: {item['path']}", file=sys.stderr)
        return 1
    print(f"governance integrity gate passed: {len(inventory)} work items")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
