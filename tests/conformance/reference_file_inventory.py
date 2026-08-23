#!/usr/bin/env python3
"""Build and validate the pinned reference-source file comparison ledger.

The reference repository is a specification corpus, not a source tree to copy.
This tool records every tracked reference path, gives it one explicit staged
classification, and validates the first comparison batch.  Later batches may
replace ``deferred-next-batch`` with an evidence-backed result without changing
the pinned source revision.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


ALLOWED_CLASSIFICATIONS = {
    "implemented-equivalent",
    "implemented-different-by-design",
    "migrate-gap",
    "not-applicable",
    "reference-only",
    "generated-history",
    "deferred-next-batch",
}
FIRST_BATCH = "governance-entrypoints"
GETTING_STARTED_BATCH = "getting-started-onboarding"
EXPECTED_REFERENCE_COMMIT = "e5acb677da6621004d96f0ef353c58fe8d3acfbf"
EXPECTED_TARGET_COMMIT = "46e426625a8cae450f1190d0bdbafd6d8e648a90"
CAPABILITY_STATUS_BATCH = "capability-status-projection"
CAPABILITY_STATUS_RECORDS: dict[str, tuple[str, list[str], str]] = {
    ".ai/project/adopter-capability-manifest.json": (
        "migrate-gap",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            "crates/cockpit-mcp/src/lib.rs",
            "docs/capabilities.md",
        ],
        "No exact Rust counterpart exists for the reference installed-surface manifest: the Runtime-native registry binds current Runtime/repository truth, while templateFiles, installedFiles, schemas, entrypoint checks, verifyInstalledSurface, and adopter acceptance remain a deferred boundary.",
    ),
    ".ai/project/capabilities.json": (
        "migrate-gap",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "No exact Rust counterpart exists for the reference repository-authored capability declaration: the current projection reports observed and Runtime-supported truth only, and never infers adopter acceptance.",
    ),
    ".ai/project/success_criteria.json": (
        "migrate-gap",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/commands.md",
        ],
        "No exact Rust counterpart exists for the reference project-level capability/intent guard criteria; Contract acceptance and Summary/Outcome evidence cover per-Work-Item completion but do not prove capability-to-scope mappings.",
    ),
    ".ai/project_profile.yaml": (
        "migrate-gap",
        [
            ".ai/project.json",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "No exact Rust counterpart exists for the complete reference project-profile policy surface; .ai/project.json supplies only strict repository profile and identity facts.",
    ),
    ".ai/cockpit/work-items/index.json": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            "crates/cockpit-mcp/src/lib.rs",
            "docs/reference/commands.md",
        ],
        "A deterministic request-scoped all-Work-Item status index replaces the tracked generated file and exposes counts, diagnostics, snapshot binding, and an index digest.",
    ),
    ".ai/cockpit/work-items/wi-06-status-interface.status.json": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            "crates/cockpit-mcp/src/lib.rs",
            "docs/reference/commands.md",
        ],
        "The request-scoped Work Item status snapshot exposes evidence-bound lifecycle facts without persisting a per-item status file.",
    ),
}


def git_paths(repository: Path) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(repository), "ls-tree", "-r", "--name-only", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return [line for line in result.stdout.splitlines() if line]


def git_working_paths(repository: Path) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(repository), "ls-files", "--others", "--exclude-standard"],
        check=True,
        capture_output=True,
        text=True,
    )
    return [line for line in result.stdout.splitlines() if line]


def git_head(repository: Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(repository), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def digest_paths(paths: list[str]) -> str:
    payload = "\n".join(sorted(paths)) + "\n"
    return "sha256:" + hashlib.sha256(payload.encode()).hexdigest()


def is_generated_history(path: str) -> bool:
    generated_prefixes = (
        ".ai/decisions/",
        ".ai/work-items/",
        ".ai/evidence/",
        ".ai/knowledge/",
        ".ai/calibration/",
        "docs/audits/",
        "docs/archive/",
        "docs/releases/",
        "docs/work-items/",
        "docs/superpowers/",
    )
    generated_names = {
        ".ai/cockpit/current_status.md",
        ".ai/cockpit/derived_artifacts.json",
        ".ai/cockpit/provenance.json",
        ".ai/cockpit/release-digests.json",
        ".ai/cockpit/release-freeze.json",
        ".ai/cockpit/sbom.json",
        ".ai/cockpit/system_invariants.json",
        ".ai/cockpit/task_report.json",
        ".ai/cockpit/task_report.md",
        ".ai/cockpit/version.json",
    }
    return path.startswith(generated_prefixes) or path in generated_names


def is_governance_entrypoint(path: str) -> bool:
    exact = {
        "AGENTS.md",
        "CLAUDE.md",
        "GEMINI.md",
        ".cursor/rules/ai-cockpit.mdc",
        ".ai/README.md",
        ".ai/glossary.md",
        ".ai/cockpit/README.md",
        ".ai/cockpit/README.ja.md",
        ".ai/cockpit/adoption.md",
        ".ai/cockpit/adoption.ja.md",
        ".ai/cockpit/checks.yaml",
        "README.md",
        "README.zh-CN.md",
        "README.ja.md",
        "SECURITY.md",
        "CONTRIBUTING.md",
        "docs/README.md",
        "docs/README.zh-CN.md",
        "docs/README.ja.md",
        "docs/architecture.md",
        "docs/architecture.zh-CN.md",
        "docs/architecture.ja.md",
        "docs/capabilities.md",
        "docs/capabilities.zh-CN.md",
        "docs/capabilities.ja.md",
        "docs/purpose.md",
        "docs/purpose.zh-CN.md",
        "docs/purpose.ja.md",
        "docs/design-philosophy.md",
        "docs/trust-layer.md",
        "docs/trust-layer.zh-CN.md",
        "docs/trust-layer.ja.md",
        "docs/documentation-architecture.md",
        "docs/configuration.md",
        "docs/configuration.ja.md",
        "docs/contract-fields.md",
        "docs/features/task-outcome-report.md",
        "docs/features/task-outcome-report.ja.md",
        "docs/features/task-outcome-report.zh-CN.md",
        "docs/operations/work-item-lifecycle.md",
        "docs/operations/work-item-lifecycle.ja.md",
        "docs/operations/work-item-lifecycle.zh-CN.md",
        "docs/operations/recovery.md",
        "docs/operations/recovery.ja.md",
        "docs/operations/recovery.zh-CN.md",
        "docs/reference/commands.md",
        "docs/reference/configuration.md",
        "docs/reference/contract-fields.md",
        "docs/reference/repository-workflow.md",
        "docs/reference/agent-parallel-work-items.md",
        "docs/reference/ai-cockpit-work-item-lifecycle.md",
        "docs/reference/outcome-report.md",
    }
    return path in exact or path.startswith(
        (".ai/guards/", ".ai/policies/", ".ai/quality/", ".ai/schemas/", ".ai/trust/schema/")
    )


def is_getting_started_path(path: str) -> bool:
    return path.startswith("docs/getting-started/")


def counterpart_for(path: str, target_paths: set[str]) -> tuple[list[str], str, str]:
    direct = [path] if path in target_paths else []
    semantic_counterparts = {
        ".ai/cockpit/README.md": [".ai/README.md", "docs/reference/agent-workflow.md"],
        ".ai/cockpit/README.ja.md": [".ai/README.md", "docs/reference/agent-workflow.ja.md"],
        ".ai/cockpit/adoption.md": ["docs/getting-started/README.md", "docs/getting-started/adopter-configuration.md"],
        ".ai/cockpit/adoption.ja.md": ["docs/getting-started/README.ja.md", "docs/getting-started/adopter-configuration.ja.md"],
        "docs/configuration.md": ["docs/reference/configuration.md"],
        "docs/configuration.ja.md": ["docs/reference/configuration.ja.md"],
        "docs/contract-fields.md": ["docs/reference/contract-fields.md"],
        "docs/design-philosophy.md": ["docs/philosophy.md"],
        "docs/documentation-architecture.md": ["docs/reference/README.md", "docs/reference/reference-parity.md"],
        "docs/operations/recovery.md": ["docs/reference/troubleshooting.md", "docs/security/adversarial-validation.md"],
        "docs/operations/recovery.ja.md": ["docs/reference/troubleshooting.ja.md", "docs/security/adversarial-validation.ja.md"],
        "docs/operations/recovery.zh-CN.md": ["docs/reference/troubleshooting.zh-CN.md", "docs/security/adversarial-validation.zh-CN.md"],
        "docs/operations/work-item-lifecycle.md": ["docs/reference/agent-workflow.md", "docs/reference/outcome-report.md"],
        "docs/operations/work-item-lifecycle.ja.md": ["docs/reference/agent-workflow.ja.md", "docs/reference/outcome-report.ja.md"],
        "docs/operations/work-item-lifecycle.zh-CN.md": ["docs/reference/agent-workflow.zh-CN.md", "docs/reference/outcome-report.zh-CN.md"],
        "docs/purpose.md": ["docs/philosophy.md", "docs/capabilities.md"],
        "docs/purpose.ja.md": ["docs/philosophy.ja.md", "docs/capabilities.ja.md"],
        "docs/purpose.zh-CN.md": ["docs/philosophy.zh-CN.md", "docs/capabilities.zh-CN.md"],
        "docs/reference/agent-parallel-work-items.md": ["docs/reference/cross-work-item-dedup.md", "docs/reference/affected-verification.md"],
        "docs/reference/ai-cockpit-work-item-lifecycle.md": ["docs/reference/agent-workflow.md", "docs/reference/outcome-report.md"],
        "docs/reference/repository-workflow.md": ["docs/reference/agent-workflow.md"],
        "docs/trust-layer.md": ["docs/philosophy.md", "docs/security/enterprise-governance.md"],
        "docs/trust-layer.ja.md": ["docs/philosophy.ja.md", "docs/security/enterprise-governance.ja.md"],
        "docs/trust-layer.zh-CN.md": ["docs/philosophy.zh-CN.md", "docs/security/enterprise-governance.zh-CN.md"],
    }
    if path in semantic_counterparts:
        return semantic_counterparts[path], "implemented-different-by-design", "The target preserves the reference reader intent through Rust-native route pages with different ownership and paths."
    if path == "AGENTS.md":
        return direct, "implemented-different-by-design", "The attached repository adapter and installed shared Runtime replace template-local copy rules."
    if path in {"CLAUDE.md", "GEMINI.md", ".cursor/rules/ai-cockpit.mdc"}:
        return [".ai/agent-interface.json", "crates/cockpit-agent/src/lib.rs"], "implemented-different-by-design", "Provider surfaces are explicit, repository-local adapter installs; absence from this repository is not a global configuration mutation."
    if path.startswith(".ai/"):
        return direct + ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-protocol/src/lib.rs"], "implemented-different-by-design", "Reference repository-local YAML/Make governance is represented by the Rust Protocol, typed Runtime services, and repository tests."
    if path in {"CONTRIBUTING.md"}:
        if path in target_paths:
            return [path], "implemented-different-by-design", "The target now publishes a Rust/Runtime-specific contributor boundary derived from the reference entrypoint."
        return direct, "migrate-gap", "The reference publishes contributor boundaries; the target must add a Rust/Runtime-specific contribution entrypoint in this batch."
    if path in {"SECURITY.md"}:
        return direct, "implemented-equivalent", "The target retains the security boundary and adds Runtime-specific deployment and patch guidance."
    if path.startswith("README") or path.startswith("docs/"):
        if direct:
            return direct, "implemented-different-by-design", "The target keeps the reader route while documenting the shared Rust Runtime and explicit repository binding."
        return [], "migrate-gap", "The reference entrypoint has no target counterpart at this path and needs an explicit later decision."
    return direct, "deferred-next-batch", "Scheduled for a later semantic comparison batch; no equivalence or omission is claimed yet."


def generate(reference: Path, target: Path, source_commit: str, target_commit: str) -> dict[str, Any]:
    reference_paths = git_paths(reference)
    target_commit_paths = git_paths(target)
    target_paths = sorted(set(target_commit_paths) | set(git_working_paths(target)))
    target_set = set(target_paths)
    records: list[dict[str, Any]] = []
    for path in reference_paths:
        if path in CAPABILITY_STATUS_RECORDS:
            classification, counterparts, reason = CAPABILITY_STATUS_RECORDS[path]
            records.append(
                {
                    "referencePath": path,
                    "batch": CAPABILITY_STATUS_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        if is_generated_history(path):
            records.append(
                {
                    "referencePath": path,
                    "batch": "history-boundary",
                    "classification": "generated-history",
                    "rustCounterparts": [],
                    "reason": "Immutable reference history or generated projection is not copied into the Rust Runtime repository.",
                }
            )
            continue
        if is_governance_entrypoint(path):
            counterparts, classification, reason = counterpart_for(path, target_set)
            records.append(
                {
                    "referencePath": path,
                    "batch": FIRST_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        if is_getting_started_path(path):
            counterparts, classification, reason = counterpart_for(path, target_set)
            records.append(
                {
                    "referencePath": path,
                    "batch": GETTING_STARTED_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        records.append(
            {
                "referencePath": path,
                "batch": "later-batch",
                "classification": "deferred-next-batch",
                "rustCounterparts": [],
                "reason": "Scheduled for a later file-by-file semantic comparison batch; no equivalence or omission is claimed yet.",
            }
        )
    return {
        "schemaVersion": 1,
        "referenceRepository": "https://github.com/spirex-ds-dev/ai-cockpit-template",
        "referenceCommit": source_commit,
        "targetRepository": "https://github.com/xinglun/ai-cockpit",
        "targetCommit": target_commit,
        "referenceTrackedFileCount": len(reference_paths),
        "targetTrackedFileCount": len(target_commit_paths),
        "targetTrackedPathDigest": digest_paths(target_commit_paths),
        "targetWorkingTreeFileCount": len(target_paths),
        "targetWorkingTreePathDigest": digest_paths(target_paths),
        "allowedClassifications": sorted(ALLOWED_CLASSIFICATIONS),
        "records": records,
    }


def validate(manifest: dict[str, Any], expected_source: str, expected_target: str) -> list[str]:
    errors: list[str] = []
    if manifest.get("schemaVersion") != 1:
        errors.append("schemaVersion must be 1")
    if manifest.get("referenceCommit") != expected_source:
        errors.append("referenceCommit is not the pinned source commit")
    if manifest.get("targetCommit") != expected_target:
        errors.append("targetCommit is not the pinned target baseline")
    records = manifest.get("records")
    if not isinstance(records, list) or not records:
        return errors + ["records must be a non-empty list"]
    paths: set[str] = set()
    for index, record in enumerate(records):
        prefix = f"record[{index}]"
        path = record.get("referencePath") if isinstance(record, dict) else None
        if not isinstance(path, str) or not path:
            errors.append(f"{prefix} missing referencePath")
            continue
        if path in paths:
            errors.append(f"duplicate referencePath: {path}")
        paths.add(path)
        classification = record.get("classification")
        if classification not in ALLOWED_CLASSIFICATIONS:
            errors.append(f"{path}: invalid classification {classification!r}")
        if not isinstance(record.get("reason"), str) or not record["reason"].strip():
            errors.append(f"{path}: missing reason")
        if not isinstance(record.get("rustCounterparts"), list):
            errors.append(f"{path}: rustCounterparts must be a list")
        if record.get("batch") == FIRST_BATCH:
            if classification == "deferred-next-batch":
                errors.append(f"{path}: first-batch file cannot be deferred")
            if not record.get("rustCounterparts") and classification not in {
                "reference-only",
                "not-applicable",
                "migrate-gap",
            }:
                errors.append(f"{path}: first-batch record needs a counterpart or explicit boundary classification")
        if record.get("batch") == GETTING_STARTED_BATCH:
            if classification == "deferred-next-batch":
                errors.append(f"{path}: getting-started file cannot remain deferred")
            if not record.get("rustCounterparts") and classification not in {
                "reference-only",
                "not-applicable",
                "migrate-gap",
            }:
                errors.append(f"{path}: getting-started record needs a counterpart or explicit gap")
    scoped = {
        record.get("referencePath"): record
        for record in records
        if isinstance(record, dict)
        and record.get("referencePath") in CAPABILITY_STATUS_RECORDS
    }
    for path in CAPABILITY_STATUS_RECORDS:
        record = scoped.get(path)
        if record is None:
            errors.append(f"{path}: capability/status comparison record is missing")
            continue
        if record.get("classification") in {None, "", "deferred-next-batch"}:
            errors.append(f"{path}: capability/status classification must be non-deferred")
        if not record.get("rustCounterparts") and "no exact Rust counterpart" not in record.get("reason", ""):
            errors.append(f"{path}: capability/status result needs counterparts or an explicit no-counterpart reason")
    expected_count = manifest.get("referenceTrackedFileCount")
    if expected_count != len(records):
        errors.append(f"referenceTrackedFileCount {expected_count!r} != record count {len(records)}")
    return errors


def apply_getting_started_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        if not isinstance(path, str) or not is_getting_started_path(path):
            continue
        record.update(
            {
                "batch": GETTING_STARTED_BATCH,
                "classification": "implemented-different-by-design",
                "rustCounterparts": [path],
                "reason": "The target provides a tri-language shared-Runtime onboarding counterpart with explicit repository binding and without reference-local installer or Make workflows.",
            }
        )
        updated += 1
    if updated != 35:
        raise ValueError(f"expected 35 getting-started records, found {updated}")
    return updated


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference", type=Path)
    parser.add_argument("--target", type=Path)
    parser.add_argument("--manifest", type=Path, default=Path("tests/conformance/reference_file_inventory.json"))
    parser.add_argument("--output", type=Path)
    parser.add_argument("--source-commit", default=EXPECTED_REFERENCE_COMMIT)
    parser.add_argument("--target-commit", default=EXPECTED_TARGET_COMMIT)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--apply-getting-started-batch", action="store_true")
    args = parser.parse_args()

    if args.reference and args.target:
        manifest = generate(args.reference, args.target, args.source_commit, args.target_commit)
        output = args.output or args.manifest
        output.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    else:
        manifest = json.loads(args.manifest.read_text())
    if args.apply_getting_started_batch:
        try:
            apply_getting_started_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    errors = validate(manifest, args.source_commit, args.target_commit)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    counts: dict[str, int] = {}
    for record in manifest["records"]:
        key = record["classification"]
        counts[key] = counts.get(key, 0) + 1
    print(json.dumps({"ok": True, "records": len(manifest["records"]), "classifications": counts}, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
