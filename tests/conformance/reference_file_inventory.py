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
EXPECTED_TARGET_COMMIT = "487f01970c49e2b85d17b0cb0536f9d60c8f05e0"
CAPABILITY_STATUS_BATCH = "capability-status-projection"
WI270_BATCH = "WI-270-reference-contract-batch"
WI270_DOC_CONCEPTS = {
    "docs/concepts/decision-states.ja.md": ("ja",),
    "docs/concepts/decision-states.md": ("en",),
    "docs/concepts/decision-states.zh-CN.md": ("zh-CN",),
}
WI270_PARALLEL_DOCS = {
    "docs/features/work-item-parallelism.ja.md": ("ja",),
    "docs/features/work-item-parallelism.md": ("en",),
    "docs/features/work-item-parallelism.zh-CN.md": ("zh-CN",),
}
WI270_REFERENCE_FILES: dict[str, tuple[list[str], str]] = {
    "docs/reference/safe-parallel-verification.md": (
        ["crates/cockpit-verification/src/lib.rs", "crates/cockpit-cli/src/main.rs", "crates/cockpit-cli/tests/verify.rs"],
        "The Rust bounded executor accepts structured command argv, caps workers, serializes unsafe scopes, and records per-command evidence; no Python runner is copied.",
    ),
    "docs/reference/work-item-intelligence-interface.md": (
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md", "docs/reference/verification-cost.md"],
        "The Runtime exposes request-scoped status and intelligence, but the reference's full cost/wait/index-version aggregation remains a later projection boundary; no silent parity is claimed.",
    ),
    "docs/reference/work-item-state-machine.md": (
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-repository/tests/resource_finalization_transition.rs"],
        "Lifecycle transitions, recovery, finalization, archive, and close are Runtime-native; provider PR states remain an explicit external boundary.",
    ),
    "docs/reference/work-item-status-interface.md": (
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "crates/cockpit-repository/tests/status_projection.rs"],
        "The Runtime publishes evidence-derived request-scoped status and human Outcome projections rather than the reference generated Python status file.",
    ),
    "scripts/ai_acceptance_policy.py": (["crates/cockpit-repository/src/governance_controls.rs", "crates/cockpit-repository/tests/contract_preflight.rs"], "Typed Rust governance controls validate stable acceptance identifiers, evidence mappings, and fail-closed readiness."),
    "scripts/ai_check_scenario_coverage.py": (["crates/cockpit-repository/src/governance_controls.rs", "crates/cockpit-repository/tests/contract_preflight.rs"], "Typed Runtime scenario coverage validation replaces the reference Python gate and binds Contract/Summary evidence."),
    "scripts/ai_check_work_item.py": (["crates/cockpit-repository/src/lib.rs", "crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/tests/contract_schema.rs"], "Runtime Contract validation owns scope, authority, unknowns, execution decisions, concurrency, and lifecycle invariants."),
    "scripts/ai_decision_protocol.py": (["crates/cockpit-repository/src/governance_controls.rs", "crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/tests/preflight_review.rs"], "Repository-bound typed decision receipts replace the reference Python decision protocol and never infer human approval."),
    "scripts/ai_intent_policy.py": (["crates/cockpit-repository/src/governance_controls.rs", "crates/cockpit-verification/src/lib.rs", "crates/cockpit-verification/tests/intent_scenario_binding.rs"], "Intent alignment and scenario binding are Runtime validation inputs with explicit unknowns and policy traceability."),
    "scripts/ai_parallel_verification.py": (["crates/cockpit-verification/src/lib.rs", "crates/cockpit-cli/src/main.rs", "crates/cockpit-cli/tests/verify.rs"], "Rust bounded execution provides argv-only parallel verification, worker caps, deterministic results, and scope-safe execution."),
    "scripts/ai_preflight_review.py": (["crates/cockpit-core/src/lib.rs", "crates/cockpit-repository/src/governance_controls.rs", "crates/cockpit-repository/tests/preflight_review.rs"], "Typed preflight derives yellow/red/green state, humanDecisionRequest, identity-bound confirmation, and recovery conditions."),
    "scripts/ai_scenario_policy.py": (["crates/cockpit-repository/src/governance_controls.rs", "crates/cockpit-core/src/lib.rs"], "Risk-sensitive scenario coverage is evaluated by the Runtime policy layer and remains fail-closed for unknown required scenarios."),
    "scripts/ai_work_item_state.py": (["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-repository/tests/resource_finalization_transition.rs"], "The Rust lifecycle state machine and recovery receipts own transitions without copying the Python module."),
    "tests/test_acceptance_policy.py": (["crates/cockpit-repository/tests/contract_preflight.rs", "crates/cockpit-repository/tests/contract_schema.rs"], "Rust regression coverage exercises acceptance evidence identity, completeness, and fail-closed validation."),
    "tests/test_ai_parallel_verification.py": (["crates/cockpit-cli/tests/verify.rs", "crates/cockpit-verification/tests/execution.rs"], "Rust tests cover bounded workers, command results, failure retention, and snapshot binding."),
    "tests/test_checkpoint_intent.py": (["crates/cockpit-repository/tests/contract_preflight.rs", "crates/cockpit-cli/tests/preflight.rs"], "Rust preflight/checkpoint tests require human-owned intent and preserve unknowns instead of inferring them."),
    "tests/test_contract_and_policy.py": (["crates/cockpit-repository/tests/contract_schema.rs", "crates/cockpit-repository/tests/contract_preflight.rs"], "Typed Contract schema and policy tests cover strict fields, scope, authority, acceptance, and generated ownership."),
    "tests/test_intent_policy.py": (["crates/cockpit-verification/tests/intent_scenario_binding.rs", "crates/cockpit-repository/tests/contract_preflight.rs"], "Rust intent alignment tests distinguish unresolved, unknown, and resolved evidence."),
    "tests/test_parallel_lifecycle_contract.py": (["crates/cockpit-repository/tests/parallel_boundary.rs", "crates/cockpit-cli/tests/parallel_boundary.rs"], "Rust lifecycle/parallel tests cover isolated scopes, serialized projections, leases, overlap, and repository isolation."),
    "tests/test_preflight_review.py": (["crates/cockpit-repository/tests/preflight_review.rs", "crates/cockpit-cli/tests/preflight.rs"], "Rust preflight tests cover required human review, scenario coverage, policy signals, and identity-bound confirmation."),
    "tests/test_scenario_coverage_gate.py": (["crates/cockpit-repository/tests/contract_preflight.rs", "crates/cockpit-repository/tests/preflight_review.rs"], "Rust tests cover required verified/unverified/not-applicable scenarios, risk acknowledgement, and invalid status fail-closed behavior."),
}


def wi270_counterpart(path: str) -> tuple[list[str], str] | None:
    if path in WI270_DOC_CONCEPTS:
        language = WI270_DOC_CONCEPTS[path][0]
        suffix = "" if language == "en" else f".{language}"
        return [
            f"docs/reference/contract-fields{suffix}.md",
            f"docs/reference/outcome-report{suffix}.md",
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-repository/tests/outcome_report.rs",
        ], "The Runtime preserves the reference red/yellow/green stop semantics through typed decision and Outcome projections; the reference current_status.md file is not copied."
    if path in WI270_PARALLEL_DOCS:
        language = WI270_PARALLEL_DOCS[path][0]
        suffix = "" if language == "en" else f".{language}"
        return [
            f"docs/work-items/WI-123-parallel-contract-boundary{suffix}.md",
            f"docs/reference/configuration{suffix}.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/tests/parallel_boundary.rs",
        ], "Repository-local Contract boundaries and slot leases replace reference orchestration files; scope overlap is conservative and generated projections remain isolated."
    return WI270_REFERENCE_FILES.get(path)
CAPABILITY_STATUS_RECORDS: dict[str, tuple[str, list[str], str]] = {
    ".ai/project/adopter-capability-manifest.json": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            "crates/cockpit-mcp/src/lib.rs",
            "crates/cockpit-repository/tests/project_governance.rs",
            "docs/capabilities.md",
        ],
        "The Runtime-native capability registry and release/adopter acceptance evidence provide the governance projection; the reference templateFiles, installedFiles, schemas, and verifyInstalledSurface installer manifest remain an explicit external Release boundary and are not copied into the repository.",
    ),
    ".ai/project/capabilities.json": (
        "implemented-different-by-design",
        [
            ".ai/project/capabilities.json",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/src/project_governance.rs",
            "crates/cockpit-repository/tests/project_governance.rs",
        ],
        "A strict Rust-native repository declaration now binds capabilities, non-capabilities, critical domains, and explicit operation mappings to repository identity and snapshot; it never infers adopter acceptance.",
    ),
    ".ai/project/success_criteria.json": (
        "implemented-different-by-design",
        [
            ".ai/project/success_criteria.json",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/src/project_governance.rs",
            "crates/cockpit-repository/tests/project_governance.rs",
            "docs/reference/commands.md",
        ],
        "Project success criteria are a strict, visible compatibility projection bound to the repository snapshot; Contract acceptance remains authoritative and the criteria cannot approve, complete, or replace a Work Item.",
    ),
    ".ai/project_profile.yaml": (
        "implemented-different-by-design",
        [
            ".ai/project.json",
            ".ai/project/profile-policy.json",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/src/project_governance.rs",
            "crates/cockpit-repository/tests/project_governance.rs",
        ],
        "The Rust-native JSON profile policy preserves approved boundaries, critical domains, review requirements, and explicit unknowns beside the strict identity/observed profile in .ai/project.json; no YAML parser or reference runtime is copied.",
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


def git_paths(repository: Path, revision: str) -> list[str]:
    if len(revision) != 40 or any(character not in "0123456789abcdef" for character in revision):
        raise ValueError(f"revision must be a full lowercase commit digest: {revision!r}")
    result = subprocess.run(
        ["git", "-C", str(repository), "ls-tree", "-r", "--name-only", revision, "--"],
        check=True,
        capture_output=True,
        text=True,
    )
    return [line for line in result.stdout.splitlines() if line]


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
    reference_paths = git_paths(reference, source_commit)
    target_commit_paths = git_paths(target, target_commit)
    # A comparison baseline is an immutable commit tree. Untracked or modified
    # files in the operator's checkout must not enter its path inventory.
    target_paths = target_commit_paths
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
        wi270 = wi270_counterpart(path)
        if wi270 is not None:
            counterparts, reason = wi270
            records.append(
                {
                    "referencePath": path,
                    "batch": WI270_BATCH,
                    "classification": "implemented-different-by-design",
                    "rustCounterparts": counterparts,
                    "reason": f"WI-270 file-level comparison: {reason}",
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
    if manifest.get("targetWorkingTreeFileCount") != manifest.get("targetTrackedFileCount"):
        errors.append("target working-tree count is not normalized to the pinned commit")
    if manifest.get("targetWorkingTreePathDigest") != manifest.get("targetTrackedPathDigest"):
        errors.append("target working-tree digest is not normalized to the pinned commit")
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
