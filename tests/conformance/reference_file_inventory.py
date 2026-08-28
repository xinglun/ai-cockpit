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
EXPECTED_TARGET_COMMIT = "bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b"
CAPABILITY_STATUS_BATCH = "capability-status-projection"
WI270_BATCH = "WI-270-reference-contract-batch"
WI287_BATCH = "WI-287-reference-checkpoint-conformance"
WI302_BATCH = "WI-302-reference-file-comparison-batch-01"
WI304_BATCH = "WI-304-reference-file-comparison-batch-02"
WI305_BATCH = "WI-305-reference-file-comparison-batch-03"
WI325_BATCH = "WI-325-reference-file-comparison-batch-05"
WI326_BATCH = "WI-326-reference-file-comparison-batch-06"
WI327_BATCH = "WI-327-reference-file-comparison-batch-07"
WI328_BATCH = "WI-328-reference-file-comparison-batch-08"
WI331_BATCH = "WI-331-reference-file-comparison-batch-09"
WI332_BATCH = "WI-332-reference-file-comparison-batch-10"
WI333_BATCH = "WI-333-reference-file-comparison-batch-11"
WI334_BATCH = "WI-334-reference-file-comparison-batch-12"
WI342_BATCH = "WI-342-reference-documentation-batch-13"
WI343_BATCH = "WI-343-reference-inventory-foundation-reconciliation"
WI344_BATCH = "WI-344-reference-documentation-batch-14"
WI346_BATCH = "WI-346-reference-governance-profiles-status"
WI347_BATCH = "WI-347-reference-knowledge-trust-lifecycle-assessment"
WI348_BATCH = "WI-348-reference-verification-operation-policy"
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

WI287_REFERENCE_FILES: dict[str, tuple[list[str], str]] = {
    "scripts/ai_checkpoint.py": (
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/src/governance_controls.rs",
        ],
        "Typed CheckpointPolicy/CheckpointEvidence and append-only amendment/resume validation provide the same fail-closed semantics without copying the reference Python implementation.",
    ),
    "tests/test_ai_checkpoint.py": (
        [
            "crates/cockpit-repository/tests/agent_risk_checkpoint.rs",
            "crates/cockpit-repository/tests/lifecycle_order.rs",
        ],
        "Rust lifecycle and tamper regressions cover checkpoint ordering, immutable before_edit evidence, amendment lineage, resume freshness, and strict identity bindings; wire formats are intentionally not copied.",
    ),
}

WI272_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_check_agent_risk.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/governance_controls.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/contract_preflight.rs",
            "crates/cockpit-repository/tests/preflight_review.rs",
        ],
        "The reference Python risk gate is mapped to typed Contract, preflight, checkpoint, scenario, and lifecycle validators in Rust; the Python module is intentionally not copied.",
    ),
    "templates/agents/AI_COCKPIT_RULES.md": (
        "implemented-different-by-design",
        [
            "AGENTS.md",
            ".ai/README.md",
            "crates/cockpit-agent/src/lib.rs",
            "docs/reference/agent-workflow.md",
            "docs/reference/agent-workflow.zh-CN.md",
            "docs/reference/agent-workflow.ja.md",
        ],
        "The reference Agent rules are projected into repository-local instructions and the generated Rust adapter; template prompt files and provider-global configuration are not copied.",
    ),
    "tests/test_ai_check_agent_risk.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-agent/tests/install.rs",
            "crates/cockpit-repository/tests/contract_preflight.rs",
            "crates/cockpit-repository/tests/preflight_review.rs",
            "crates/cockpit-repository/tests/lifecycle_order.rs",
            "crates/cockpit-cli/tests/lifecycle.rs",
        ],
        "Rust regression tests cover the typed Contract, human review, lifecycle ordering, and generated adapter boundaries represented by the reference Python test corpus.",
    ),
    "tests/test_outcome_lifecycle_rules.py": (
        "implemented-different-by-design",
        [
            "AGENTS.md",
            ".ai/README.md",
            "crates/cockpit-agent/tests/install.rs",
            "docs/reference/agent-workflow.md",
            "docs/reference/agent-workflow.zh-CN.md",
            "docs/reference/agent-workflow.ja.md",
        ],
        "Outcome terminality, direct human handoff, current-Work-Item repair, and narrow successor rules are enforced by Rust-native instructions and adapter regression tests rather than copied reference Python tests.",
    ),
}

WI293_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "tests/test_release_workflow.py": (
        "deferred-next-batch",
        ["tests/ci/quality_route_test.py", "tests/ci/repository_gate_manifest_test.py"],
        "WI-293 adds Rust/Python Contract gate convergence checks; complete release workflow ordering and provider evidence remain deferred.",
    ),
    "tests/test_workflows.py": (
        "deferred-next-batch",
        ["tests/ci/quality_route_test.py", ".github/workflows/ci.yml"],
        "WI-293 adds the Contract gate order and shadow boundary; full workflow graph, timeout, concurrency, and action policy parity remain deferred.",
    ),
}

WI302_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    ".ai/cockpit/bandit_low_risk_baseline.json": (
        "not-applicable",
        [],
        "The source file is a generated Bandit baseline for the reference repository's Python tooling. The Rust Runtime has no Python/Bandit surface; this is not a product or repository Protocol omission.",
    ),
    ".gitattributes": (
        "implemented-different-by-design",
        [".gitattributes", "tests/release/source_archive_policy_test.sh"],
        "The source excludes selected mutable governance projections from its Python archive. Rust uses a stricter, tested source-archive boundary that excludes .ai, .worktrees, dist, and target while retaining Cargo sources and lockfile.",
    ),
    ".github/CODEOWNERS": (
        "not-applicable",
        ["CONTRIBUTING.md", "docs/getting-started/adopter-configuration.md", "docs/security/enterprise-deployment-boundary.md"],
        "The source entry assigns a personal GitHub owner. A universal owner cannot be inferred for adopters and is not Runtime authority; review ownership remains an external repository/provider decision documented as a boundary.",
    ),
    ".github/dependabot.yml": (
        "not-applicable",
        ["Cargo.toml", "Cargo.lock", ".github/workflows/ci.yml", "docs/security/enterprise-deployment-boundary.md"],
        "The source configuration is optional provider automation for pip and GitHub Actions updates. The Rust target has Cargo.lock and pinned-action policy, while dependency-update service selection remains external and repository-owned rather than a Runtime capability.",
    ),
    ".github/workflows/release.yml": (
        "implemented-different-by-design",
        [".github/workflows/release.yml", "tests/release/workflow_policy.sh", "tests/release/version_consistency.sh", "tests/release/adopter_acceptance.sh", "tests/release/adopter_upgrade_acceptance.sh"],
        "The Rust release workflow preserves the source publication responsibility through target-specific archives, checksums, SBOM/provenance, Homebrew/Linux/Windows smoke, and public/N-1 adopter acceptance. Cargo and Rust Runtime gates replace Python/Make steps; byte-level workflow parity is not claimed.",
    ),
    ".gitignore": (
        "implemented-different-by-design",
        [".gitignore", "tests/release/source_archive_policy_test.sh"],
        "The target ignore policy covers Cargo/Rust build outputs, cross-platform tooling, and local governance review files. Source Python bytecode and Make-era paths are retained only where applicable; archive policy is regression-tested.",
    ),
    "LICENSE": (
        "implemented-different-by-design",
        ["LICENSE", ".github/workflows/release.yml", "tests/release/source_archive_policy_test.sh"],
        "Both projects publish an MIT license boundary. The copyright holder is target-specific and the Rust release package includes the target LICENSE; source copyright text is not copied as a governance decision.",
    ),
    "Makefile": (
        "implemented-different-by-design",
        [".github/workflows/ci.yml", "tests/ci/run_repository_gates.py", "docs/reference/commands.md", "Cargo.toml"],
        "The source Makefile is a Python orchestration surface. The target deliberately uses the Rust CLI, Cargo, and explicit CI/release scripts with repository-bound --repo context; no second Make governance layer is required.",
    ),
}

WI304_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    ".github/workflows/compatibility.yml": (
        "implemented-different-by-design",
        [
            ".github/workflows/ci.yml",
            "tests/ci/quality_route.py",
            "tests/ci/run_repository_gates.py",
            "tests/release/adopter_acceptance.sh",
            "docs/capabilities.md",
            "docs/release/distribution.md",
        ],
        "WI-304 compares every source job and boundary: shellcheck/install.sh, pinned Python and lockfile lanes, real/extended/mobile stack matrices, and non-blocking latest probes. The Rust target uses its dynamic light/standard/strict route, canonical manifest, Rust workspace and platform gates, and immutable Release adopter acceptance. It has no install.sh or source Make/Python matrix; object-repository toolchain coverage remains an explicit adopter/external responsibility rather than hidden parity.",
    ),
    ".github/workflows/smoke.yml": (
        "implemented-different-by-design",
        [
            ".github/workflows/ci.yml",
            ".github/workflows/release.yml",
            "tests/ci/repository_gate_manifest.json",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
            "docs/reference/reference-file-comparison.md",
            "docs/release/distribution.md",
        ],
        "WI-304 compares all source smoke jobs, dispatch inputs, needs edges, artifacts, release/measurement conditions, and installer checks. The Rust target deliberately splits those responsibilities across ci.yml, release.yml, the canonical gate manifest, and immutable public/N-1 adopter acceptance. Source Python project-test shards, install.sh/Make smoke, and source-specific latest-toolchain probes have no target equivalent and remain documented external/adopter boundaries; no byte-level workflow parity is claimed.",
    ),
}

WI305_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/architecture/installation-detection-boundary.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/installation.md",
            "docs/getting-started/first-calibration.md",
            "docs/getting-started/adopter-configuration.md",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/tests/attach.rs",
            "crates/cockpit-cli/tests/profile_propose.rs",
        ],
        "The target exposes the same read-only-first facts and explicit write boundary through inspect, status, doctor, attach, profile propose, and calibration. It intentionally has no source-local Installer or implicit installation plan: the shared Runtime is installed from an immutable Release, while repository attachment and profile decisions remain explicit.",
    ),
    "docs/architecture/interactive-installation-wizard.md": (
        "reference-only",
        [
            "docs/getting-started/installation.md",
            "docs/getting-started/adopter-configuration.md",
            "docs/architecture/product-boundary.md",
        ],
        "The source ten-stage interactive Installer Wizard is retained as reference architecture only. Rust deliberately does not ship a second interactive installer: public Release installation, explicit inspect/attach/profile commands, preflight human review, and provider-owned installation boundaries are the supported adopter flow.",
    ),
    "docs/architecture/lightweight-verification-and-soft-gates.md": (
        "implemented-different-by-design",
        [
            "docs/reference/verification-route.md",
            "docs/reference/verification-semantics.md",
            "docs/reference/ci-quality-gates.md",
            "docs/reference/verification-cost.md",
            "crates/cockpit-verification/src/lib.rs",
            "crates/cockpit-repository/tests/verification_route.rs",
            "crates/cockpit-verification/tests/cost_observation.rs",
        ],
        "Rust preserves stage-aware verification, fail-closed governance decisions, explicit skipped or unknown boundaries, one request-scoped context, dynamic light/standard/strict CI routing, and advisory cost/reuse telemetry through typed Runtime services. The source hard/soft/informational checker labels are represented as a documented boundary rather than copied as a generic wire enum; the source Make command and Python checker registry are not copied.",
    ),
    "docs/architecture/wizard-io-and-localization.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/installation.md",
            "docs/reference/outcome-report.md",
            "docs/reference/commands.md",
            "crates/cockpit-cli/src/main.rs",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
            "crates/cockpit-mcp/tests/rpc.rs",
        ],
        "The target localizes Runtime-generated CLI/MCP Outcome and command presentation in en/zh-CN/ja, preserves contract/source values verbatim, and fails closed at explicit command and preflight boundaries. Source Wizard-specific TTY back/help/pause input is not a Runtime feature because the target has no interactive Installer Wizard; adapters remain responsible for conversation UX.",
    ),
}

WI306_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/assets/ai-cockpit-demo.gif": (
        "reference-only",
        [],
        "The reference GIF is a visual demonstration asset, not Runtime code or a repository-governance contract. The Rust project records its pinned type, dimensions, size, and digest in the comparison Work Item but does not copy binary media into the Runtime repository.",
    ),
    "docs/case-study-ai-rollback-corruption.md": (
        "implemented-different-by-design",
        [
            "docs/security/adversarial-validation.md",
            "docs/security/adversarial-validation.zh-CN.md",
            "docs/security/adversarial-validation.ja.md",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/contract_preflight.rs",
        ],
        "The hypothetical rollback-corruption scenario is projected into the tri-language adversarial-validation route and typed scope/Contract checks. Rust stops on unauthorized paths and preserves evidence; it does not claim automatic semantic rollback or detection of every business-impacting regression.",
    ),
    "docs/concepts/evidence-governance.md": (
        "implemented-different-by-design",
        [
            "docs/security/enterprise-governance.md",
            "docs/security/enterprise-governance.zh-CN.md",
            "docs/security/enterprise-governance.ja.md",
            "docs/reference/outcome-report.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The Evidence → Governance Decision → Human Control chain is documented by the enterprise-governance and human-facing Outcome routes and implemented by typed repository-bound evidence/lifecycle services. Provider proof remains delegated and is never inferred from Agent prose.",
    ),
    "docs/concepts/trust-layer.md": (
        "implemented-different-by-design",
        [
            "docs/architecture/product-boundary.md",
            "docs/philosophy.md",
            "docs/security/enterprise-governance.md",
            "docs/reference/capability-truth-matrix.md",
        ],
        "The reference calibrated-trust explanation is preserved across the Rust product-boundary, design-philosophy, enterprise-governance, and capability-truth routes. The target explicitly remains a Repository Governance Layer, not an Agent Runtime, sandbox, identity provider, or compliance certificate.",
    ),
}

WI323_BATCH = "WI-323-reference-documentation-foundation"
WI323_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/contributing/installation-document-maintenance.md": (
        "implemented-different-by-design",
        [
            "docs/reference/README.md",
            "docs/reference/README.zh-CN.md",
            "docs/reference/README.ja.md",
            "tests/docs/documentation_acceptance.sh",
            "tests/docs/getting_started_semantic.sh",
        ],
        "The target keeps the source maintenance responsibilities as a tri-language, thin reader route with link/metadata acceptance checks and dedicated release/security/recovery pages. Rust uses the shared Runtime and explicit --repo commands; source Make metadata commands are not copied.",
    ),
    "docs/current/README.md": (
        "implemented-different-by-design",
        [
            "docs/current/README.md",
            "docs/current/README.zh-CN.md",
            "docs/current/README.ja.md",
            ".ai/README.md",
            ".ai/glossary.md",
            "AGENTS.md",
            "docs/reference/README.md",
        ],
        "The target provides the same canonical current-agent route through the repository-owned .ai read set, AGENTS.md, and tri-language current/reference pages. The source make ai-documentation-read-set command and source Python authority files are not target commands.",
    ),
    "docs/design/harden-work-item-pr-closure.md": (
        "implemented-different-by-design",
        [
            "docs/reference/agent-workflow.md",
            "docs/reference/agent-workflow.zh-CN.md",
            "docs/reference/agent-workflow.ja.md",
            "docs/reference/commands.md",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/tests/resource_finalization.rs",
        ],
        "The target enforces the same latest-base, dedicated-branch, reviewed-PR, merge-before-close, synchronization, and exact-cleanup boundary through the Rust lifecycle and repository workflow. Provider PR creation and merging remain external; source make targets are not copied.",
    ),
    "docs/distribution.md": (
        "implemented-different-by-design",
        [
            "docs/release/distribution.md",
            "docs/release/distribution.zh-CN.md",
            "docs/release/distribution.ja.md",
            "docs/current/README.md",
            "docs/current/README.zh-CN.md",
            "docs/current/README.ja.md",
        ],
        "The source compatibility entry is represented by the target's adopter-first current route and detailed Rust Release distribution/installation/adopter-acceptance pages. The public artifact and installer contract is target-specific and is not a byte-level source copy.",
    ),
    "docs/enterprise-security-boundary.md": (
        "implemented-different-by-design",
        [
            "docs/security/enterprise-deployment-boundary.md",
            "docs/security/enterprise-deployment-boundary.zh-CN.md",
            "docs/security/enterprise-deployment-boundary.ja.md",
            "docs/security/enterprise-governance.md",
            "docs/security/enterprise-governance.zh-CN.md",
            "docs/security/enterprise-governance.ja.md",
            "SECURITY.md",
        ],
        "The target preserves the source separation between repository evidence and external enterprise controls, with additional authority, delegated evidence, retention, audit, deployment, and non-certification boundaries. It does not claim sandbox, identity-provider, or compliance certification capabilities.",
    ),
    "docs/examples/trust-layer-demo.sh": (
        "reference-only",
        [],
        "The offline shell demonstration is retained as explanatory reference material only. Its stop/continue examples are represented by typed Runtime preflight, intent, capability, and adversarial tests, but the source demo script is not copied or executed as Runtime authority.",
    ),
    "docs/features/human-benefit-report.md": (
        "implemented-different-by-design",
        [
            "docs/features/human-benefit-report.md",
            "docs/reference/outcome-report.md",
            "docs/reference/task-outcome-events.md",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
            "crates/cockpit-mcp/tests/rpc.rs",
        ],
        "The target preserves the person-facing report order, evidence-count semantics, stale/malformed stop boundary, and no-unsupported-benefit rule through Rust OutcomeV2, CLI human handoff, and MCP projection. It uses work-item outcome and work_item_outcome rather than the source make/Python report generator.",
    ),
    "docs/features/human-benefit-report.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/features/human-benefit-report.zh-CN.md",
            "docs/reference/outcome-report.zh-CN.md",
            "docs/reference/task-outcome-events.zh-CN.md",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
            "crates/cockpit-mcp/tests/rpc.rs",
        ],
        "Rust 版保留中文面向人的报告顺序、evidence 计数语义、过期/损坏即停止和不臆造收益的边界，使用 work-item outcome 与 MCP work_item_outcome；不复制参考源的 Make/Python 报告生成器。",
    ),
    "docs/features/human-benefit-report.ja.md": (
        "implemented-different-by-design",
        [
            "docs/features/human-benefit-report.ja.md",
            "docs/reference/outcome-report.ja.md",
            "docs/reference/task-outcome-events.ja.md",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
            "crates/cockpit-mcp/tests/rpc.rs",
        ],
        "Rust 版は人向け report の順序、evidence count の意味、stale/malformed の停止、根拠のない benefit を fact にしない境界を OutcomeV2/CLI/MCP で保ちます。source の Make/Python report generator はコピーしません。",
    ),
}

WI325_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/features/task-outcome-report-self-check.md": (
        "reference-only",
        [
            "docs/features/task-outcome-report.md",
            "docs/reference/outcome-report.md",
            "docs/reference/task-outcome-events.md",
            ".ai/README.md",
        ],
        "The source self-check is an internal, historical WI22 handoff with obsolete publication claims. Current Outcome, event, and agent-route boundaries are documented and Runtime-validated elsewhere; the internal progress narrative is not copied as current capability.",
    ),
    "docs/fixtures/real-fixture-evidence.ja.md": (
        "implemented-different-by-design",
        [
            "tests/fixtures/README.ja.md",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
            "docs/release/distribution.ja.md",
            "docs/security/adversarial-validation.ja.md",
        ],
        "The source fixture report's local multi-stack matrix is represented by Rust conformance fixtures and the immutable Release adopter/upgrade acceptance harness. Local evidence, provider evidence, and enterprise assurance remain separate; the source make/Python fixture lifecycle is not copied.",
    ),
    "docs/fixtures/real-fixture-evidence.md": (
        "implemented-different-by-design",
        [
            "tests/fixtures/README.md",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
            "docs/release/distribution.md",
            "docs/security/adversarial-validation.md",
        ],
        "The source fixture report's local multi-stack matrix is represented by Rust conformance fixtures and the immutable Release adopter/upgrade acceptance harness. Local evidence, provider evidence, and enterprise assurance remain separate; the source make/Python fixture lifecycle is not copied.",
    ),
    "docs/guides/lightweight-verification.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/verification-route.ja.md",
            "docs/reference/verification-semantics.ja.md",
            "docs/reference/ci-quality-gates.ja.md",
            "docs/reference/verification-cost.ja.md",
        ],
        "The source Task/PR/Release signal guidance is preserved through stage-aware Rust verification, dynamic light/standard/strict CI routing, and bounded cost observation. Warnings remain non-authorizing and critical failures stop; source checker scripts are not copied.",
    ),
    "docs/guides/lightweight-verification.md": (
        "implemented-different-by-design",
        [
            "docs/reference/verification-route.md",
            "docs/reference/verification-semantics.md",
            "docs/reference/ci-quality-gates.md",
            "docs/reference/verification-cost.md",
        ],
        "The source Task/PR/Release signal guidance is preserved through stage-aware Rust verification, dynamic light/standard/strict CI routing, and bounded cost observation. Warnings remain non-authorizing and critical failures stop; source checker scripts are not copied.",
    ),
    "docs/guides/lightweight-verification.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/verification-route.zh-CN.md",
            "docs/reference/verification-semantics.zh-CN.md",
            "docs/reference/ci-quality-gates.zh-CN.md",
            "docs/reference/verification-cost.zh-CN.md",
        ],
        "源文件关于 Task/PR/Release 信号的语义由 Rust 的阶段验证、动态 light/standard/strict CI 路由和有界成本观测保留。警告不能授权，关键失败会停止；不复制源 Python checker 脚本。",
    ),
    "docs/installation.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/installation.md",
            "docs/getting-started/installation-security.md",
            "docs/release/distribution.md",
            ".ai/README.md",
        ],
        "The source compatibility page is represented by the Rust reader-first installation and Release distribution route. A shared binary is installed independently, repository attachment and Agent discovery are explicit, and calibration/profile decisions are not implied by installation; source ten-stage wizard and Make commands are not copied.",
    ),
    "docs/maintainers/adding-or-classifying-a-check.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-quality-gates.md",
            "tests/ci/repository_gate_manifest.json",
            "tests/ci/quality_route.py",
            "tests/ci/run_repository_gates.py",
            "tests/ci/repository_gate_manifest_test.sh",
        ],
        "The source checker-registration guidance is represented by the versioned gate manifest, dynamic route, and gate-runner receipts. Required profiles, dependencies, skipped boundaries, and fail-closed results remain explicit; a source-specific checker registry and Python authority module are not copied.",
    ),
    "docs/maintainers/task-outcome-events.md": (
        "implemented-different-by-design",
        [
            "docs/reference/task-outcome-events.md",
            "docs/features/task-outcome-report.md",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/task_outcome_events.rs",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
        ],
        "The source append-only event, correction, deduplication, validation, privacy, and handoff boundaries are implemented by typed Rust Task Outcome events and archive binding. Generated projections never replace Contract authority and historical lines are not rewritten.",
    ),
}

WI326_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/non-make-adaptation.ja.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/installation.ja.md",
            "docs/reference/commands.ja.md",
            "docs/reference/agent-workflow.ja.md",
        ],
        "The source Make bridge guide is preserved as an explicit Rust Runtime installation and Agent workflow boundary. This repository does not copy the source Makefile.ai contract or require a second Make governance layer; adopter-owned stack commands remain outside the Core.",
    ),
    "docs/operations/quality-gates.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-quality-gates.ja.md",
            "docs/reference/quality-gate-manifest.ja.md",
            "tests/ci/repository_gate_manifest.json",
            "tests/ci/run_repository_gates.py",
        ],
        "The source quality-gate ownership, evidence, traceability, and light/standard/strict routing semantics are represented by the versioned Rust-native gate manifest and documented CI route. Source Make targets, Python checker registries, and template-maintenance fixtures are not copied.",
    ),
    "docs/operations/quality-gates.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-quality-gates.md",
            "docs/reference/quality-gate-manifest.md",
            "tests/ci/repository_gate_manifest.json",
            "tests/ci/run_repository_gates.py",
        ],
        "The source quality-gate ownership, evidence, traceability, and light/standard/strict routing semantics are represented by the versioned Rust-native gate manifest and documented CI route. Source Make targets, Python checker registries, and template-maintenance fixtures are not copied.",
    ),
    "docs/operations/quality-gates.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-quality-gates.zh-CN.md",
            "docs/reference/quality-gate-manifest.zh-CN.md",
            "tests/ci/repository_gate_manifest.json",
            "tests/ci/run_repository_gates.py",
        ],
        "源文件关于质量门所有权、证据、追踪和 light/standard/strict 动态路由的语义，由版本化 Rust 原生门禁清单和 CI 路由文档保留。不复制源 Make 目标、Python checker 注册表或模板维护 fixture。",
    ),
    "docs/overview.ja.md": (
        "implemented-different-by-design",
        [
            "docs/architecture.ja.md",
            "docs/capabilities.ja.md",
            "docs/reference/agent-workflow.ja.md",
            "docs/reference/commands.ja.md",
        ],
        "The source five-layer overview is preserved by the Rust product-boundary architecture, capabilities, Agent workflow, and command routes. Runtime governance is request-scoped and repository-bound; the source Python/Make status file and verification registry are not copied.",
    ),
    "docs/philosophy/design-philosophy.ja.md": (
        "implemented-different-by-design",
        [
            "docs/philosophy.ja.md",
            "docs/capabilities.ja.md",
            "docs/security/enterprise-governance.ja.md",
        ],
        "The source calibrated-trust, evidence-over-self-declaration, proportional-control, and human-responsibility principles are preserved across the Rust product-boundary and enterprise-governance routes. The target remains a Repository Governance Layer rather than an Agent Runtime, sandbox, identity provider, or compliance certificate.",
    ),
    "docs/philosophy/design-philosophy.md": (
        "implemented-different-by-design",
        [
            "docs/philosophy.md",
            "docs/capabilities.md",
            "docs/security/enterprise-governance.md",
        ],
        "The source calibrated-trust, evidence-over-self-declaration, proportional-control, and human-responsibility principles are preserved across the Rust product-boundary and enterprise-governance routes. The target remains a Repository Governance Layer rather than an Agent Runtime, sandbox, identity provider, or compliance certificate.",
    ),
    "docs/philosophy/design-philosophy.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/philosophy.zh-CN.md",
            "docs/capabilities.zh-CN.md",
            "docs/security/enterprise-governance.zh-CN.md",
        ],
        "源文件关于校准后信任、证据优先于自我声明、与风险相称的控制和人的责任原则，由 Rust 产品边界与企业治理文档保留。目标仍是 Repository Governance Layer，而不是 Agent Runtime、安全沙箱、身份提供方或合规证书。",
    ),
    "docs/plans/harden-work-item-pr-closure.md": (
        "reference-only",
        [
            "docs/reference/agent-workflow.md",
            "docs/reference/commands.md",
            "docs/reference/lifecycle-order.md",
            "tests/ci/governance_integrity_gate.py",
        ],
        "The source file is an internal historical hardening plan for the Python ai-finish/ai-close workflow. Its closure intent is represented by current Rust lifecycle and governance-integrity routes, but the plan's historical implementation steps and obsolete command names are not current Runtime capability.",
    ),
}

WI327_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/adopter-long-cycle-validation.ja.md": (
        "implemented-different-by-design",
        [
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
            "docs/release/distribution.ja.md",
            "docs/reference/commands.ja.md",
            "docs/security/adversarial-validation.ja.md",
        ],
        "The source multi-stack and independent-adopter long-cycle semantics are represented by the immutable published-binary adopter/upgrade acceptance harness and the Rust lifecycle/distribution boundaries. Source Python, Make, and fixture matrix execution is not copied; provider and enterprise claims remain separate evidence boundaries.",
    ),
    "docs/reference/adopter-long-cycle-validation.md": (
        "implemented-different-by-design",
        [
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
            "docs/release/distribution.md",
            "docs/reference/commands.md",
            "docs/security/adversarial-validation.md",
        ],
        "The source multi-stack and independent-adopter long-cycle semantics are represented by the immutable published-binary adopter/upgrade acceptance harness and the Rust lifecycle/distribution boundaries. Source Python, Make, and fixture matrix execution is not copied; provider and enterprise claims remain separate evidence boundaries.",
    ),
    "docs/reference/adoption-reality-report.md": (
        "implemented-different-by-design",
        [
            "docs/capabilities.md",
            "docs/release/distribution.md",
            "docs/security/enterprise-governance.md",
            "crates/cockpit-repository/src/project_governance.rs",
            "crates/cockpit-repository/tests/project_governance.rs",
            "tests/release/adopter_acceptance.sh",
        ],
        "The source conservative capability/adopter reality projection is represented by Runtime capability, profile, status, and published-adopter evidence boundaries. Template-owned bytes never prove adopter/provider configuration, external identity, SBOM, provenance, signing, or enterprise assurance; the source report generator is not copied.",
    ),
    "docs/reference/bandit-synchronization-security-audit.md": (
        "reference-only",
        [
            "docs/reference/ci-quality-gates.md",
            "docs/security/threat-model.md",
            "tests/ci/run_repository_gates.py",
        ],
        "The source document is a scanner-specific historical Bandit finding inventory and synchronization incident. The Rust target has no Python/Bandit surface and must not claim the source count or digest; its native quality and threat-model boundaries remain separate and do not require copying the source audit bytes.",
    ),
    "docs/reference/calibration-inventory.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.md",
            "docs/getting-started/first-calibration.md",
            "docs/reference/configuration.md",
            "docs/capabilities.md",
            "crates/cockpit-repository/src/project_governance.rs",
            "crates/cockpit-repository/tests/project_governance.rs",
        ],
        "The source calibration inventory's fact/evidence boundary is represented by repository-bound profile proposal and confirmation, capability/status projections, and explicit unknowns. The Rust Runtime does not copy the source ten-column Python inventory or turn static presence into adopter, identity, audit, sandbox, or enterprise proof.",
    ),
    "docs/reference/calibration-profiles.ja.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.ja.md",
            "docs/getting-started/first-calibration.ja.md",
            "docs/reference/configuration.ja.md",
            "docs/capabilities.ja.md",
            ".ai/project/profile-policy.json",
            "crates/cockpit-repository/src/project_governance.rs",
        ],
        "The source Lite/Standard/Strict calibration boundary is represented by the Rust repository profile policy and explicit proposal/confirmation flow. It remains distinct from per-Work-Item quality routing, preserves unknowns and human selection, and does not claim source YAML or external identity/compliance proof.",
    ),
    "docs/reference/calibration-profiles.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.md",
            "docs/getting-started/first-calibration.md",
            "docs/reference/configuration.md",
            "docs/capabilities.md",
            ".ai/project/profile-policy.json",
            "crates/cockpit-repository/src/project_governance.rs",
        ],
        "The source Lite/Standard/Strict calibration boundary is represented by the Rust repository profile policy and explicit proposal/confirmation flow. It remains distinct from per-Work-Item quality routing, preserves unknowns and human selection, and does not claim source YAML or external identity/compliance proof.",
    ),
    "docs/reference/calibration-profiles.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.zh-CN.md",
            "docs/getting-started/first-calibration.zh-CN.md",
            "docs/reference/configuration.zh-CN.md",
            "docs/capabilities.zh-CN.md",
            ".ai/project/profile-policy.json",
            "crates/cockpit-repository/src/project_governance.rs",
        ],
        "源文件关于 Lite/Standard/Strict 的校准边界，由 Rust repository profile policy 与显式 proposal/confirm 流程保留。它与单个 Work Item 的质量路由分离，保留 unknown 和人工选择，不宣称复制源 YAML，也不宣称外部身份或合规证明。",
    ),
    "docs/reference/calibration-session-model.ja.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.ja.md",
            "docs/getting-started/first-calibration.ja.md",
            "docs/reference/configuration.ja.md",
            "crates/cockpit-repository/src/project_governance.rs",
            "crates/cockpit-repository/tests/project_governance.rs",
        ],
        "The source internal resumable calibration Session model is represented only by the target's explicit read-only profile proposal, human confirmation, and repository-bound calibration facts. No generic interactive Session or checklist is silently introduced; unknowns and human authority remain visible.",
    ),
}

WI328_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/calibration-session-model.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.md",
            "docs/getting-started/first-calibration.md",
            "docs/reference/configuration.md",
            "crates/cockpit-repository/src/project_governance.rs",
        ],
        "The source maintainer/auditor Session model is represented by repository-bound profile proposal, human confirmation, and explicit calibration facts. The target does not introduce a generic persisted Session or treat a proposal as active policy; unknowns and human authority remain visible.",
    ),
    "docs/reference/calibration-session-model.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.zh-CN.md",
            "docs/getting-started/first-calibration.zh-CN.md",
            "docs/reference/configuration.zh-CN.md",
            "crates/cockpit-repository/src/project_governance.rs",
        ],
        "源文件关于可恢复 Session、证据、提议和人工确认的维护者/审计者语义，由 repository-bound profile proposal、confirm 和显式 calibration facts 保留。目标不引入通用持久化 Session，也不把 proposal 当作 active policy；unknown 和人工责任保持可见。",
    ),
    "docs/reference/calibration-session.ja.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.ja.md",
            "docs/getting-started/first-calibration.ja.md",
            "docs/reference/configuration.ja.md",
            "crates/cockpit-repository/src/project_governance.rs",
        ],
        "Source の 10 段階 interactive Session は、repository-bound な profile proposal、human confirmation、calibration facts として意味だけを保持します。Rust target は汎用 Session、source Make/Python、または enterprise/security proof を暗黙に追加しません。",
    ),
    "docs/reference/calibration-session.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/calibration.md",
            "docs/getting-started/first-calibration.md",
            "docs/reference/configuration.md",
            "crates/cockpit-repository/src/project_governance.rs",
        ],
        "The source ten-stage persisted Calibration Session and wizard are a source-specific orchestration surface. The target keeps a read-only profile proposal and explicit human confirmation over repository facts; it does not claim the source Session schema, Make/Python commands, or wizard activation transaction.",
    ),
    "docs/reference/canonical-terminology.md": (
        "implemented-different-by-design",
        [
            ".ai/glossary.md",
            "docs/reference/configuration.md",
            "docs/reference/outcome-report.md",
        ],
        "Canonical Runtime terms are maintained in the repository glossary and current reference pages. Governance `light`/`standard`/`strict` is deliberately distinct from any source Calibration `lite` domain; the target does not introduce a second profile vocabulary or treat `release` as a profile.",
    ),
    "docs/reference/capability-claim-authoring.md": (
        "reference-only",
        [
            "docs/capabilities.md",
            "docs/reference/reference-parity.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "WI-330 formally closes this source boundary: the lexical Capability Truth Matrix checker and claim-binding front matter remain reference-only, not a target Runtime gate. The target capability registry reports observed, repository-bound facts and explicit exclusions; it must not claim source matrix validation or silently promote prose to evidence. Any future bounded capability-claim/evidence gate requires a separately human-owned Work Item; no Python/V1 checker is copied.",
    ),
    "docs/reference/capability-evidence-freshness.md": (
        "reference-only",
        [
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/outcome-report.md",
            "docs/reference/verification-evidence-reuse.md",
        ],
        "WI-330 records that the target validates Work Item verification freshness and identity-bound receipts, but intentionally has no separate Capability Truth row expiry or portable-environment matrix. The source capability-row freshness policy remains reference-only; no current capability claim may infer it, and any future extension needs a separate human-owned Work Item.",
    ),
    "docs/reference/capability-truth-matrix.json": (
        "reference-only",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "docs/capabilities.md",
            "docs/reference/reference-parity.md",
        ],
        "WI-330 confirms that the source 30-row public Capability Truth Matrix is not copied. Rust `capability_truth_registry` is a request-scoped observed-capability projection with explicit adopter/external exclusions, not a public claim matrix or authorization; strict row/evidence binding remains an optional future product decision requiring explicit human scope.",
    ),
    "docs/reference/capability-truth-matrix.md": (
        "reference-only",
        [
            "crates/cockpit-repository/src/lib.rs",
            "docs/capabilities.md",
            "docs/reference/reference-parity.md",
        ],
        "WI-330 closes the comparison without copying the source matrix documentation. Current target capability and adoption pages deliberately distinguish observed Runtime facts, repository evidence, adopter installation, provider evidence, and enterprise assurance; they do not advertise a source matrix or claim checker. A future implementation would require a dedicated human-owned Work Item and new Rust-native evidence semantics.",
    ),
}

WI331_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/checks-catalog.md": (
        "implemented-different-by-design",
        [
            "docs/reference/checks-catalog.md",
            "docs/reference/checks-catalog.zh-CN.md",
            "docs/reference/checks-catalog.ja.md",
            "docs/reference/ci-quality-gates.md",
            "tests/ci/repository_gate_manifest.json",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source check catalog is represented by tri-language Rust-native documentation, the versioned gate manifest, and Runtime Contract/lifecycle validators. Target profiles remain policy-selected and distinguish verification coverage from Evidence Assurance; source Make/Python command ownership is not copied.",
    ),
    "docs/reference/ci-release-evidence.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-release-evidence.md",
            "docs/reference/ci-release-evidence.zh-CN.md",
            "docs/reference/ci-release-evidence.ja.md",
            "tests/ci/repository_gate_manifest.json",
            ".github/workflows/ci.yml",
            ".github/workflows/release.yml",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
            "docs/release/distribution.md",
        ],
        "The source CI/Release evidence contract is represented by the provider-bound Rust Contract gate, versioned manifest, CI/release workflows, checksum/SBOM/provenance policy, and published-binary adopter harness. Local, provider, public Release, and enterprise evidence remain separate; PR prose and source fixtures are never treated as proof.",
    ),
}

WI332_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/comprehension-review-2026-08-14.md": (
        "reference-only",
        [
            "docs/README.md",
            "docs/philosophy.md",
            "docs/architecture.md",
            "docs/reference/agent-workflow.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "This source file is a historical P0 desk-review evidence record authored for the reference repository. Its reviewer result cannot be transferred as target evidence or recreated without an actual independent review. The target preserves the six-question reader route through its English documentation and acceptance checks, while keeping native editorial quality explicitly unverified.",
    ),
    "docs/reference/comprehension-review-2026-08-14.zh-CN.md": (
        "reference-only",
        [
            "docs/README.zh-CN.md",
            "docs/philosophy.zh-CN.md",
            "docs/architecture.zh-CN.md",
            "docs/reference/agent-workflow.zh-CN.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "This source file is a historical Simplified Chinese desk-review evidence record from the reference repository, not a portable claim about the target. The target supplies the same reader questions through localized routes and link checks, but does not invent a native-language reviewer result or copy source evidence bytes.",
    ),
    "docs/reference/comprehension-review-2026-08-14.ja.md": (
        "reference-only",
        [
            "docs/README.ja.md",
            "docs/philosophy.ja.md",
            "docs/architecture.ja.md",
            "docs/reference/agent-workflow.ja.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "This source file is a historical Japanese desk-review evidence record from the reference repository. The target keeps the six-question reader route and localized link checks, but cannot claim an independent native editorial review or transfer the source review's score as evidence.",
    ),
}

WI333_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/comprehension-validation-protocol.md": (
        "reference-only",
        [
            "docs/README.md",
            "docs/reference/agent-workflow.md",
            "docs/reference/reference-file-comparison.md",
            "docs/features/human-benefit-report.md",
        ],
        "The source protocol is an external human-reader study procedure, not Runtime authority or a participant-recruitment service. The target preserves a reader-first route and evidence/benefit boundary but does not claim to run the source study or collect participants.",
    ),
    "docs/reference/comprehension-validation-protocol.zh-CN.md": (
        "reference-only",
        [
            "docs/README.zh-CN.md",
            "docs/reference/agent-workflow.zh-CN.md",
            "docs/reference/reference-file-comparison.zh-CN.md",
            "docs/features/human-benefit-report.zh-CN.md",
        ],
        "The source Chinese protocol is a reference-repository human-study procedure. The target keeps the localized reader route and privacy/evidence boundary, but has no target participant study and does not transfer source study authority.",
    ),
    "docs/reference/comprehension-validation-protocol.ja.md": (
        "reference-only",
        [
            "docs/README.ja.md",
            "docs/reference/agent-workflow.ja.md",
            "docs/reference/reference-file-comparison.ja.md",
            "docs/features/human-benefit-report.ja.md",
        ],
        "The source Japanese protocol is a reference-repository human-study procedure. The target keeps the localized reader route and privacy/evidence boundary, but has no target participant study and does not transfer source study authority.",
    ),
    "docs/reference/comprehension-validation-response.schema.json": (
        "reference-only",
        [
            ".ai/README.md",
            "docs/reference/outcome-report.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "This schema governs raw participant answers for the source study. The target intentionally stores no participant-identifying or study-response records; repository Outcome/evidence schemas are a different governance surface and cannot substitute for participant evidence.",
    ),
    "docs/reference/comprehension-validation-responses/peter_01.en.json": (
        "reference-only",
        [
            "docs/README.md",
            "docs/features/human-benefit-report.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "Historical anonymized English participant evidence is bound to the source revision and study protocol. It cannot be copied or presented as evidence that the target documentation was understood.",
    ),
    "docs/reference/comprehension-validation-responses/peter_02.en.json": (
        "reference-only",
        [
            "docs/README.md",
            "docs/features/human-benefit-report.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "Historical current-revision English participant evidence belongs to the source repository. The target does not claim an equivalent participant receipt or copy the raw answer.",
    ),
    "docs/reference/comprehension-validation-responses/tanaka_01.ja.json": (
        "reference-only",
        [
            "docs/README.ja.md",
            "docs/features/human-benefit-report.ja.md",
            "docs/reference/reference-file-comparison.ja.md",
        ],
        "Historical anonymized Japanese participant evidence is source-bound and non-transferable. The target preserves the Japanese reader route without inventing a native participant result.",
    ),
    "docs/reference/comprehension-validation-responses/tanaka_02.ja.json": (
        "reference-only",
        [
            "docs/README.ja.md",
            "docs/features/human-benefit-report.ja.md",
            "docs/reference/reference-file-comparison.ja.md",
        ],
        "Historical current-revision Japanese participant evidence belongs to the source repository. The target does not claim an equivalent participant receipt or copy the raw answer.",
    ),
    "docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json": (
        "reference-only",
        [
            "docs/README.zh-CN.md",
            "docs/features/human-benefit-report.zh-CN.md",
            "docs/reference/reference-file-comparison.zh-CN.md",
        ],
        "Historical anonymized Simplified Chinese participant evidence is source-bound and non-transferable. The target preserves the Chinese reader route without inventing a native participant result.",
    ),
    "docs/reference/comprehension-validation-responses/xiaoli_02.zh-CN.json": (
        "reference-only",
        [
            "docs/README.zh-CN.md",
            "docs/features/human-benefit-report.zh-CN.md",
            "docs/reference/reference-file-comparison.zh-CN.md",
        ],
        "Historical current-revision Simplified Chinese participant evidence belongs to the source repository. The target does not claim an equivalent participant receipt or copy the raw answer.",
    ),
    "docs/reference/comprehension-validation-results.json": (
        "reference-only",
        [
            "docs/features/human-benefit-report.md",
            "docs/features/human-benefit-report.zh-CN.md",
            "docs/features/human-benefit-report.ja.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "The source result is a revision-bound participant-study claim with source commits and sample receipts. Target human-benefit and Outcome projections do not authorize a comprehension result, release, or enterprise claim.",
    ),
    "docs/reference/comprehension-validation-results.md": (
        "reference-only",
        [
            "docs/features/human-benefit-report.md",
            "docs/reference/outcome-report.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "The source result report is an immutable historical study claim. The target documents the limitation that Runtime evidence, Agent self-review, and link checks are not participant evidence; no result is copied or generalized.",
    ),
}

WI334_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/content-bound-evidence-reuse.md": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/src/lib.rs",
            "crates/cockpit-evidence/tests/reuse.rs",
            "docs/reference/configuration.md",
            "docs/reference/cross-work-item-dedup.md",
        ],
        "The target keeps exact content identity as one component of a composite EvidenceContext and never reuses a receipt unless every bound identity matches. The Rust API intentionally replaces the source Python content policy and does not claim source wire compatibility.",
    ),
    "docs/reference/diff-bound-evidence-reuse.md": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/src/lib.rs",
            "crates/cockpit-evidence/tests/reuse.rs",
            "crates/cockpit-git/src/lib.rs",
            "docs/reference/configuration.md",
        ],
        "The target binds base/head revisions and changed-path identity in typed DiffIdentity, validates canonical SHA-256 bindings, and reruns on mismatch. The source Python helper and command surface are not copied.",
    ),
    "docs/reference/environment-bound-reuse.md": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/src/lib.rs",
            "crates/cockpit-evidence/tests/reuse.rs",
            "crates/cockpit-verification/src/lib.rs",
            "docs/reference/configuration.md",
        ],
        "The target records environment, toolchain, Runtime, profile, policy, command, and stage identity in a strict composite EvidenceContext. Unknown, expired, failed, protected, or mismatched receipts execute again; no process environment is serialized wholesale.",
    ),
    "docs/reference/evidence-binding-foundation.md": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/src/lib.rs",
            "crates/cockpit-evidence/tests/reuse.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The versioned Rust ReusableReceipt and EvidenceContext provide a stricter composite identity and repository-local receipt store. Validation is fail-closed and advisory: it can select reuse but never bypass governance, protected nodes, or required checks.",
    ),
    "scripts/ai_evidence_binding.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/src/lib.rs",
            "crates/cockpit-evidence/tests/reuse.rs",
        ],
        "The source binding builder/validator is represented by typed Rust structs, content-addressed receipt IDs, deny-unknown-fields parsing, and deterministic Unknown/Stale-to-execute decisions. Source Python APIs are not shipped.",
    ),
    "scripts/ai_diff_bound_reuse.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/src/lib.rs",
            "crates/cockpit-evidence/tests/reuse.rs",
            "crates/cockpit-git/src/lib.rs",
        ],
        "Typed DiffIdentity and repository snapshot facts replace the source diff helper. Base/head, changed-path, scope, governance, and expiry mismatches remain rerun conditions.",
    ),
    "scripts/ai_environment_reuse.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/src/lib.rs",
            "crates/cockpit-evidence/tests/reuse.rs",
            "crates/cockpit-verification/src/lib.rs",
        ],
        "The source environment adapter is represented by explicit Runtime/toolchain/environment digests in EvidenceContext. The Rust executor accepts bounded environment inputs and does not expose a wholesale environment snapshot API.",
    ),
    "tests/test_ai_evidence_binding.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/tests/reuse.rs",
            "crates/cockpit-repository/tests/receipt_store.rs",
        ],
        "Rust tests cover strict receipt schema, content/diff/environment identity mismatch, expiry, failed/protected nodes, tampering, and deterministic fail-closed execution; the source pytest corpus is not copied.",
    ),
    "tests/test_ai_diff_bound_reuse.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/tests/reuse.rs",
            "crates/cockpit-git/tests/repository.rs",
        ],
        "Rust tests cover exact composite diff identity, clean and changed path sets, canonical ordering, malformed/traversal inputs, policy mismatch, expiry, and input immutability without source test-wire parity.",
    ),
    "tests/test_ai_environment_reuse.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-evidence/tests/reuse.rs",
            "crates/cockpit-verification/tests/execution.rs",
        ],
        "Rust tests cover environment/toolchain identity, Runtime and profile binding, stale/unknown/failed receipts, protected execution, and strict digest validation. Secret filtering remains an explicit external-input boundary rather than a copied Python module.",
    ),
}

WI342_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/distribution.md": (
        "implemented-different-by-design",
        [
            "docs/release/distribution.md",
            "docs/release/distribution.zh-CN.md",
            "docs/release/distribution.ja.md",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
        ],
        "The source distribution reference is represented by the target's release/distribution route and immutable public/N-1 adopter acceptance harness. Rust documents explicit shared-Runtime installation, repository binding, checksum/SBOM/provenance and cleanup boundaries; source Make/Python commands and source release bytes are not copied.",
    ),
    "docs/reference/distribution.ja.md": (
        "implemented-different-by-design",
        [
            "docs/release/distribution.ja.md",
            "docs/release/distribution.zh-CN.md",
            "docs/release/distribution.md",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
        ],
        "The Japanese source distribution route is preserved in the target's tri-language release/distribution pages and public/N-1 acceptance harness. Target-specific Rust Runtime and repository binding semantics replace source Make/Python installer details without claiming byte or wire parity.",
    ),
    "docs/reference/documentation-architecture.md": (
        "implemented-different-by-design",
        [
            "docs/current/README.md",
            "docs/getting-started/README.md",
            "docs/reference/README.md",
            "docs/reference/reference-file-comparison.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The source documentation layers, reader-criticality, canonical ownership, multilingual map, and split rules are represented by the target current/getting-started/reference routes, tri-language acceptance checks, and explicit parity ledger. The target keeps the installed Runtime and repository-local .ai read set as authority; source make/Python documentation tooling is not copied.",
    ),
    "docs/reference/documentation-architecture.ja.md": (
        "implemented-different-by-design",
        [
            "docs/current/README.ja.md",
            "docs/getting-started/README.ja.md",
            "docs/reference/README.ja.md",
            "docs/reference/reference-file-comparison.ja.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The Japanese source reader map and layer ownership are represented by the target Japanese current/getting-started/reference routes and shared tri-language acceptance checks. The source navigation prose is not a Runtime authority file; repository-local .ai instructions and explicit Rust pages remain the boundary.",
    ),
    "docs/reference/documentation-authority-boundary.md": (
        "implemented-different-by-design",
        [
            "docs/current/README.md",
            "docs/current/README.zh-CN.md",
            "docs/current/README.ja.md",
            ".ai/README.md",
            "AGENTS.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The target preserves the source separation between current instructions, supporting reference pages, and historical records through the repository-owned .ai read set, AGENTS.md, current/reference routes, frontmatter, and documentation acceptance. It intentionally has no second generic authority router or source-specific Python command.",
    ),
    "docs/reference/documentation-authority-registry.json": (
        "implemented-different-by-design",
        [
            "docs/current/README.md",
            "docs/current/README.zh-CN.md",
            "docs/current/README.ja.md",
            "docs/reference/README.md",
            "docs/reference/README.zh-CN.md",
            "docs/reference/README.ja.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The source machine-readable authority/topic registry is replaced by explicit target current/reference navigation, frontmatter authority/status metadata, tri-language route checks, and the .ai/AGENTS read boundary. No source topic inventory is treated as Runtime capability or copied as a global Agent configuration.",
    ),
    "docs/reference/documentation-context-registry.json": (
        "reference-only",
        [
            "docs/current/README.md",
            "docs/reference/README.md",
            ".ai/README.md",
            ".ai/glossary.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "The source registry classifies source-specific plans and historical records. Those context labels are not portable adopter evidence or Runtime authority. The target explicitly keeps current instructions in .ai/README.md and AGENTS.md, reference pages opt-in, and Work Item/archive records immutable; no source plan registry is copied.",
    ),
    "docs/reference/enterprise-control-checklist.md": (
        "implemented-different-by-design",
        [
            "docs/security/enterprise-governance.md",
            "docs/security/enterprise-governance.zh-CN.md",
            "docs/security/enterprise-governance.ja.md",
            "docs/security/enterprise-deployment-boundary.md",
            "docs/getting-started/adopter-configuration.md",
        ],
        "The source adopter checklist is represented by the target enterprise-governance, deployment-boundary, and adopter-configuration routes. Rust distinguishes repository facts, delegated provider evidence, retention/audit responsibilities, and non-certification claims; a checklist row never becomes enterprise approval by presence alone.",
    ),
    "docs/reference/enterprise-control-matrix.json": (
        "reference-only",
        [
            "docs/security/enterprise-governance.md",
            "docs/security/enterprise-deployment-boundary.md",
            "docs/getting-started/adopter-configuration.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "The source JSON is an observed-control inventory owned by the reference adopter context, not a portable compliance result. The target documents the same external-control boundary and imports typed delegated evidence when supplied, but does not copy source not_verified rows or infer organization state.",
    ),
    "docs/reference/external-identity-boundary.md": (
        "implemented-different-by-design",
        [
            "docs/security/enterprise-governance.md",
            "docs/security/enterprise-governance.zh-CN.md",
            "docs/security/enterprise-governance.ja.md",
            "docs/reference/contract-fields.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/tests/contract_v2.rs",
        ],
        "The source identity levels and approval boundary are implemented through typed Rust authority/approval evidence, policy precedence, external delegated evidence, and tri-language enterprise documentation. Repository declarations never authenticate a person; provider and enterprise assurance remain external and must be bound explicitly.",
    ),
}

WI343_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/cross-wi-integration.md": (
        "reference-only",
        [
            "docs/reference/reference-parity.md",
            "docs/reference/outcome-report.md",
            ".ai/work-items/archive/",
        ],
        "The source aggregate report is an advisory historical integration view. The target's per-Work-Item archive validation, reference-parity ledger, and human Outcome boundary provide the corresponding audit surfaces without adding a cross-Work-Item Runtime report or claiming an observable conversation receipt.",
    ),
    "docs/reference/dependabot-intake.md": (
        "not-applicable",
        [
            "docs/reference/ci-release-evidence.md",
            "Cargo.toml",
            "Cargo.lock",
        ],
        "Dependabot bot-branch intake is provider-specific and is not a Runtime capability. The target keeps generic delegated provider evidence and explicit Work Item source binding, while dependency facts and update-service selection remain repository/provider responsibilities.",
    ),
    "docs/reference/deprecated-assets-registry.json": (
        "reference-only",
        [
            ".ai/README.md",
            "docs/reference/agent-workflow.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source registry is a source-repository cleanup inventory, not a portable Runtime protocol. Explicit lifecycle closure, immutable history, and exact resource finalization provide the target cleanup boundary without shipping a source deletion registry or Make scan.",
    ),
    "docs/reference/deprecated-assets.md": (
        "reference-only",
        [
            "docs/reference/agent-workflow.md",
            "docs/reference/reference-parity.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source obsolete-command and registry-hygiene explanation remains reference documentation. Rust uses explicit --repo lifecycle commands, immutable archives, and reviewed resource finalization; it does not claim the source check-deprecated-assets command.",
    ),
    "docs/reference/derived-artifacts.md": (
        "implemented-different-by-design",
        [
            "docs/reference/outcome-report.md",
            "docs/reference/verification-semantics.md",
            ".ai/README.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The target preserves the source fact-versus-view boundary through typed Contract, evidence, archive, status, and Outcome projections. Derived views cannot authorize later decisions; no source Python registry or second authority is required or read.",
    ),
}

WI344_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/failure-recovery-usability.md": (
        "implemented-different-by-design",
        [
            "docs/reference/troubleshooting.md",
            "docs/features/task-outcome-report.md",
            "docs/reference/outcome-report.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The target provides a repository-bound recovery and human Outcome projection with explicit failed gates, recovery conditions, interventions, stops, resolutions, and next actions. The source nine-scenario Python report validator and exact report wire shape are not copied; companion source scripts/tests remain separately staged for semantic comparison.",
    ),
    "docs/reference/final-north-star-acceptance.json": (
        "implemented-different-by-design",
        [
            "docs/reference/final-replacement-acceptance.md",
            "docs/reference/reference-parity.md",
            "tests/conformance/final_replacement_acceptance.sh",
        ],
        "The source twenty-dimension acceptance decision is represented by the target's bounded final-replacement acceptance route and exact dimension/parity documentation. The target preserves explicit external-adopter/provider limitations and does not copy the source JSON decision bytes.",
    ),
    "docs/reference/final-north-star-acceptance.md": (
        "implemented-different-by-design",
        [
            "docs/reference/final-replacement-acceptance.md",
            "docs/reference/outcome-report.md",
            "docs/reference/reference-parity.md",
        ],
        "The target keeps the North Star boundary through final-replacement acceptance, evidence-bound Outcome, and reference-parity routes. Local tests cannot substitute for external adopter/provider evidence, and the source evaluator prose is not copied as current release authority.",
    ),
    "docs/reference/final-wiii-remediation-closure-audit.md": (
        "reference-only",
        [
            "docs/reference/reference-parity.md",
            "docs/reference/agent-workflow.md",
            "docs/reference/work-item-intelligence-interface.md",
        ],
        "This is a source-repository-specific historical audit of the reference Work Item Intelligence remediation and its provider PR history. The target documents its own Rust-native Work Item and parallelism boundaries but must not import source PR identities, reviewer claims, or historical closure evidence.",
    ),
    "docs/reference/full-remediation-acceptance.md": (
        "reference-only",
        [
            "docs/reference/final-replacement-acceptance.md",
            "docs/reference/reference-parity.md",
            "docs/reference/outcome-report.md",
        ],
        "This is an internal source-project acceptance baseline for source WI-01 through WI-19 and its historical release sequence. The target retains only its own evidence-bound acceptance and reader routes; source Work Item history, progress gates, and release claims are not portable adopter capability.",
    ),
}

WI346_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/governance-profiles.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/governance-profiles.ja.md",
            "docs/reference/governance-profile-cost-separation.ja.md",
            "docs/reference/ci-quality-gates.ja.md",
            "docs/reference/verification-route.ja.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The reference risk-based Light/Standard/Strict guidance is preserved in a Japanese Rust-native route with release as an operation escalation, cost separated from verification and assurance, and fail-closed controls. Source Make/Python dispatch and source wire shapes are not copied.",
    ),
    "docs/reference/governance-profiles.md": (
        "implemented-different-by-design",
        [
            "docs/reference/governance-profiles.md",
            "docs/reference/governance-profile-cost-separation.md",
            "docs/reference/ci-quality-gates.md",
            "docs/reference/verification-route.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The reference risk-based Light/Standard/Strict guidance is preserved in a Rust-native route with release as an operation escalation, cost separated from verification and assurance, and fail-closed controls. Source Make/Python dispatch and source wire shapes are not copied.",
    ),
    "docs/reference/governance-profiles.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/governance-profiles.zh-CN.md",
            "docs/reference/governance-profile-cost-separation.zh-CN.md",
            "docs/reference/ci-quality-gates.zh-CN.md",
            "docs/reference/verification-route.zh-CN.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "源文件关于 Light/Standard/Strict 风险质量路由的语义由 Rust 原生路线保留：release 是操作升级，成本与 Verification/Assurance 分离，强制控制保持 fail-closed。不复制源 Make/Python 调度或源 wire shape。",
    ),
    "docs/reference/how-to-read-cockpit-status.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/how-to-read-cockpit-status.ja.md",
            "docs/reference/outcome-report.ja.md",
            "docs/reference/commands.ja.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The reference human status-reading order and semantic color signals are projected onto the target's visible tri-language Outcome and request-scoped status commands. The target preserves contract text and never infers approval from a color; source report wire fields are not copied.",
    ),
    "docs/reference/how-to-read-cockpit-status.md": (
        "implemented-different-by-design",
        [
            "docs/reference/how-to-read-cockpit-status.md",
            "docs/reference/outcome-report.md",
            "docs/reference/commands.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The reference human status-reading order and semantic color signals are projected onto the target's visible tri-language Outcome and request-scoped status commands. The target preserves contract text and never infers approval from a color; source report wire fields are not copied.",
    ),
    "docs/reference/how-to-read-cockpit-status.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/how-to-read-cockpit-status.zh-CN.md",
            "docs/reference/outcome-report.zh-CN.md",
            "docs/reference/commands.zh-CN.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "源文件关于面向人的 status 阅读顺序和颜色信号，由目标的三语可见 Outcome 与 request-scoped status 命令投影。目标保留 Contract 原文，不从颜色推断批准；不复制源报告 wire 字段。",
    ),
}

WI347_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/human-report-semantic-quality.md": (
        "implemented-different-by-design",
        [
            "docs/features/human-benefit-report.md",
            "docs/features/task-outcome-report.md",
            "docs/reference/outcome-report.md",
        ],
        "The reference human-benefit ordering and forbidden-claim boundary are represented by the Rust Outcome and task-report projections. Source report prose is not an independent authority and is not copied.",
    ),
    "docs/reference/implementation-knowledge.ja.md": (
        "implemented-different-by-design",
        ["docs/reference/implementation-knowledge.ja.md", "crates/cockpit-knowledge/src/lib.rs"],
        "Japanese implementation-knowledge semantics are documented on the Rust read-only projection; source filters and generated records are not copied.",
    ),
    "docs/reference/implementation-knowledge.md": (
        "implemented-different-by-design",
        ["docs/reference/implementation-knowledge.md", "crates/cockpit-knowledge/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs"],
        "The target exposes deterministic repository-bound Knowledge records and four conjunctive CLI/MCP filters. The reference's wider date/commit/supersession query surface remains an explicit non-claim.",
    ),
    "docs/reference/implementation-knowledge.zh-CN.md": (
        "implemented-different-by-design",
        ["docs/reference/implementation-knowledge.zh-CN.md", "crates/cockpit-knowledge/src/lib.rs"],
        "中文实现知识页面映射 Rust 只读投影，明确当前过滤器和未实现的更宽查询维度；不复制源生成记录。",
    ),
    "docs/reference/input-trust-dataflow.ja.md": (
        "implemented-different-by-design",
        ["docs/reference/input-trust-dataflow.ja.md", "crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs"],
        "Japanese provenance guidance maps to typed FactOrigin/TraceableFact/TraceableDerivation and fail-closed repository observation; source Python trust code is not copied.",
    ),
    "docs/reference/input-trust-dataflow.md": (
        "implemented-different-by-design",
        ["docs/reference/input-trust-dataflow.md", "crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-repository/tests/input_trust.rs"],
        "The target preserves provenance classification, cross-step traceability, and prompt-injection boundaries through typed Rust facts and repository tests; it does not claim source JSON wire or external authentication parity.",
    ),
    "docs/reference/input-trust-dataflow.zh-CN.md": (
        "implemented-different-by-design",
        ["docs/reference/input-trust-dataflow.zh-CN.md", "crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/tests/input_trust.rs"],
        "中文输入信任数据流由类型化来源、追溯派生和 fail-closed 测试承担；不复制源 Python 模块或宣称 provider 身份认证。",
    ),
    "docs/reference/installed-lifecycle.md": (
        "implemented-different-by-design",
        ["docs/reference/installed-lifecycle.md", "docs/release/distribution.md", "docs/getting-started/installation.md", "docs/architecture/versioning.md"],
        "The shared Rust Runtime, explicit attach, immutable Release acceptance, and separate migration boundary represent the source lifecycle responsibility. Source Python installer/Make orchestration is external reference material.",
    ),
    "docs/reference/instruction-traceability.md": (
        "implemented-different-by-design",
        ["docs/reference/instruction-traceability.md", "tests/conformance/reference_file_inventory.json", "docs/reference/reference-file-comparison.md", "docs/reference/reference-parity.md"],
        "The target inventory, comparison/parity records, Work Item evidence, and closure receipts provide structural forward/reverse traceability. The source remediation manifest and checker are not copied as Runtime authority.",
    ),
    "docs/reference/japanese-capability-assessment.json": (
        "implemented-different-by-design",
        ["docs/reference/japanese-capability-assessment.md", "docs/reference/japanese-capability-assessment.zh-CN.md", "docs/reference/japanese-capability-assessment.ja.md", "tests/docs/documentation_acceptance.sh", "tests/cli/intelligence.rs"],
        "The source release assessment is projected to Rust tri-language docs and executable presentation/adversarial tests. Source assessment bytes, Python calibration, and participant evidence remain reference-bound; no general fluency claim is made.",
    ),
}

WI348_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/japanese-capability-assessment.md": (
        "implemented-different-by-design",
        [
            "docs/reference/japanese-capability-assessment.md",
            "docs/reference/japanese-capability-assessment.zh-CN.md",
            "docs/reference/japanese-capability-assessment.ja.md",
            "tests/cli/intelligence.rs",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The source assessment matrix is mapped to bounded tri-language reader, Outcome, adversarial, installation, and documentation checks. Source assessment JSON/Python calibration and general fluency claims remain reference-bound.",
    ),
    "docs/reference/lightweight-verification-and-soft-gates.md": (
        "implemented-different-by-design",
        [
            "docs/reference/lightweight-verification-and-soft-gates.md",
            "docs/reference/lightweight-verification-and-soft-gates.zh-CN.md",
            "docs/reference/lightweight-verification-and-soft-gates.ja.md",
            "docs/reference/governance-profiles.md",
            "crates/cockpit-verification/src/lib.rs",
            "crates/cockpit-evidence/src/lib.rs",
        ],
        "Rust preserves proportional light/standard/strict routing, content-addressed reuse, deterministic dependency handling, monotonic escalation, and visible soft/unknown boundaries without copying the source Python/Make checker.",
    ),
    "docs/reference/multilingual-semantic-parity.md": (
        "implemented-different-by-design",
        [
            "docs/reference/multilingual-semantic-parity.md",
            "docs/reference/multilingual-semantic-parity.zh-CN.md",
            "docs/reference/multilingual-semantic-parity.ja.md",
            "crates/cockpit-cli/tests/intelligence.rs",
            "crates/cockpit-repository/src/outcome_render.rs",
            "docs/reference/outcome-report.md",
        ],
        "The target tests equivalent Runtime-owned labels, markers, safety, unknown, decision, limitation, and next-action semantics in three languages while preserving Contract text in its authoring language; source comparator wire and arbitrary prose translation are not copied.",
    ),
    "docs/reference/open-pr-issue-reconciliation-662.json": (
        "reference-only",
        [
            "docs/reference/provider-reconciliation-boundary.md",
            "docs/reference/provider-reconciliation-boundary.zh-CN.md",
            "docs/reference/provider-reconciliation-boundary.ja.md",
            "docs/reference/reference-parity.md",
        ],
        "This is a historical source/provider inventory. It cannot prove current GitHub, release, or enterprise state and is retained only as pinned reference context.",
    ),
    "docs/reference/open-pr-issue-reconciliation-662.md": (
        "reference-only",
        [
            "docs/reference/provider-reconciliation-boundary.md",
            "docs/reference/provider-reconciliation-boundary.zh-CN.md",
            "docs/reference/provider-reconciliation-boundary.ja.md",
            "docs/reference/reference-parity.md",
        ],
        "This historical reconciliation narrative is source-bound and cannot authorize a target merge, release, or close. Current provider observations require fresh external evidence.",
    ),
    "docs/reference/operation-time-policy-reevaluation.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/operation-time-policy-reevaluation.ja.md",
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-core/tests/operation_time_policy.rs",
        ],
        "The Rust Core supplies a strict operation-time request/decision evaluator with the same high-risk categories and fail-closed mismatch rules; source Python trust modules and provider execution remain external.",
    ),
    "docs/reference/operation-time-policy-reevaluation.md": (
        "implemented-different-by-design",
        [
            "docs/reference/operation-time-policy-reevaluation.md",
            "docs/reference/operation-time-policy-reevaluation.zh-CN.md",
            "docs/reference/operation-time-policy-reevaluation.ja.md",
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-core/tests/operation_time_policy.rs",
        ],
        "The Rust Core supplies a strict operation-time request/decision evaluator with explicit operation, target, scope, authority, freshness, trust, and impact facts. It evaluates but never executes or grants provider permission.",
    ),
    "docs/reference/operation-time-policy-reevaluation.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/operation-time-policy-reevaluation.zh-CN.md",
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-core/tests/operation_time_policy.rs",
        ],
        "Rust Core 的严格操作时请求/决定评估器覆盖源语义的高风险操作和 fail-closed 绑定检查；评估不执行操作，也不授予 provider 权限。",
    ),
    "docs/reference/performance-diagnosis.md": (
        "implemented-different-by-design",
        [
            "docs/reference/performance-diagnosis.md",
            "docs/reference/performance-diagnosis.zh-CN.md",
            "docs/reference/performance-diagnosis.ja.md",
            "docs/reference/governance-cost-metrics.md",
            "crates/cockpit-verification/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "Rust request-scoped diagnosis and cost observations cover measured execution/reuse facts and preserve unknown/partial boundaries. The source JSONL parser, provider waits, P95, and performance claims are not invented.",
    ),
    "docs/reference/pre-release-documentation-alignment.json": (
        "reference-only",
        [
            "docs/reference/provider-reconciliation-boundary.md",
            "docs/reference/reference-file-comparison.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "This generated source alignment receipt is historical assessment evidence, not target Runtime authority or current release proof. Target documentation is validated by its own repository-local checks and evidence.",
    ),
}

WI345_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/governance-complexity.ja.md": (
        "reference-only",
        ["docs/reference/governance-complexity.ja.md", "docs/reference/reference-parity.ja.md", "docs/reference/governance-integrity-gate.ja.md"],
        "The source document explains its Python/Make complexity scanner and source archive-maintenance policy. The target keeps immutable archive and repository-integrity boundaries, but does not ship that source-specific scanner or claim equivalent complexity metrics; this target page records the non-portable boundary.",
    ),
    "docs/reference/governance-complexity.md": (
        "reference-only",
        ["docs/reference/governance-complexity.md", "docs/reference/reference-parity.md", "docs/reference/governance-integrity-gate.md"],
        "The source document explains its Python/Make complexity scanner and source archive-maintenance policy. The target keeps immutable archive and repository-integrity boundaries, but does not ship that source-specific scanner or claim equivalent complexity metrics; this target page records the non-portable boundary.",
    ),
    "docs/reference/governance-cost-metrics.md": (
        "implemented-different-by-design",
        ["docs/reference/governance-cost-metrics.md", "docs/reference/verification-cost.md", "crates/cockpit-cli/src/main.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-verification/src/lib.rs"],
        "Rust exposes repository-bound diagnose plus typed VerificationCostEstimate/Observation and receipt telemetry. It does not copy the source JSONL lifecycle event parser, phase/wait taxonomy, or source report wire shape; cost output remains advisory and identity-bound.",
    ),
    "docs/reference/governance-performance-budget.md": (
        "implemented-different-by-design",
        ["docs/reference/governance-performance-budget.md", "tests/performance/README.md", "crates/cockpit-verification/src/lib.rs", "tests/performance/regression_gate.sh"],
        "Rust uses identity-bound PerformanceBaseline samples and explicit regression budgets, while the source P95/profile report is not Runtime authority. Budget overrun never removes required verification, and no automatic P95 or governance profile is inferred.",
    ),
    "docs/reference/governance-profile-cost-separation.md": (
        "implemented-different-by-design",
        ["docs/reference/governance-profile-cost-separation.md", "docs/reference/ci-quality-gates.md", "docs/reference/verification-route.md", "crates/cockpit-verification/src/lib.rs"],
        "Rust keeps light/standard/strict routing and separates VerificationTier from EvidenceAssurance. Operation/stage policy and protected gates drive escalation; cost observations cannot lower requirements, and no source profile name or hidden automatic decision is imported.",
    ),
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
        wi287 = WI287_REFERENCE_FILES.get(path)
        if wi287 is not None:
            counterparts, reason = wi287
            records.append(
                {
                    "referencePath": path,
                    "batch": WI287_BATCH,
                    "classification": "implemented-different-by-design",
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi272 = WI272_REFERENCE_FILES.get(path)
        if wi272 is not None:
            classification, counterparts, reason = wi272
            records.append(
                {
                    "referencePath": path,
                    "batch": "WI-272-reference-agent-rule-batch",
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi293 = WI293_REFERENCE_FILES.get(path)
        if wi293 is not None:
            classification, counterparts, reason = wi293
            records.append(
                {
                    "referencePath": path,
                    "batch": "WI-293-ci-contract-aware-gates-recovery",
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi304 = WI304_REFERENCE_FILES.get(path)
        if wi304 is not None:
            classification, counterparts, reason = wi304
            records.append(
                {
                    "referencePath": path,
                    "batch": WI304_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi305 = WI305_REFERENCE_FILES.get(path)
        if wi305 is not None:
            classification, counterparts, reason = wi305
            records.append(
                {
                    "referencePath": path,
                    "batch": WI305_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi306 = WI306_REFERENCE_FILES.get(path)
        if wi306 is not None:
            classification, counterparts, reason = wi306
            records.append(
                {
                    "referencePath": path,
                    "batch": "WI-308-reference-file-comparison-batch-04-retry",
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi323 = WI323_REFERENCE_FILES.get(path)
        if wi323 is not None:
            classification, counterparts, reason = wi323
            records.append(
                {
                    "referencePath": path,
                    "batch": WI323_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi325 = WI325_REFERENCE_FILES.get(path)
        if wi325 is not None:
            classification, counterparts, reason = wi325
            records.append(
                {
                    "referencePath": path,
                    "batch": WI325_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi326 = WI326_REFERENCE_FILES.get(path)
        if wi326 is not None:
            classification, counterparts, reason = wi326
            records.append(
                {
                    "referencePath": path,
                    "batch": WI326_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi327 = WI327_REFERENCE_FILES.get(path)
        if wi327 is not None:
            classification, counterparts, reason = wi327
            records.append(
                {
                    "referencePath": path,
                    "batch": WI327_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi328 = WI328_REFERENCE_FILES.get(path)
        if wi328 is not None:
            classification, counterparts, reason = wi328
            records.append(
                {
                    "referencePath": path,
                    "batch": WI328_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi331 = WI331_REFERENCE_FILES.get(path)
        if wi331 is not None:
            classification, counterparts, reason = wi331
            records.append(
                {
                    "referencePath": path,
                    "batch": WI331_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi332 = WI332_REFERENCE_FILES.get(path)
        if wi332 is not None:
            classification, counterparts, reason = wi332
            records.append(
                {
                    "referencePath": path,
                    "batch": WI332_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi333 = WI333_REFERENCE_FILES.get(path)
        if wi333 is not None:
            classification, counterparts, reason = wi333
            records.append(
                {
                    "referencePath": path,
                    "batch": WI333_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi334 = WI334_REFERENCE_FILES.get(path)
        if wi334 is not None:
            classification, counterparts, reason = wi334
            records.append(
                {
                    "referencePath": path,
                    "batch": WI334_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi343 = WI343_REFERENCE_FILES.get(path)
        if wi343 is not None:
            classification, counterparts, reason = wi343
            records.append(
                {
                    "referencePath": path,
                    "batch": WI343_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi344 = WI344_REFERENCE_FILES.get(path)
        if wi344 is not None:
            classification, counterparts, reason = wi344
            records.append(
                {
                    "referencePath": path,
                    "batch": WI344_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi346 = WI346_REFERENCE_FILES.get(path)
        if wi346 is not None:
            classification, counterparts, reason = wi346
            records.append(
                {
                    "referencePath": path,
                    "batch": WI346_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi347 = WI347_REFERENCE_FILES.get(path)
        if wi347 is not None:
            classification, counterparts, reason = wi347
            records.append(
                {
                    "referencePath": path,
                    "batch": WI347_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi348 = WI348_REFERENCE_FILES.get(path)
        if wi348 is not None:
            classification, counterparts, reason = wi348
            records.append(
                {
                    "referencePath": path,
                    "batch": WI348_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi345 = WI345_REFERENCE_FILES.get(path)
        if wi345 is not None:
            classification, counterparts, reason = wi345
            records.append(
                {
                    "referencePath": path,
                    "batch": "WI-345-reference-governance-cost-batch-15",
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi342 = WI342_REFERENCE_FILES.get(path)
        if wi342 is not None:
            classification, counterparts, reason = wi342
            records.append(
                {
                    "referencePath": path,
                    "batch": WI342_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi302 = WI302_REFERENCE_FILES.get(path)
        if wi302 is not None:
            classification, counterparts, reason = wi302
            records.append(
                {
                    "referencePath": path,
                    "batch": WI302_BATCH,
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
    if any(
        isinstance(record, dict) and record.get("batch") == WI325_BATCH
        for record in records
    ):
        wi325_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI325_BATCH
        ]
        expected_wi325_paths = set(WI325_REFERENCE_FILES)
        actual_wi325_paths = {
            record.get("referencePath")
            for record in wi325_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi325_paths != expected_wi325_paths:
            errors.append(
                "WI-325 batch paths do not match the pinned nine-file set: "
                f"expected {sorted(expected_wi325_paths)!r}, got {sorted(actual_wi325_paths)!r}"
            )
        if len(wi325_records) != len(expected_wi325_paths):
            errors.append(
                f"WI-325 batch must contain {len(expected_wi325_paths)} records, found {len(wi325_records)}"
            )
        wi325_classifications = [record.get("classification") for record in wi325_records]
        if wi325_classifications.count("implemented-different-by-design") != 8:
            errors.append("WI-325 batch must contain eight implemented-different-by-design records")
        if wi325_classifications.count("reference-only") != 1:
            errors.append("WI-325 batch must contain one reference-only record")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi325_classifications
        ):
            errors.append("WI-325 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI326_BATCH
        for record in records
    ):
        wi326_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI326_BATCH
        ]
        expected_wi326_paths = set(WI326_REFERENCE_FILES)
        actual_wi326_paths = {
            record.get("referencePath")
            for record in wi326_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi326_paths != expected_wi326_paths:
            errors.append(
                "WI-326 batch paths do not match the pinned nine-file set: "
                f"expected {sorted(expected_wi326_paths)!r}, got {sorted(actual_wi326_paths)!r}"
            )
        if len(wi326_records) != len(expected_wi326_paths):
            errors.append(
                f"WI-326 batch must contain {len(expected_wi326_paths)} records, found {len(wi326_records)}"
            )
        wi326_classifications = [record.get("classification") for record in wi326_records]
        if wi326_classifications.count("implemented-different-by-design") != 8:
            errors.append("WI-326 batch must contain eight implemented-different-by-design records")
        if wi326_classifications.count("reference-only") != 1:
            errors.append("WI-326 batch must contain one reference-only record")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi326_classifications
        ):
            errors.append("WI-326 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI327_BATCH
        for record in records
    ):
        wi327_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI327_BATCH
        ]
        expected_wi327_paths = set(WI327_REFERENCE_FILES)
        actual_wi327_paths = {
            record.get("referencePath")
            for record in wi327_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi327_paths != expected_wi327_paths:
            errors.append(
                "WI-327 batch paths do not match the pinned nine-file set: "
                f"expected {sorted(expected_wi327_paths)!r}, got {sorted(actual_wi327_paths)!r}"
            )
        if len(wi327_records) != len(expected_wi327_paths):
            errors.append(
                f"WI-327 batch must contain {len(expected_wi327_paths)} records, found {len(wi327_records)}"
            )
        wi327_classifications = [record.get("classification") for record in wi327_records]
        if wi327_classifications.count("implemented-different-by-design") != 8:
            errors.append("WI-327 batch must contain eight implemented-different-by-design records")
        if wi327_classifications.count("reference-only") != 1:
            errors.append("WI-327 batch must contain one reference-only record")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi327_classifications
        ):
            errors.append("WI-327 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI328_BATCH
        for record in records
    ):
        wi328_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI328_BATCH
        ]
        expected_wi328_paths = set(WI328_REFERENCE_FILES)
        actual_wi328_paths = {
            record.get("referencePath")
            for record in wi328_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi328_paths != expected_wi328_paths:
            errors.append(
                "WI-328 batch paths do not match the pinned nine-file set: "
                f"expected {sorted(expected_wi328_paths)!r}, got {sorted(actual_wi328_paths)!r}"
            )
        if len(wi328_records) != len(expected_wi328_paths):
            errors.append(
                f"WI-328 batch must contain {len(expected_wi328_paths)} records, found {len(wi328_records)}"
            )
        wi328_classifications = [record.get("classification") for record in wi328_records]
        if wi328_classifications.count("implemented-different-by-design") != 5:
            errors.append("WI-328 batch must contain five implemented-different-by-design records")
        if wi328_classifications.count("reference-only") != 4:
            errors.append("WI-328 batch must contain four reference-only records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi328_classifications
        ):
            errors.append("WI-328 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI331_BATCH
        for record in records
    ):
        wi331_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI331_BATCH
        ]
        expected_wi331_paths = set(WI331_REFERENCE_FILES)
        actual_wi331_paths = {
            record.get("referencePath")
            for record in wi331_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi331_paths != expected_wi331_paths:
            errors.append(
                "WI-331 batch paths do not match the pinned two-file set: "
                f"expected {sorted(expected_wi331_paths)!r}, got {sorted(actual_wi331_paths)!r}"
            )
        if len(wi331_records) != len(expected_wi331_paths):
            errors.append(
                f"WI-331 batch must contain {len(expected_wi331_paths)} records, found {len(wi331_records)}"
            )
        wi331_classifications = [record.get("classification") for record in wi331_records]
        if wi331_classifications.count("implemented-different-by-design") != len(expected_wi331_paths):
            errors.append("WI-331 batch must contain two implemented-different-by-design records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi331_classifications
        ):
            errors.append("WI-331 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI332_BATCH
        for record in records
    ):
        wi332_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI332_BATCH
        ]
        expected_wi332_paths = set(WI332_REFERENCE_FILES)
        actual_wi332_paths = {
            record.get("referencePath")
            for record in wi332_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi332_paths != expected_wi332_paths:
            errors.append(
                "WI-332 batch paths do not match the pinned three-file set: "
                f"expected {sorted(expected_wi332_paths)!r}, got {sorted(actual_wi332_paths)!r}"
            )
        if len(wi332_records) != len(expected_wi332_paths):
            errors.append(
                f"WI-332 batch must contain {len(expected_wi332_paths)} records, found {len(wi332_records)}"
            )
        wi332_classifications = [record.get("classification") for record in wi332_records]
        if wi332_classifications.count("reference-only") != len(expected_wi332_paths):
            errors.append("WI-332 batch must contain three reference-only records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi332_classifications
        ):
            errors.append("WI-332 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI334_BATCH
        for record in records
    ):
        wi334_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI334_BATCH
        ]
        expected_wi334_paths = set(WI334_REFERENCE_FILES)
        actual_wi334_paths = {
            record.get("referencePath")
            for record in wi334_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi334_paths != expected_wi334_paths:
            errors.append(
                "WI-334 batch paths do not match the pinned ten-file set: "
                f"expected {sorted(expected_wi334_paths)!r}, got {sorted(actual_wi334_paths)!r}"
            )
        if len(wi334_records) != len(expected_wi334_paths):
            errors.append(
                f"WI-334 batch must contain {len(expected_wi334_paths)} records, found {len(wi334_records)}"
            )
        wi334_classifications = [record.get("classification") for record in wi334_records]
        if wi334_classifications.count("implemented-different-by-design") != len(expected_wi334_paths):
            errors.append("WI-334 batch must contain ten implemented-different-by-design records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi334_classifications
        ):
            errors.append("WI-334 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI342_BATCH
        for record in records
    ):
        wi342_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI342_BATCH
        ]
        expected_wi342_paths = set(WI342_REFERENCE_FILES)
        actual_wi342_paths = {
            record.get("referencePath")
            for record in wi342_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi342_paths != expected_wi342_paths:
            errors.append(
                "WI-342 batch paths do not match the pinned ten-file set: "
                f"expected {sorted(expected_wi342_paths)!r}, got {sorted(actual_wi342_paths)!r}"
            )
        if len(wi342_records) != len(expected_wi342_paths):
            errors.append(
                f"WI-342 batch must contain {len(expected_wi342_paths)} records, found {len(wi342_records)}"
            )
        wi342_classifications = [record.get("classification") for record in wi342_records]
        if wi342_classifications.count("implemented-different-by-design") != 8:
            errors.append("WI-342 batch must contain eight implemented-different-by-design records")
        if wi342_classifications.count("reference-only") != 2:
            errors.append("WI-342 batch must contain two reference-only records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi342_classifications
        ):
            errors.append("WI-342 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI343_BATCH
        for record in records
    ):
        wi343_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI343_BATCH
        ]
        expected_wi343_paths = set(WI343_REFERENCE_FILES)
        actual_wi343_paths = {
            record.get("referencePath")
            for record in wi343_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi343_paths != expected_wi343_paths:
            errors.append(
                "WI-343 batch paths do not match the pinned five-file set: "
                f"expected {sorted(expected_wi343_paths)!r}, got {sorted(actual_wi343_paths)!r}"
            )
        if len(wi343_records) != len(expected_wi343_paths):
            errors.append(
                f"WI-343 batch must contain {len(expected_wi343_paths)} records, found {len(wi343_records)}"
            )
        wi343_classifications = [record.get("classification") for record in wi343_records]
        if wi343_classifications.count("implemented-different-by-design") != 1:
            errors.append("WI-343 batch must contain one implemented-different-by-design record")
        if wi343_classifications.count("not-applicable") != 1:
            errors.append("WI-343 batch must contain one not-applicable record")
        if wi343_classifications.count("reference-only") != 3:
            errors.append("WI-343 batch must contain three reference-only records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi343_classifications
        ):
            errors.append("WI-343 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI344_BATCH
        for record in records
    ):
        wi344_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI344_BATCH
        ]
        expected_wi344_paths = set(WI344_REFERENCE_FILES)
        actual_wi344_paths = {
            record.get("referencePath")
            for record in wi344_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi344_paths != expected_wi344_paths:
            errors.append(
                "WI-344 batch paths do not match the pinned five-file set: "
                f"expected {sorted(expected_wi344_paths)!r}, got {sorted(actual_wi344_paths)!r}"
            )
        if len(wi344_records) != len(expected_wi344_paths):
            errors.append(
                f"WI-344 batch must contain {len(expected_wi344_paths)} records, found {len(wi344_records)}"
            )
        wi344_classifications = [record.get("classification") for record in wi344_records]
        if wi344_classifications.count("implemented-different-by-design") != 3:
            errors.append("WI-344 batch must contain three implemented-different-by-design records")
        if wi344_classifications.count("reference-only") != 2:
            errors.append("WI-344 batch must contain two reference-only records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi344_classifications
        ):
            errors.append("WI-344 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI346_BATCH
        for record in records
    ):
        wi346_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI346_BATCH
        ]
        expected_wi346_paths = set(WI346_REFERENCE_FILES)
        actual_wi346_paths = {
            record.get("referencePath")
            for record in wi346_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi346_paths != expected_wi346_paths:
            errors.append(
                "WI-346 batch paths do not match the pinned six-file set: "
                f"expected {sorted(expected_wi346_paths)!r}, got {sorted(actual_wi346_paths)!r}"
            )
        if len(wi346_records) != len(expected_wi346_paths):
            errors.append(
                f"WI-346 batch must contain {len(expected_wi346_paths)} records, found {len(wi346_records)}"
            )
        wi346_classifications = [record.get("classification") for record in wi346_records]
        if wi346_classifications.count("implemented-different-by-design") != len(expected_wi346_paths):
            errors.append("WI-346 batch must contain six implemented-different-by-design records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi346_classifications
        ):
            errors.append("WI-346 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI347_BATCH
        for record in records
    ):
        wi347_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI347_BATCH
        ]
        expected_wi347_paths = set(WI347_REFERENCE_FILES)
        actual_wi347_paths = {
            record.get("referencePath")
            for record in wi347_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi347_paths != expected_wi347_paths:
            errors.append(
                "WI-347 batch paths do not match the pinned ten-file set: "
                f"expected {sorted(expected_wi347_paths)!r}, got {sorted(actual_wi347_paths)!r}"
            )
        if len(wi347_records) != len(expected_wi347_paths):
            errors.append(
                f"WI-347 batch must contain {len(expected_wi347_paths)} records, found {len(wi347_records)}"
            )
        wi347_classifications = [record.get("classification") for record in wi347_records]
        if wi347_classifications.count("implemented-different-by-design") != len(expected_wi347_paths):
            errors.append("WI-347 batch must contain ten implemented-different-by-design records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi347_classifications
        ):
            errors.append("WI-347 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI348_BATCH
        for record in records
    ):
        wi348_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI348_BATCH
        ]
        expected_wi348_paths = set(WI348_REFERENCE_FILES)
        actual_wi348_paths = {
            record.get("referencePath")
            for record in wi348_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi348_paths != expected_wi348_paths:
            errors.append(
                "WI-348 batch paths do not match the pinned ten-file set: "
                f"expected {sorted(expected_wi348_paths)!r}, got {sorted(actual_wi348_paths)!r}"
            )
        if len(wi348_records) != len(expected_wi348_paths):
            errors.append(
                f"WI-348 batch must contain {len(expected_wi348_paths)} records, found {len(wi348_records)}"
            )
        wi348_classifications = [record.get("classification") for record in wi348_records]
        if wi348_classifications.count("implemented-different-by-design") != 7:
            errors.append("WI-348 batch must contain seven implemented-different-by-design records")
        if wi348_classifications.count("reference-only") != 3:
            errors.append("WI-348 batch must contain three reference-only records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi348_classifications
        ):
            errors.append("WI-348 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == "WI-345-reference-governance-cost-batch-15"
        for record in records
    ):
        wi345_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == "WI-345-reference-governance-cost-batch-15"
        ]
        expected_wi345_paths = set(WI345_REFERENCE_FILES)
        actual_wi345_paths = {
            record.get("referencePath")
            for record in wi345_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi345_paths != expected_wi345_paths:
            errors.append(
                "WI-345 batch paths do not match the pinned five-file set: "
                f"expected {sorted(expected_wi345_paths)!r}, got {sorted(actual_wi345_paths)!r}"
            )
        if len(wi345_records) != len(expected_wi345_paths):
            errors.append(
                f"WI-345 batch must contain {len(expected_wi345_paths)} records, found {len(wi345_records)}"
            )
        wi345_classifications = [record.get("classification") for record in wi345_records]
        if wi345_classifications.count("implemented-different-by-design") != 3:
            errors.append("WI-345 batch must contain three implemented-different-by-design records")
        if wi345_classifications.count("reference-only") != 2:
            errors.append("WI-345 batch must contain two reference-only records")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi345_classifications
        ):
            errors.append("WI-345 batch cannot leave deferred or migrate-gap records")
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
