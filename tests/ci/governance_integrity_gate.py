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
RECORD_SUFFIXES = ("contract", "summary", "archive", "outcome")


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


def default_branch_contains_reviewed_head(repo: Path, head_revision: str) -> bool:
    """Return true only when the exact reviewed PR head is in the checkout.

    This is deliberately narrower than a provider merge lookup: a merge (or
    an equivalent reviewed integration) is proven only when the immutable PR
    head is an ancestor of the synchronized default-branch checkout. If a
    provider used squash/rebase and the exact head cannot be proven, the gate
    stays unknown and the normal explicit finalization/close path remains
    required.
    """
    if not re.fullmatch(r"[0-9a-f]{40}", head_revision):
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
            ["git", "merge-base", "--is-ancestor", head_revision, current.stdout.strip()],
            cwd=repo,
            check=False,
            capture_output=True,
        ).returncode
        == 0
    )


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
    reason_valid = isinstance(reason, str) and (
        reason == "awaiting_merge_close"
        or reason.startswith("awaiting_merge_close: ")
    )
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
        and isinstance(head_revision, str)
        and re.fullmatch(r"[0-9a-f]{40}", head_revision) is not None
        and branch_identity.get("headRevision") == head_revision
        and worktree.get("headRevision") == head_revision
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
            phase in {"feature_branch", "pull_request"}
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
    return (
        value.get("schemaVersion") == 1
        and value.get("workItemId") == work_item
        and value.get("predecessorWorkItemId") == work_item
        and isinstance(value.get("successorWorkItemId"), str)
        and bool(value.get("successorWorkItemId"))
        and value.get("decision") in {"supersede", "successor", "retry"}
        and isinstance(repository_id, str)
        and value.get("repositoryId") == repository_id
        and isinstance(evidence_refs, list)
        and bool(evidence_refs)
        and all(isinstance(item, str) and item for item in evidence_refs)
        and isinstance(value.get("reason"), str)
        and bool(value["reason"].strip())
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

        if location == "archive":
            if not archive_digests_valid(repo, work_item):
                findings.append(
                    finding(
                        work_item,
                        "archive_digest_mismatch",
                        f".ai/work-items/archive/{work_item}.archive.json",
                    )
                )
            recovery_path = repo / ".ai/decisions" / f"{work_item}.recovery.json"
            recovery_value: dict[str, Any] = {}
            recovery_receipt_valid = False
            if recovery_path.is_file() and not recovery_path.is_symlink():
                try:
                    recovery_value = load_json(recovery_path)
                except ValueError:
                    recovery_value = {}
                recovery_receipt_valid = valid_recovery_decision(
                    repo, work_item, recovery_value
                )

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
            for parity_doc, implemented in PARITY_DOCS:
                line = work_item_rows.get(parity_doc)
                if line is None:
                    findings.append(finding(work_item, "missing_parity_entry", parity_doc))
                    continue
                if evidence not in line:
                    findings.append(finding(work_item, "missing_parity_evidence", parity_doc))
                if decision is not None and decision not in line:
                    findings.append(finding(work_item, "missing_parity_decision", parity_doc))
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
