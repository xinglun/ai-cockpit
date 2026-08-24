#!/usr/bin/env python3
"""Plan and apply repository-owned post-close documentation promotion."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from promote_closed_work_item import (
    PromotionError,
    TerminalEvidence,
    planned_changes,
    promote,
    raw_digest,
    validate_terminal_evidence,
)


PLAN_KIND = "ai-cockpit.post-close-documentation-plan"


class PostCloseError(RuntimeError):
    """A fail-closed post-close orchestration error."""


def require_keys(value: dict[str, Any], expected: set[str], label: str) -> None:
    unknown = sorted(set(value) - expected)
    missing = sorted(expected - set(value))
    if unknown:
        raise PostCloseError(f"unknown plan field in {label}: {', '.join(unknown)}")
    if missing:
        raise PostCloseError(f"missing plan field in {label}: {', '.join(missing)}")


def reject_duplicate_plan_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise PostCloseError(f"post-close plan has duplicate key {key!r}")
        value[key] = item
    return value


def digest_text(text: str) -> str:
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def git(repository: Path, *arguments: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(repository), *arguments],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "git command failed"
        raise PostCloseError(detail)
    return result.stdout.strip()


def git_context(repository: Path) -> tuple[str, str]:
    root = Path(git(repository, "rev-parse", "--show-toplevel")).resolve()
    if root != repository:
        raise PostCloseError("repository path is not the Git worktree root")
    revision = git(repository, "rev-parse", "HEAD")
    base_revision = git(repository, "rev-parse", "refs/remotes/origin/main")
    if revision != base_revision:
        raise PostCloseError(
            "post-close repository is not exactly synchronized with origin/main"
        )
    return revision, base_revision


def dirty_paths(repository: Path) -> set[str]:
    result = subprocess.run(
        [
            "git",
            "-C",
            str(repository),
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise PostCloseError(result.stderr.strip() or "Git status failed")
    output = result.stdout
    paths: set[str] = set()
    entries = iter(output.split("\0"))
    for entry in entries:
        if not entry:
            continue
        if len(entry) < 4 or entry[2] != " ":
            raise PostCloseError("Git status output is malformed")
        status = entry[:2]
        paths.add(entry[3:])
        if "R" in status or "C" in status:
            source = next(entries, "")
            if source:
                paths.add(source)
    return paths


def terminal_bindings(
    repository: Path, evidence: TerminalEvidence
) -> dict[str, Any]:
    paths = {
        "archive": f".ai/work-items/archive/{evidence.work_item_id}.archive.json",
        "contract": evidence.archive_path,
        "verification": evidence.evidence_path,
        "finalization": evidence.finalization_path,
        "close": evidence.close_path,
    }
    return {
        name: {"digest": raw_digest(repository / path), "path": path}
        for name, path in paths.items()
    } | {"finalizationSequence": 2}


def build_plan(repository: Path, work_item_id: str) -> dict[str, Any]:
    evidence = validate_terminal_evidence(repository, work_item_id)
    revision, base_revision = git_context(repository)
    if dirty_paths(repository):
        raise PostCloseError("post-close planning requires a clean repository")
    changes = planned_changes(repository, evidence)
    planned = []
    for path in sorted(changes, key=lambda candidate: candidate.relative_to(repository).as_posix()):
        planned.append(
            {
                "afterDigest": digest_text(changes[path]),
                "beforeDigest": raw_digest(path),
                "path": path.relative_to(repository).as_posix(),
            }
        )
    return {
        "base": {"branch": "main", "remote": "origin", "revision": base_revision},
        "changes": planned,
        "kind": PLAN_KIND,
        "repositoryId": evidence.repository_id,
        "repositoryRevision": revision,
        "schemaVersion": 1,
        "terminalEvidence": terminal_bindings(repository, evidence),
        "workItemId": work_item_id,
    }


def write_plan(path: Path, plan: dict[str, Any]) -> None:
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise PostCloseError("plan output must be a regular non-symlink path")
    text = json.dumps(plan, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary_path = Path(temporary)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            stream.write(text)
            stream.flush()
            os.fsync(stream.fileno())
        mode = stat.S_IMODE(path.stat().st_mode) if path.exists() else 0o600
        os.chmod(temporary_path, mode)
        os.replace(temporary_path, path)
    finally:
        try:
            temporary_path.unlink()
        except FileNotFoundError:
            pass


def read_plan(path: Path) -> dict[str, Any]:
    try:
        mode = path.lstat().st_mode
    except OSError as error:
        raise PostCloseError(
            "plan input must be a regular non-symlink file"
        ) from error
    if stat.S_ISLNK(mode) or not stat.S_ISREG(mode):
        raise PostCloseError("plan input must be a regular non-symlink file")
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_plan_keys,
        )
    except PostCloseError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise PostCloseError(f"post-close plan is malformed: {error}") from error
    if not isinstance(value, dict):
        raise PostCloseError("post-close plan top level must be an object")
    return value


def apply_plan(
    repository: Path, work_item_id: str, plan: dict[str, Any]
) -> dict[str, Any]:
    evidence = validate_terminal_evidence(repository, work_item_id)
    revision, base_revision = git_context(repository)
    require_keys(
        plan,
        {
            "base",
            "changes",
            "kind",
            "repositoryId",
            "repositoryRevision",
            "schemaVersion",
            "terminalEvidence",
            "workItemId",
        },
        "root",
    )
    if plan.get("kind") != PLAN_KIND or plan.get("schemaVersion") != 1:
        raise PostCloseError("post-close plan kind or schema mismatch")
    if plan.get("repositoryId") != evidence.repository_id:
        raise PostCloseError("post-close plan repository identity mismatch")
    if plan.get("workItemId") != work_item_id:
        raise PostCloseError("post-close plan Work Item identity mismatch")
    if plan.get("repositoryRevision") != revision:
        raise PostCloseError("post-close plan repository revision is stale")
    if plan.get("base") != {
        "branch": "main",
        "remote": "origin",
        "revision": base_revision,
    }:
        raise PostCloseError("post-close plan base identity mismatch")
    if plan.get("terminalEvidence") != terminal_bindings(repository, evidence):
        raise PostCloseError("post-close plan terminal evidence mismatch")

    expected_changes = planned_changes(repository, evidence)
    raw_changes = plan.get("changes")
    if not isinstance(raw_changes, list):
        raise PostCloseError("post-close plan changes are missing")
    for index, change in enumerate(raw_changes):
        if not isinstance(change, dict):
            raise PostCloseError(f"post-close plan change {index} is malformed")
        require_keys(
            change,
            {"afterDigest", "beforeDigest", "path"},
            f"changes[{index}]",
        )
    by_path = {
        path.relative_to(repository).as_posix(): (path, text)
        for path, text in expected_changes.items()
    }
    if [change["path"] for change in raw_changes] != sorted(by_path):
        raise PostCloseError("post-close plan paths do not match the six controlled paths")

    states: set[str] = set()
    for change in raw_changes:
        try:
            path, after_text = by_path[change["path"]]
        except (KeyError, TypeError) as error:
            raise PostCloseError(
                "post-close plan path is not a controlled documentation path"
            ) from error
        if change.get("afterDigest") != digest_text(after_text):
            raise PostCloseError("post-close plan after digest mismatch")
        current_digest = raw_digest(path)
        if current_digest == change.get("beforeDigest"):
            states.add("before")
        elif current_digest == change.get("afterDigest"):
            states.add("after")
        else:
            raise PostCloseError("post-close plan current document digest mismatch")
    if len(states) != 1:
        raise PostCloseError("post-close documentation is partially projected")

    controlled_paths = set(by_path)
    observed_dirty = dirty_paths(repository)
    unexpected_dirty = sorted(observed_dirty - controlled_paths)
    if unexpected_dirty:
        raise PostCloseError(
            "unexpected dirty path before post-close apply: "
            + ", ".join(unexpected_dirty)
        )
    if states == {"before"} and observed_dirty:
        raise PostCloseError("controlled documentation is dirty before post-close apply")
    if states == {"after"} and observed_dirty not in (set(), controlled_paths):
        raise PostCloseError("post-close documentation is partially projected")

    report = promote(repository, work_item_id, check=False)
    promote(repository, work_item_id, check=True)
    return {
        "changedPaths": report["changedPaths"],
        "state": "promoted" if states == {"before"} else "current",
        "workItemId": work_item_id,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--work-item", required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--plan-out", type=Path)
    mode.add_argument("--apply-plan", type=Path)
    args = parser.parse_args()
    repository = args.repo.resolve()
    try:
        if args.plan_out is not None:
            plan = build_plan(repository, args.work_item)
            write_plan(args.plan_out, plan)
            report = {
                "changedPaths": [change["path"] for change in plan["changes"]],
                "planPath": str(args.plan_out),
                "state": "planned",
                "workItemId": args.work_item,
            }
        else:
            report = apply_plan(
                repository, args.work_item, read_plan(args.apply_plan)
            )
        print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    except (PostCloseError, PromotionError) as error:
        print(f"post-close Work Item orchestration failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
