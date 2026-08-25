#!/usr/bin/env python3
import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path


def write(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(value, str):
        path.write_text(value, encoding="utf-8")
    else:
        path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


parser = argparse.ArgumentParser()
parser.add_argument("--spec", required=True)
parser.add_argument("--output", required=True)
args = parser.parse_args()
spec = json.loads(Path(args.spec).read_text(encoding="utf-8"))
root = Path(args.output)
shutil.rmtree(root, ignore_errors=True)

repository_id = "sha256:" + "b" * 64
runtime_digest = "sha256:" + "a" * 64
runtime_version = "0.2.23"
version = spec.get("currentRelease", "9.9.9")
metadata = {"packages": [{"name": "fixture-package", "version": version, "source": None}]}
write(root / "cargo-metadata.json", metadata)
write(root / ".ai/project.json", {"repositoryId": repository_id})
write(
    root / "docs/reference/pending-parity-registry.json",
    {"schemaVersion": 1, "entries": []},
)

rows = {"en": [], "zh": [], "ja": []}
for item in spec.get("workItems", []):
    work_item = item["id"]
    classification = item.get("classification", "current_release")
    archive = root / ".ai/work-items/archive"
    is_current = classification in {"current_release", "current_archive_timestamp"}
    created_at = "2026-02-01T00:00:00Z" if is_current else "2025-01-01T00:00:00Z"
    decision_kind = item.get("terminalDecision", "close")
    base_revision = "2" * 40
    contract = {
        "baseRevision": base_revision,
        "createdAt": created_at,
        "repositoryId": repository_id,
        "workItemId": work_item,
    }
    if classification == "current_archive_timestamp":
        contract.pop("createdAt")
    if decision_kind == "finalize":
        receipt_base_branch = item.get("receiptBaseBranch", "main")
        receipt_branch = "codex/fixture"
        receipt_url = "https://github.com/example/fixture/pull/999"
        receipt_worktree = str(root.resolve())
        contract["resourceContext"] = {
            "baseBranch": receipt_base_branch,
            "baseRemote": "origin",
            "branch": receipt_branch,
            "provider": "github",
            "pullRequest": receipt_url,
            "worktree": receipt_worktree,
        }
    contract_path = archive / f"{work_item}.contract.json"
    write(
        contract_path,
        contract,
    )
    contract_digest = "sha256:" + hashlib.sha256(contract_path.read_bytes()).hexdigest()
    if is_current:
        write(archive / f"{work_item}.summary.json", {"workItemId": work_item})
        write(
            archive / f"{work_item}.archive.json",
            {
                "createdAt": "2026-03-01T00:00:00Z",
                "workItemId": work_item,
                "state": "superseded" if decision_kind == "recovery" else "archived",
            },
        )
        write(
            archive / f"{work_item}.outcome.json",
            {
                "workItemId": work_item,
                "decisionState": "red" if decision_kind == "recovery" else "green",
                "state": "blocked" if decision_kind == "recovery" else "finish_ready",
            },
        )
        evidence = f".ai/evidence/{work_item}.verification.json"
        decision = f".ai/decisions/{work_item}.{decision_kind}.json"
        write(
            root / evidence,
            {
                "passed": True,
                "repositoryId": repository_id,
                "runtimeDigest": runtime_digest,
                "runtimeVersion": runtime_version,
                "workItemId": work_item,
                "state": "verified",
            },
        )
        if decision_kind == "close":
            write(
                root / decision,
                {
                    "workItemId": work_item,
                    "repositoryId": repository_id,
                    "state": "closed",
                    "decisionState": "confirmed",
                    "humanDecision": "approved",
                    "structuredDecision": {
                        "decision": "approved",
                        "actor": "fixture-human",
                        "authoritySource": "fixture-policy",
                        "reason": "Fixture close decision.",
                        "decidedAt": "2026-03-01T00:00:00Z",
                        "resumeCondition": "None.",
                        "evidenceRefs": [evidence],
                        "policyRefs": ["fixture-policy"],
                    },
                },
            )
        elif decision_kind == "recovery":
            write(
                root / decision,
                {
                    "schemaVersion": 1,
                    "workItemId": work_item,
                    "predecessorWorkItemId": work_item,
                    "successorWorkItemId": "WI-901-successor",
                    "decision": "supersede",
                    "repositoryId": repository_id,
                    "reason": "The predecessor is preserved as immutable blocked history.",
                    "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
                },
            )
        elif decision_kind == "finalize":
            invalid_finalize = item.get("invalidFinalize")
            finalize_after = {
                "branch": "present",
                "pullRequest": "unmerged",
                "worktree": "clean",
            }
            if invalid_finalize == "deleted_branch":
                finalize_after["branch"] = "deleted"
            finalize_result = {
                "disposition": "blocked",
                "failureCodes": ["unmerged_pull_request"],
                "unknownCodes": [],
            }
            if invalid_finalize == "retained":
                finalize_result["disposition"] = "retained"
                finalize_result["failureCodes"] = []
            finalize_repository_id = (
                "sha256:" + "f" * 64
                if invalid_finalize == "foreign"
                else repository_id
            )
            head_revision = "1" * 40
            write(
                root / decision,
                {
                    "after": finalize_after,
                    "branch": {
                        "headRevision": head_revision,
                        "name": receipt_branch,
                        "remote": "origin",
                    },
                    "contractDigest": contract_digest,
                    "provider": "github",
                    "pullRequest": {
                        "baseBranch": receipt_base_branch,
                        "baseRemote": "origin",
                        "baseRevision": base_revision,
                        "headRevision": head_revision,
                        "mergeCommit": None,
                        "number": 999,
                        "url": receipt_url,
                    },
                    "repositoryId": finalize_repository_id,
                    "resourceContext": contract["resourceContext"],
                    "result": finalize_result,
                    "reason": item.get(
                        "finalizeReason",
                        "awaiting_merge_close: fixture PR remains unmerged",
                    ),
                    "runtimeDigest": runtime_digest,
                    "runtimeVersion": runtime_version,
                    "worktree": {
                        "branch": receipt_branch,
                        "headRevision": head_revision,
                        "path": receipt_worktree,
                        "worktreeId": work_item,
                    },
                    "workItemId": work_item,
                },
            )
        else:
            raise SystemExit(f"unsupported terminalDecision fixture: {decision_kind}")
    else:
        evidence = "historical evidence intentionally exempt"
        decision = "historical decision intentionally exempt"
    short = work_item.split("-", 2)[:2]
    short_id = "-".join(short)
    statuses = (
        ("Implemented", "已实现", "Implemented")
        if is_current
        else ("Legacy", "历史", "Legacy")
    )
    rows["en"].append(f"| {short_id} — fixture | {statuses[0]} | `{evidence}`; `{decision}` |")
    rows["zh"].append(f"| {short_id} — fixture | {statuses[1]} | `{evidence}`；`{decision}` |")
    rows["ja"].append(f"| {short_id} — fixture | {statuses[2]} | `{evidence}`；`{decision}` |")

for language, filename in (
    ("en", "reference-parity.md"),
    ("zh", "reference-parity.zh-CN.md"),
    ("ja", "reference-parity.ja.md"),
):
    write(root / "docs/reference" / filename, "\n".join(rows[language]) + "\n")

mutation = spec.get("mutation")
current = next(
    (
        item
        for item in spec.get("workItems", [])
        if item.get("classification") in {"current_release", "current_archive_timestamp"}
    ),
    None,
)
mutation_item = next(
    (
        item
        for item in spec.get("workItems", [])
        if item.get("terminalDecision") == "recovery"
    ),
    current,
)
if current:
    work_item = current["id"]
    if mutation == "missing_work_item":
        (root / ".ai/work-items/archive" / f"{work_item}.contract.json").unlink()
    elif mutation == "missing_evidence":
        (root / ".ai/evidence" / f"{work_item}.verification.json").unlink()
    elif mutation == "missing_close":
        (root / ".ai/decisions" / f"{work_item}.close.json").unlink()
    elif mutation == "invalid_outcome":
        write(
            root / ".ai/work-items/archive" / f"{work_item}.outcome.json",
            {"workItemId": work_item, "decisionState": "red"},
        )
    elif mutation == "invalid_recovery":
        recovery_work_item = mutation_item["id"]
        recovery_path = root / ".ai/decisions" / f"{recovery_work_item}.recovery.json"
        value = json.loads(recovery_path.read_text(encoding="utf-8"))
        value["repositoryId"] = "sha256:" + "f" * 64
        write(recovery_path, value)

for problem in spec.get("problems", []):
    write(root / ".ai/problems" / f"{problem['id']}.json", problem)

repository_phase = spec.get("repositoryPhase")
if repository_phase:
    feature_finalize_work_items = []
    feature_finalize_receipts = {}
    if repository_phase == "feature":
        # Model the canonical finalization receipt as the append-only
        # governance commit that it represents. Keeping it out of the
        # initial base commit prevents the fixture from requiring an unsafe
        # in-place modification when the gate checks the append range.
        for release_item in spec.get("workItems", []):
            if release_item.get("terminalDecision") != "finalize":
                continue
            receipt_path = root / ".ai/decisions" / f"{release_item['id']}.finalize.json"
            if receipt_path.is_file():
                feature_finalize_work_items.append(release_item["id"])
                feature_finalize_receipts[release_item["id"]] = json.loads(
                    receipt_path.read_text(encoding="utf-8")
                )
                receipt_path.unlink()
    subprocess.run(["git", "init", "-q", "-b", "main"], cwd=root, check=True)
    subprocess.run(["git", "config", "user.name", "Fixture"], cwd=root, check=True)
    subprocess.run(["git", "config", "user.email", "fixture@example.invalid"], cwd=root, check=True)
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    subprocess.run(["git", "commit", "-qm", "fixture"], cwd=root, check=True)
    subprocess.run(
        ["git", "update-ref", "refs/remotes/origin/main", "refs/heads/main"],
        cwd=root,
        check=True,
    )
    subprocess.run(
        ["git", "symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"],
        cwd=root,
        check=True,
    )
    if repository_phase == "feature":
        # The canonical finalization receipt is an append-only governance
        # commit. Bind it to the immediately preceding reviewed head so the
        # gate can allow that receipt append while rejecting later code drift.
        feature_head = subprocess.run(
            ["git", "rev-parse", "HEAD^{commit}"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        for work_item in feature_finalize_work_items:
            receipt_path = root / ".ai/decisions" / f"{work_item}.finalize.json"
            receipt_path.parent.mkdir(parents=True, exist_ok=True)
            receipt = feature_finalize_receipts[work_item]
            receipt["branch"]["headRevision"] = feature_head
            receipt["pullRequest"]["headRevision"] = feature_head
            receipt["worktree"]["headRevision"] = feature_head
            write(receipt_path, receipt)
        if any(item.get("terminalDecision") == "finalize" for item in spec.get("workItems", [])):
            finalize_paths = [
                str(path.relative_to(root))
                for path in (root / ".ai/decisions").glob("*.finalize.json")
            ]
            subprocess.run(["git", "add", *finalize_paths], cwd=root, check=True)
            subprocess.run(
                ["git", "commit", "-qm", "bind fixture finalization"],
                cwd=root,
                check=True,
            )
        subprocess.run(["git", "checkout", "-qb", "codex/fixture"], cwd=root, check=True)
    elif repository_phase in {"release_tag", "main_merged"}:
        # Model the provider merge that makes a pre-merge head an ancestor of
        # the immutable release tag.  Update the fixture receipt with the
        # actual feature commit after the history exists.
        subprocess.run(["git", "checkout", "-qb", "codex/fixture"], cwd=root, check=True)
        write(root / "fixture-change.txt", "merged feature\n")
        subprocess.run(["git", "add", "fixture-change.txt"], cwd=root, check=True)
        subprocess.run(["git", "commit", "-qm", "fixture change"], cwd=root, check=True)
        feature_head = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=root, check=True, capture_output=True, text=True
        ).stdout.strip()
        release_item = next(
            item
            for item in spec.get("workItems", [])
            if item.get("terminalDecision") == "finalize"
        )
        receipt_path = root / ".ai/decisions" / f"{release_item['id']}.finalize.json"
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipt["branch"]["headRevision"] = feature_head
        receipt["pullRequest"]["headRevision"] = feature_head
        receipt["worktree"]["headRevision"] = feature_head
        if spec.get("mutation") == "non_ancestor":
            receipt["branch"]["headRevision"] = "f" * 40
            receipt["pullRequest"]["headRevision"] = "f" * 40
            receipt["worktree"]["headRevision"] = "f" * 40
        write(receipt_path, receipt)
        subprocess.run(["git", "add", str(receipt_path.relative_to(root))], cwd=root, check=True)
        subprocess.run(["git", "commit", "-qm", "bind fixture finalization"], cwd=root, check=True)
        subprocess.run(["git", "checkout", "-q", "main"], cwd=root, check=True)
        subprocess.run(["git", "merge", "--no-ff", "-m", "merge fixture", "codex/fixture"], cwd=root, check=True)
        tagged_head = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=root, check=True, capture_output=True, text=True
        ).stdout.strip()
        if repository_phase == "release_tag":
            subprocess.run(["git", "tag", "v9.9.9", tagged_head], cwd=root, check=True)
    elif repository_phase != "main":
        raise SystemExit(f"unsupported repositoryPhase fixture: {repository_phase}")
