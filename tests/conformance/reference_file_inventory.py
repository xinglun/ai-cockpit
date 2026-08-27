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
EXPECTED_TARGET_COMMIT = "a533d49dfa848d95742833f8cd1b5f7e1bb897d5"
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
        "The source lexical Capability Truth Matrix checker and claim-binding front matter are not a target Runtime gate. The target capability registry reports observed, repository-bound facts and explicit exclusions; it must not claim source matrix validation or silently promote prose to evidence. A bounded capability-claim/evidence WI-329 is required before adding such a gate.",
    ),
    "docs/reference/capability-evidence-freshness.md": (
        "reference-only",
        [
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/outcome-report.md",
            "docs/reference/verification-evidence-reuse.md",
        ],
        "The target validates Work Item verification freshness and identity-bound receipts, but has no separate Capability Truth row expiry or portable-environment matrix. The source capability-row freshness policy remains reference-only and is an explicit WI-329 candidate rather than an implied capability.",
    ),
    "docs/reference/capability-truth-matrix.json": (
        "reference-only",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "docs/capabilities.md",
            "docs/reference/reference-parity.md",
        ],
        "The source 30-row public Capability Truth Matrix is not copied. Rust `capability_truth_registry` is a request-scoped observed-capability projection with explicit adopter/external exclusions, not a public claim matrix; WI-329 owns any future strict row/evidence binding.",
    ),
    "docs/reference/capability-truth-matrix.md": (
        "reference-only",
        [
            "crates/cockpit-repository/src/lib.rs",
            "docs/capabilities.md",
            "docs/reference/reference-parity.md",
        ],
        "The source matrix documentation is retained as a reference boundary only. Current target capability and adoption pages deliberately distinguish observed Runtime facts, repository evidence, adopter installation, provider evidence, and enterprise assurance; no source matrix or claim checker is advertised until WI-329.",
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
