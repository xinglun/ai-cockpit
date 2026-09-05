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
import copy
import hashlib
import json
import subprocess
import sys
from collections import Counter
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
EXPECTED_REFERENCE_COMMIT = "fde3380f81fea5fd2e288f7a8849f737dc074060"
EXPECTED_TARGET_COMMIT = "cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd"
HISTORICAL_REFERENCE_COMMIT = "e5acb677da6621004d96f0ef353c58fe8d3acfbf"
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
WI368_BATCH = "WI-368-reference-file-comparison-batch-16"
WI411_BATCH = "WI-411-reference-java-fixture-boundary"
WI414_BATCH = "WI-414-reference-python-fixture-boundary"
WI432_BATCH = "WI-432-reference-typescript-fixture-boundary"
WI437_BATCH = "WI-437-reference-rebaseline-governance"
WI441_BATCH = "WI-441-reference-entrypoint-parity"
WI461_BATCH = GETTING_STARTED_BATCH
WI464_BATCH = "WI-464-reference-file-comparison-batch-24"
WI475_BATCH = "WI-475-reference-file-comparison-batch-25"
WI482_BATCH = "WI-482-reference-file-comparison-batch-26"
WI494_BATCH = "WI-494-reference-file-comparison-batch-27"
WI496_BATCH = "WI-496-reference-file-comparison-batch-28"
WI504_BATCH = "WI-504-reference-file-comparison-batch-29"
WI507_BATCH = "WI-507-reference-file-comparison-batch-30"
WI508_BATCH = "WI-508-reference-file-comparison-batch-31"
WI512_BATCH = "WI-512-reference-docs-batch-33"
WI516_BATCH = "WI-516-reference-file-comparison-batch-34"
WI539_BATCH = "WI-539-reference-file-comparison-batch-36"
WI543_BATCH = "WI-543-reference-file-comparison-batch-37"
WI548_BATCH = "WI-548-reference-file-comparison-batch-38"
WI550_BATCH = "WI-550-reference-file-comparison-batch-39"
WI552_BATCH = "WI-552-reference-file-comparison-batch-40"
WI557_BATCH = "WI-557-reference-file-comparison-batch-41"
WI559_BATCH = "WI-559-reference-file-comparison-batch-42"
WI563_BATCH = "WI-563-reference-file-comparison-batch-43"
WI568_BATCH = "WI-568-reference-file-comparison-batch-44"
WI572_BATCH = "WI-572-reference-installer-quality-batch-45"
WI579_BATCH = "WI-579-reference-template-parity-batch-46"
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

WI437_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    ".ai/cockpit/README.ja.md": (
        "implemented-different-by-design",
        [".ai/README.md", "docs/reference/agent-workflow.ja.md"],
        "The local reference no longer carries a source-only REPORT_LANGUAGE Make argument. The Rust target uses Runtime-owned language selection and repository-scoped context; no source Make command or byte-compatible Japanese cockpit file is required.",
    ),
    ".ai/cockpit/README.md": (
        "implemented-different-by-design",
        [".ai/README.md", "docs/reference/agent-workflow.md", "docs/reference/outcome-report.md"],
        "The local reference no longer carries a Python-template Implementation Approach section and documents its current diagnostics route. Rust keeps the evidence-bound approach and Outcome projection in typed Runtime surfaces, so source prose changes do not remove a Rust capability or justify copying the template file.",
    ),
    ".ai/cockpit/adoption.ja.md": (
        "implemented-different-by-design",
        ["docs/getting-started/README.ja.md", "docs/getting-started/adopter-configuration.ja.md"],
        "The local reference no longer carries a source Make REPORT_LANGUAGE argument. Rust onboarding uses the installed shared Runtime and localized presentation boundary, with no template-local Make command to migrate.",
    ),
    ".ai/guards/changed_critical_coverage_policy.json": (
        "implemented-different-by-design",
        ["tests/conformance/reference_file_inventory.py", "tests/ci/governance_integrity_gate.py", "crates/cockpit-repository/src/governance_controls.rs"],
        "The source guard no longer lists Python-only coverage surfaces. Rust keeps coverage and governance integrity in native tests and typed Runtime controls; the source guard JSON is not a target configuration file.",
    ),
    ".ai/guards/coverage_policy.yaml": (
        "implemented-different-by-design",
        ["tests/ci/governance_integrity_gate.py", "crates/cockpit-repository/src/governance_controls.rs", "docs/reference/ci-quality-gates.md"],
        "The source guard no longer lists obsolete Python implementation-knowledge, onboarding, status, and outcome associations. Rust coverage ownership is expressed through native tests, CI gate manifests, and Runtime controls rather than a copied YAML association registry.",
    ),
    ".ai/quality/governance-routing.yaml": (
        "implemented-different-by-design",
        [".github/workflows/ci.yml", "tests/ci/quality_route.py", "tests/ci/run_repository_gates.py", "docs/reference/ci-quality-gates.md"],
        "The local reference no longer carries duplicated per-profile depth/evidence fields and keeps routing selection separate from gate execution. Rust preserves that separation through the dynamic route and versioned gate manifest; source YAML fields are not wire requirements.",
    ),
    ".ai/schemas/task_outcome.schema.json": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-mcp/src/lib.rs", "docs/reference/outcome-report.md"],
        "The local reference simplifies its Python Task Outcome schema without template-specific handoff fields. Rust OutcomeV2 and humanHandoff are a separate typed Protocol/presentation contract required by this Runtime; the source schema is not copied and does not authorize removal of the Rust projection.",
    ),
}

WI441_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "AGENTS.md": (
        "implemented-different-by-design",
        ["AGENTS.md", ".ai/README.md", "docs/reference/agent-workflow.md", "crates/cockpit-agent/src/lib.rs"],
        "The current source keeps one Work Item/branch/PR, latest-base discovery, explicit closure, and human-controlled boundaries. The Rust target enforces those semantics through AGENTS.md, the repository route, typed lifecycle services, and generated adapters; source make commands and the source hosted-snapshot exception are not target requirements.",
    ),
    "GEMINI.md": (
        "implemented-different-by-design",
        [".ai/README.md", "crates/cockpit-agent/src/lib.rs", "crates/cockpit-agent/tests/install.rs", "docs/reference/agent-workflow.md"],
        "The source Gemini rules are provider-facing Contract-first, Summary, checkpoint, and fail-closed guidance. The target does not commit a provider-specific GEMINI.md; explicit agent install can generate an owned Gemini adapter from the shared repository route, so absence from this repository is not an omitted global configuration.",
    ),
    "docs/README.md": (
        "implemented-different-by-design",
        ["docs/README.md", "docs/current/README.md", "docs/getting-started/README.md", "docs/reference/README.md"],
        "The current source provides a short reader-first North Star and goal route. The target preserves that route with a richer Rust-specific current/getting-started/operations/reference map, explicit Runtime/repository boundaries, and tri-language links; source page structure and claims are not copied byte-for-byte.",
    ),
    "docs/README.zh-CN.md": (
        "implemented-different-by-design",
        ["docs/README.zh-CN.md", "docs/current/README.zh-CN.md", "docs/getting-started/README.zh-CN.md", "docs/reference/README.zh-CN.md"],
        "The Chinese reader route preserves the source's four-question and goal-first intent through the target's explicit current, getting-started, operations, and reference pages, with Rust Runtime and adopter boundaries kept visible.",
    ),
    "docs/README.ja.md": (
        "implemented-different-by-design",
        ["docs/README.ja.md", "docs/current/README.ja.md", "docs/getting-started/README.ja.md", "docs/reference/README.ja.md"],
        "The Japanese reader route preserves the source's four-question and goal-first intent through the target's explicit current, getting-started, operations, and reference pages; source-specific page bytes are not copied.",
    ),
    "docs/capabilities.md": (
        "implemented-different-by-design",
        ["docs/capabilities.md", "docs/reference/capability-truth-matrix.md", "docs/reference/commands.md"],
        "The current source capability page defines the Repository Governance Layer and its external non-claims. The target keeps those boundaries and adds concrete Rust CLI/MCP, scaffold, profile, knowledge, Outcome, and isolation paths without converting source manifest statuses into Runtime authority.",
    ),
    "docs/capabilities.zh-CN.md": (
        "implemented-different-by-design",
        ["docs/capabilities.zh-CN.md", "docs/reference/capability-truth-matrix.md", "docs/reference/commands.zh-CN.md"],
        "The Chinese target page preserves the source capability and external-responsibility boundary while documenting the repository-bound Rust Runtime and adopter inheritance path in Chinese; it is not a source wire or status manifest copy.",
    ),
    "docs/capabilities.ja.md": (
        "implemented-different-by-design",
        ["docs/capabilities.ja.md", "docs/reference/capability-truth-matrix.md", "docs/reference/commands.ja.md"],
        "The Japanese target page preserves the source capability and external-responsibility boundary while documenting the repository-bound Rust Runtime and adopter inheritance path in Japanese; source status bytes are not copied.",
    ),
    "docs/features/task-outcome-report.md": (
        "implemented-different-by-design",
        ["docs/features/task-outcome-report.md", "docs/reference/outcome-report.md", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "crates/cockpit-mcp/src/lib.rs"],
        "The source Task Outcome page defines an evidence-backed report separate from status and PR presentation. The target preserves that separation through OutcomeV2, CLI/MCP human handoff, immutable evidence, and repository-bound lifecycle validation; source report wire shape and make commands are not copied.",
    ),
}

# WI-461 is a narrow rebaseline of the nine onboarding pages whose pinned
# source bytes changed after the earlier getting-started comparison.  Keep the
# prior source-change metadata on each record, but replace the deferred status
# only after this batch has re-read the current source and the Rust reader
# route.  The source commands and wire shape remain non-portable; these are
# semantic/documentation decisions, not template-file copies.
WI461_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/getting-started/first-work-item.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/first-work-item.md",
            "docs/reference/agent-workflow.md",
            "docs/reference/outcome-report.md",
        ],
        "The current source removes its obsolete REPORT_LANGUAGE Make argument. The Rust page preserves the complete start-to-close lifecycle, visible human Outcome, explicit repository binding, and review stop boundary with native CLI commands; source Make syntax and bytes are not copied.",
    ),
    "docs/getting-started/first-work-item.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/first-work-item.zh-CN.md",
            "docs/reference/agent-workflow.zh-CN.md",
            "docs/reference/outcome-report.zh-CN.md",
        ],
        "源文件仅移除了过时的 REPORT_LANGUAGE Make 参数。Rust 中文页面保留完整的 start 到 close 生命周期、可见的人类 Outcome、显式仓库绑定和人工 review 停止边界，并使用 Rust CLI；不复制源 Make 语法或文件字节。",
    ),
    "docs/getting-started/first-work-item.ja.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/first-work-item.ja.md",
            "docs/reference/agent-workflow.ja.md",
            "docs/reference/outcome-report.ja.md",
        ],
        "現行 source は obsolete な REPORT_LANGUAGE Make argument を削除しました。Rust 日本語ページは start から close までの lifecycle、visible human Outcome、明示的な repository binding、review 停止境界を native CLI で保持し、source の Make syntax と bytes はコピーしません。",
    ),
    "docs/getting-started/security-release-verification.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/security-release-verification.md",
            "docs/release/distribution.md",
            "docs/getting-started/installation-security.md",
        ],
        "The current source narrows the release route to release-digests.json and removes the obsolete release.json dual-asset paragraph. The Rust pages express the same digest, tag, SBOM, provenance, provider-responsibility, and adopter-isolation boundaries through its release manifest/SHA256SUMS route; source release projections are not copied.",
    ),
    "docs/getting-started/security-release-verification.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/security-release-verification.zh-CN.md",
            "docs/release/distribution.zh-CN.md",
            "docs/getting-started/installation-security.zh-CN.md",
        ],
        "源文件将发布路径收窄为 release-digests.json，并删除旧的 release.json 双资产说明。Rust 中文文档通过 release manifest/SHA256SUMS 路径表达相同的 digest、tag、SBOM、provenance、provider 责任和 adopter 隔离边界；不复制源投影文件。",
    ),
    "docs/getting-started/security-release-verification.ja.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/security-release-verification.ja.md",
            "docs/release/distribution.ja.md",
            "docs/getting-started/installation-security.ja.md",
        ],
        "現行 source は release route を release-digests.json に絞り、obsolete な release.json dual-asset 説明を削除しました。Rust 日本語文書は release manifest/SHA256SUMS route で digest、tag、SBOM、provenance、provider responsibility、adopter isolation の境界を示し、source projection file はコピーしません。",
    ),
    "docs/getting-started/standard-adoption-guide.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/standard-adoption-guide.md",
            "docs/getting-started/installation.md",
            "docs/getting-started/first-work-item.md",
        ],
        "The current source removes its obsolete REPORT_LANGUAGE Make argument. The Rust guide keeps the reader-first install, attach, calibration, adapter, Work Item, Outcome, merge, cleanup, and close route while using the shared repository-bound Runtime rather than source Make workflow bytes.",
    ),
    "docs/getting-started/standard-adoption-guide.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/standard-adoption-guide.zh-CN.md",
            "docs/getting-started/installation.zh-CN.md",
            "docs/getting-started/first-work-item.zh-CN.md",
        ],
        "源文件仅移除了过时的 REPORT_LANGUAGE Make 参数。Rust 中文指南保留面向读者的 install、attach、calibration、adapter、Work Item、Outcome、merge、cleanup 和 close 路径，并使用共享的仓库绑定 Runtime；不复制源 Make 工作流字节。",
    ),
    "docs/getting-started/standard-adoption-guide.ja.md": (
        "implemented-different-by-design",
        [
            "docs/getting-started/standard-adoption-guide.ja.md",
            "docs/getting-started/installation.ja.md",
            "docs/getting-started/first-work-item.ja.md",
        ],
        "現行 source は obsolete な REPORT_LANGUAGE Make argument を削除しました。Rust 日本語 guide は reader-first の install、attach、calibration、adapter、Work Item、Outcome、merge、cleanup、close route を shared repository-bound Runtime で保持し、source Make workflow bytes はコピーしません。",
    ),
}

# WI-464 re-reads the four workflow/build paths changed in the maintained
# local reference after the earlier WI-302/WI-304 decisions.  The source
# remains a Python/Make provider surface; the Rust target keeps its own
# explicit CI/release/adopter boundaries and must not copy source bytes.
WI464_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
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
        "The current source changes its ShellCheck installation and Rust action pin inside a Python/multi-stack compatibility matrix. Rust keeps a pinned-action policy, dynamic quality route, Rust workspace/platform gates, and immutable public adopter acceptance; source install.sh, Python lanes, and multi-stack matrix behavior remain adopter/provider boundaries rather than copied workflow bytes.",
    ),
    ".github/workflows/release.yml": (
        "implemented-different-by-design",
        [
            ".github/workflows/release.yml",
            "tests/release/workflow_policy.sh",
            "tests/release/version_consistency.sh",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
            "docs/release/distribution.md",
        ],
        "The current source binds a source-side release-digests projection into its Python archive and removes the obsolete release.json dual-asset check. Rust expresses the same immutable tag, archive, checksum, SBOM/provenance, platform smoke, and adopter-isolation responsibilities through release-manifest/SHA256SUMS and Rust-native release tools; source release.json/release-digests bytes are not copied.",
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
        "The current source removes a REPORT_LANGUAGE Make argument from its Python smoke route. Rust has no source smoke.yml or Make bridge; CI, release, gate-manifest, and immutable adopter harnesses provide the corresponding repository-bound checks with language-aware human projection and explicit --repo context.",
    ),
    "Makefile": (
        "implemented-different-by-design",
        [
            ".github/workflows/ci.yml",
            "tests/ci/run_repository_gates.py",
            "docs/reference/commands.md",
            "Cargo.toml",
        ],
        "The current source removes Python/Make shard, knowledge, and REPORT_LANGUAGE helpers. The Rust target intentionally has no Makefile; Cargo, the Rust CLI, the canonical gate manifest, and explicit repository-bound commands provide the maintained interface, while source Python orchestration and generated knowledge helpers remain source-only.",
    ),
}

# WI-475 re-reads the maintained reference's human-facing Outcome, append-only
# event, and quality-gate operations pages.  The Rust target intentionally
# places these responsibilities under its canonical reference/features pages
# and typed Runtime/gate surfaces; source Python/Make/provider files are not
# copied or treated as wire compatibility requirements.
WI475_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/features/human-benefit-report.md": (
        "implemented-different-by-design",
        [
            "docs/features/human-benefit-report.md",
            "docs/features/task-outcome-report.md",
            "docs/reference/outcome-report.md",
            "docs/reference/task-outcome-events.md",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
            "crates/cockpit-mcp/tests/rpc.rs",
        ],
        "The source report's deterministic human projection, evidence-count semantics, archive ownership, and explicit non-claims are represented by Rust OutcomeV2/humanHandoff, task-outcome references, and CLI/MCP regressions. Source ai-finish/check-ai-pr reports and Python/Make paths remain source/provider surfaces rather than copied target files.",
    ),
    "docs/features/human-benefit-report.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/features/human-benefit-report.zh-CN.md",
            "docs/features/task-outcome-report.zh-CN.md",
            "docs/reference/outcome-report.zh-CN.md",
            "docs/reference/task-outcome-events.zh-CN.md",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
            "crates/cockpit-mcp/tests/rpc.rs",
        ],
        "中文页面保留 source 的确定性人类投影、evidence 计数语义、归档归属和非声明边界；Rust 通过 OutcomeV2/humanHandoff、Task Outcome 参考和 CLI/MCP 回归承载。源 ai-finish/check-ai-pr 报告及 Python/Make 路径是 source/provider 边界，不复制为目标文件。",
    ),
    "docs/features/human-benefit-report.ja.md": (
        "implemented-different-by-design",
        [
            "docs/features/human-benefit-report.ja.md",
            "docs/features/task-outcome-report.ja.md",
            "docs/reference/outcome-report.ja.md",
            "docs/reference/task-outcome-events.ja.md",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
            "crates/cockpit-mcp/tests/rpc.rs",
        ],
        "source の deterministic な human projection、evidence count、archive ownership、non-claim 境界を Rust の OutcomeV2/humanHandoff、Task Outcome reference、CLI/MCP regression で表します。source の ai-finish/check-ai-pr report と Python/Make path は source/provider boundary であり、target file として copy しません。",
    ),
    "docs/maintainers/task-outcome-events.md": (
        "implemented-different-by-design",
        [
            "docs/reference/task-outcome-events.md",
            "docs/reference/task-outcome-events.zh-CN.md",
            "docs/reference/task-outcome-events.ja.md",
            "docs/features/task-outcome-report.md",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/task_outcome_events.rs",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
        ],
        "The source append-only event, correction, fingerprint, relationship, privacy, and provider-evidence responsibilities are implemented by the strict Rust event stream and tri-language references. Source generator/validator/renderer scripts remain semantic material; no Python schema or Make target is copied.",
    ),
    "docs/operations/quality-gates.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-quality-gates.md",
            "docs/reference/governance-integrity-gate.md",
            "tests/ci/repository_gate_manifest.json",
            "tests/ci/run_repository_gates.py",
            ".github/workflows/ci.yml",
            "docs/release/distribution.md",
        ],
        "The source quality hierarchy, dynamic routing, shadow comparison, shard ownership, timing/JUnit/coverage evidence, timeout, performance-sample, and traceability responsibilities are represented by Rust Contract gates plus the reviewed gate manifest and CI/release surfaces. Source make quality, Makefile.ai.stack, and Python runner bytes remain adopter/provider boundaries and are not copied.",
    ),
    "docs/operations/quality-gates.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-quality-gates.zh-CN.md",
            "docs/reference/governance-integrity-gate.zh-CN.md",
            "tests/ci/repository_gate_manifest.json",
            "tests/ci/run_repository_gates.py",
            ".github/workflows/ci.yml",
            "docs/release/distribution.zh-CN.md",
        ],
        "中文责任由 Rust Contract gate、审核过的 gate manifest、CI/release 页面承载：保留 source 的质量层级、动态路由、shadow 对照、分片责任、计时/JUnit/coverage、超时、性能样本和可追溯性。源 make quality、Makefile.ai.stack 和 Python runner bytes 属于 adopter/provider 边界，不复制。",
    ),
    "docs/operations/quality-gates.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-quality-gates.ja.md",
            "docs/reference/governance-integrity-gate.ja.md",
            "tests/ci/repository_gate_manifest.json",
            "tests/ci/run_repository_gates.py",
            ".github/workflows/ci.yml",
            "docs/release/distribution.ja.md",
        ],
        "source の quality hierarchy、dynamic route、shadow 比較、shard ownership、timing/JUnit/coverage evidence、timeout、performance sample、traceability を Rust Contract gate、reviewed gate manifest、CI/release reference で表します。source の make quality、Makefile.ai.stack、Python runner bytes は adopter/provider boundary として copy しません。",
    ),
}

# WI-482 re-reads the eight maintained-reference documentation paths whose
# source bytes changed after the previous parity decision.  The reference
# narrowed its operations page, moved parallel/handoff detail out of the
# short lifecycle route, removed a template-only quality-shard section, and
# removed the obsolete REPORT_LANGUAGE argument.  The Rust target keeps the
# corresponding semantics in its own reader route, Runtime lifecycle,
# parallelism references, and trust/enterprise pages; it does not copy the
# source files or Make workflow.
WI482_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/operations/work-item-lifecycle.md": (
        "implemented-different-by-design",
        [
            "docs/reference/agent-workflow.md",
            "docs/reference/outcome-report.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "The maintained source shortens this operations page to the canonical serial lifecycle and moves parallel guidance elsewhere. The Rust reader route keeps the same lifecycle, human pause, exact cleanup, and separate parallel/reference boundaries in its Runtime-native documentation; source page bytes and make commands are not copied.",
    ),
    "docs/operations/work-item-lifecycle.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/agent-workflow.zh-CN.md",
            "docs/reference/outcome-report.zh-CN.md",
            "docs/reference/reference-file-comparison.zh-CN.md",
        ],
        "源中文 operations 页面收敛为标准串行生命周期，并把并行说明移到其他路线。Rust 中文读者路线在 Runtime 原生文档中保留相同生命周期、人工暂停、精确清理和独立并行边界；不复制源文件字节或 Make 命令。",
    ),
    "docs/operations/work-item-lifecycle.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/agent-workflow.ja.md",
            "docs/reference/outcome-report.ja.md",
            "docs/reference/reference-file-comparison.ja.md",
        ],
        "現行 source の operations page は標準の serial lifecycle に絞り、parallel の説明を別 route に移しました。Rust の reader route は Runtime-native な文書で同じ lifecycle、human pause、正確な cleanup、独立した parallel 境界を保持し、source bytes と Make command はコピーしません。",
    ),
    "docs/reference/agent-parallel-work-items.md": (
        "implemented-different-by-design",
        [
            "docs/reference/cross-work-item-dedup.md",
            "docs/reference/affected-verification.md",
            "docs/reference/agent-workflow.md",
            "AGENTS.md",
            ".ai/README.md",
        ],
        "The source removes its conversation-handoff appendix because that boundary is owned by the Agent/host route. Rust keeps bounded parallel planning, serialized projections, one-Work-Item identity, and the mandatory visible Outcome handoff in repository rules and dedicated references; the source document is not copied.",
    ),
    "docs/reference/ai-cockpit-work-item-lifecycle.md": (
        "implemented-different-by-design",
        [
            "docs/reference/agent-workflow.md",
            "docs/reference/outcome-report.md",
            "docs/reference/ci-quality-gates.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source no longer includes its template-internal quality-shard worktree section or duplicate retry-repair section, and drops REPORT_LANGUAGE from the finish example. Rust retains the applicable lifecycle, quality routing, retry boundary, and localized presentation through its installed Runtime and typed references; template Make/Python helper behavior is explicitly not an adopter requirement.",
    ),
    "docs/trust-layer.md": (
        "implemented-different-by-design",
        [
            "docs/philosophy.md",
            "docs/security/enterprise-governance.md",
            "docs/architecture.md",
            "docs/capabilities.md",
        ],
        "The source Trust Layer page is a consolidated Python/Make-oriented explanation. Rust deliberately projects its Why/What/How, evidence, human decision, enterprise boundary, and non-claims across philosophy, architecture, capability, and enterprise-governance pages; no same-path Trust Layer or source Make command is required.",
    ),
    "docs/trust-layer.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/philosophy.zh-CN.md",
            "docs/security/enterprise-governance.zh-CN.md",
            "docs/architecture.zh-CN.md",
            "docs/capabilities.zh-CN.md",
        ],
        "源中文 Trust Layer 将 Python/Make 说明集中在单页；Rust 中文文档按设计拆分到设计思想、架构、能力真值和企业治理边界，保留 Why/What/How、证据、人类决定和非声明语义，不要求同路径页面或源 Make 命令。",
    ),
    "docs/trust-layer.ja.md": (
        "implemented-different-by-design",
        [
            "docs/philosophy.ja.md",
            "docs/security/enterprise-governance.ja.md",
            "docs/architecture.ja.md",
            "docs/capabilities.ja.md",
        ],
        "source の Trust Layer は Python/Make の説明を一つの page に集約します。Rust は設計上、design philosophy、architecture、capability truth、enterprise governance に Why/What/How、evidence、human decision、non-claim を分けて投影し、同じ path の page や source Make command は要求しません。",
    ),
}

# WI-494 re-reads the seven maintained reference records whose source bytes
# changed after their earlier reference-only decisions.  These are
# source-bound capability, comprehension-study, and cleanup-registry records;
# none is a Rust Runtime wire contract or an adopter artifact.  The target
# keeps the applicable boundaries in its own typed capability, documentation,
# and lifecycle surfaces without copying participant data or source tooling.
WI494_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/capability-truth-matrix.json": (
        "reference-only",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "docs/capabilities.md",
            "docs/reference/reference-parity.md",
        ],
        "This source-owned capability claim matrix records the template's implementation status, freshness, and evidence claims. Rust exposes request-scoped, repository-bound capability truth through typed Runtime projections; source capability claims and adopter/provider status are not copied into the target ledger or protocol.",
    ),
    "docs/reference/comprehension-validation-responses/peter_01.en.json": (
        "reference-only",
        [
            "docs/README.md",
            "docs/features/human-benefit-report.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "This is an anonymized, source-bound English participant response for a comprehension study at one document revision. It is preserved as reference evidence only; Rust does not import participant responses or claim that this study proves adopter, release, safety, or enterprise readiness.",
    ),
    "docs/reference/comprehension-validation-responses/tanaka_01.ja.json": (
        "reference-only",
        [
            "docs/README.ja.md",
            "docs/features/human-benefit-report.ja.md",
            "docs/reference/reference-file-comparison.ja.md",
        ],
        "これは特定の document revision に対する source-bound な日本語 participant response です。Rust は participant data や comprehension claim を取り込まず、reader-facing documentation と Outcome の境界だけを Rust-native に保持します。adopter、release、safety、enterprise readiness の証明ではありません。",
    ),
    "docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json": (
        "reference-only",
        [
            "docs/README.zh-CN.md",
            "docs/features/human-benefit-report.zh-CN.md",
            "docs/reference/reference-file-comparison.zh-CN.md",
        ],
        "这是绑定到特定文档修订版的中文匿名参与者研究记录。Rust 不导入参与者数据或理解度声明，只在自己的读者文档与 Outcome 路线上保留相应边界；它不证明 adopter、发布、安全或企业准备度。",
    ),
    "docs/reference/comprehension-validation-results.json": (
        "reference-only",
        [
            "docs/features/human-benefit-report.md",
            "docs/features/human-benefit-report.zh-CN.md",
            "docs/features/human-benefit-report.ja.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "This source result is a narrow, revision-bound comprehension receipt with one reader per required locale. It is reference-study evidence only and cannot be transferred to the Rust Runtime as a product, release, safety, security, or enterprise claim.",
    ),
    "docs/reference/comprehension-validation-results.md": (
        "reference-only",
        [
            "docs/features/human-benefit-report.md",
            "docs/reference/outcome-report.md",
            "docs/reference/reference-file-comparison.md",
        ],
        "The maintained source report explains a revision-bound comprehension result and its limitations. Rust keeps the reader-facing Outcome and human-benefit boundaries in its own documentation; it does not copy the source study report or inherit its claim.",
    ),
    "docs/reference/deprecated-assets-registry.json": (
        "reference-only",
        [
            ".ai/README.md",
            "docs/reference/agent-workflow.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "This source cleanup registry identifies source-specific deprecated assets and prohibited command chains. Rust uses immutable Work Item history, explicit resource finalization, and reviewed cleanup receipts; the source scanner/registry is not a Runtime deletion authority and is not copied.",
    ),
}

# WI-496 re-reads the next ten maintained reference paths: release
# distribution/profile explanations, a source-only context registry, and
# revision-bound Japanese/pre-release assessment reports.  Runtime and
# adopter responsibilities are mapped to Rust-native surfaces; source
# Python/Make bytes and source-specific receipts remain non-portable.
WI496_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/distribution.ja.md": (
        "implemented-different-by-design",
        [
            "docs/release/distribution.ja.md",
            "docs/architecture/release-distribution.ja.md",
            "docs/getting-started/installation.ja.md",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
        ],
        "Source distribution guidance is projected into the Rust release, installation, checksum/SBOM/provenance, and adopter-acceptance surfaces with explicit repository/runtime boundaries. Source Make/Python runner bytes, source venv provisioning, and provider release state are not copied.",
    ),
    "docs/reference/distribution.md": (
        "implemented-different-by-design",
        [
            "docs/release/distribution.md",
            "docs/architecture/release-distribution.md",
            "docs/getting-started/installation.md",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
        ],
        "The source distribution contract is preserved through the Rust-native immutable release/archive, checksum, SBOM, provenance, installer, and adopter-acceptance routes. Source Make/Python orchestration, worktree-local venvs, and provider-owned release decisions are deliberately not copied.",
    ),
    "docs/reference/documentation-context-registry.json": (
        "reference-only",
        [
            ".ai/README.md",
            ".ai/glossary.md",
            "AGENTS.md",
            "docs/reference/README.md",
            "docs/reference/instruction-traceability.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "This registry is source-maintenance context metadata for plan/report paths, mutability, and historical instructions; it is not a portable Runtime or adopter protocol. Rust keeps explicit current-instruction, historical-reference, and documentation-authority boundaries without copying source planning records.",
    ),
    "docs/reference/governance-profiles.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/governance-profiles.ja.md",
            "docs/reference/ci-quality-gates.ja.md",
            "tests/ci/quality_route.py",
            "tests/ci/run_repository_gates.py",
            "crates/cockpit-repository/src/governance_controls.rs",
        ],
        "Source profile guidance is implemented through the Rust typed dynamic quality route and repository-bound gate manifest. Verification tier and evidence assurance remain orthogonal, and source YAML/Make profile bytes are not copied into adopters.",
    ),
    "docs/reference/governance-profiles.md": (
        "implemented-different-by-design",
        [
            "docs/reference/governance-profiles.md",
            "docs/reference/ci-quality-gates.md",
            "tests/ci/quality_route.py",
            "tests/ci/run_repository_gates.py",
            "crates/cockpit-repository/src/governance_controls.rs",
        ],
        "The source light/standard/strict profile semantics are projected into the Rust dynamic route, reviewed gate manifest, and typed governance controls. VerificationTier and EvidenceAssurance are separate dimensions; source YAML/Make configuration and provider execution remain outside the target protocol.",
    ),
    "docs/reference/governance-profiles.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/governance-profiles.zh-CN.md",
            "docs/reference/ci-quality-gates.zh-CN.md",
            "tests/ci/quality_route.py",
            "tests/ci/run_repository_gates.py",
            "crates/cockpit-repository/src/governance_controls.rs",
        ],
        "源端 light/standard/strict profile 语义由 Rust 动态路由、审核过的 gate manifest 和类型化治理控制承载。VerificationTier 与 EvidenceAssurance 保持正交；源 YAML/Make 配置及 provider 执行责任不复制到目标协议。",
    ),
    "docs/reference/japanese-capability-assessment.json": (
        "reference-only",
        [
            "docs/reference/japanese-capability-assessment.md",
            "docs/reference/japanese-capability-assessment.zh-CN.md",
            "docs/reference/japanese-capability-assessment.ja.md",
            "tests/docs/documentation_acceptance.sh",
            "crates/cockpit-cli/tests/intelligence.rs",
        ],
        "The source JSON is a revision-bound 58-file Japanese assessment and receipt for the reference repository. Its participant/source digests and release result cannot become Rust or adopter proof; the target maintains its own bounded multilingual documentation and executable evidence instead.",
    ),
    "docs/reference/japanese-capability-assessment.md": (
        "implemented-different-by-design",
        [
            "docs/reference/japanese-capability-assessment.md",
            "docs/reference/japanese-capability-assessment.zh-CN.md",
            "docs/reference/japanese-capability-assessment.ja.md",
            "docs/reference/outcome-report.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The source assessment explanation is retained as an evidence-bound multilingual boundary in the Rust tri-language reader route. The target does not inherit the source corpus/assessment digest or claim general fluency, provider behavior, or translated Contract facts.",
    ),
    "docs/reference/pre-release-documentation-alignment.json": (
        "reference-only",
        [
            "tests/docs/documentation_acceptance.sh",
            "tests/docs/parity_status_check.sh",
            "tests/ci/governance_integrity_gate.py",
            "docs/reference/reference-file-comparison.md",
            "docs/reference/japanese-capability-assessment.md",
        ],
        "This source JSON is a revision/work-item-bound pre-release audit receipt for the template's documentation surfaces. Rust performs its own fresh documentation, governance, parity, and multilingual checks; the source receipt and its status are not portable evidence.",
    ),
    "docs/reference/pre-release-documentation-alignment.md": (
        "reference-only",
        [
            "tests/docs/documentation_acceptance.sh",
            "tests/docs/parity_status_check.sh",
            "tests/ci/governance_integrity_gate.py",
            "docs/reference/reference-file-comparison.md",
            "docs/reference/ci-quality-gates.md",
        ],
        "The source report summarizes a revision-bound pre-release alignment audit. Target documentation and gates provide analogous responsibilities with new Runtime evidence; the source report, source Work Item, and source release claim are not copied or inherited.",
    ),
}

# WI-504 re-reads five source documentation changes at the pinned local
# reference commit.  The first four are provider/source-maintenance guidance
# whose portable semantics already live in the Rust-native reader routes.  The
# root upgrade entry is restored as a bounded compatibility pointer so the
# source reader path does not become a dead link in the target.
WI504_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/repository-workflow.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/repository-workflow.ja.md",
            "docs/reference/repository-workflow.md",
            "docs/reference/repository-workflow.zh-CN.md",
            ".ai/README.md",
        ],
        "The source change removes a provider-specific REPORT_LANGUAGE argument. The Rust Japanese workflow already uses localized Runtime presentation without that argument and keeps explicit repository-scoped lifecycle, evidence, review, and cleanup boundaries; source Make commands are not copied.",
    ),
    "docs/reference/troubleshooting.md": (
        "implemented-different-by-design",
        [
            "docs/reference/troubleshooting.md",
            "docs/reference/troubleshooting.zh-CN.md",
            "docs/reference/troubleshooting.ja.md",
            "docs/reference/commands.md",
        ],
        "The source removes a provider-specific external-handoff troubleshooting note. Rust keeps the general stop/recovery contract, explicit --repo binding, and evidence-preservation rules in its tri-language route; provider handoff records remain external and are not copied.",
    ),
    "docs/reference/verification-evidence-reuse.md": (
        "implemented-different-by-design",
        [
            "docs/reference/verification-evidence-reuse.md",
            "docs/reference/verification-evidence-reuse.zh-CN.md",
            "docs/reference/verification-evidence-reuse.ja.md",
            "docs/reference/verification-route.md",
            "crates/cockpit-verification/src/lib.rs",
        ],
        "The source makes a no-change decision for its Python/Make reuse proposal. Rust independently provides bounded, identity-bound, fail-closed reuse under a separate authorized Runtime boundary; this is not source wire or implementation parity and source receipts are not imported.",
    ),
    "docs/reference/work-item-lifecycle-closure.md": (
        "implemented-different-by-design",
        [
            "docs/reference/work-item-lifecycle-closure.md",
            "docs/reference/repository-workflow.md",
            "docs/reference/agent-workflow.md",
            "docs/reference/governance-integrity-gate.md",
        ],
        "The source removes a provider-specific hosted-governance recovery section and a REPORT_LANGUAGE argument. Rust retains the portable closure, exact cleanup, recovery, and evidence rules across linked Rust-native routes; source Make/Python/provider recovery commands are not Runtime requirements.",
    ),
    "docs/upgrade.md": (
        "implemented-different-by-design",
        [
            "docs/upgrade.md",
            "docs/reference/upgrade.md",
            "docs/reference/upgrade.zh-CN.md",
            "docs/reference/upgrade.ja.md",
        ],
        "The source file is a compatibility reader entry. Rust restores the same navigation role with a minimal root pointer to the canonical tri-language reference upgrade route, while keeping shared Runtime installation, repository migration, and provider configuration as separate boundaries.",
    ),
}

# WI-507 compares the first five maintained language-adaptation example
# readers after the documentation batch. These are source/provider-specific
# onboarding examples, not Runtime code or a portable wire contract. The
# target records their governance boundary as reference-only and points to
# existing Rust-native contract, verification, and adopter routes without
# copying any SDK, Make, installer, or application example.
WI507_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "examples/flutter/README.md": (
        "reference-only",
        [
            "docs/reference/flutter-fixture-adaptation.md",
            "docs/reference/flutter-fixture-adaptation.zh-CN.md",
            "docs/reference/flutter-fixture-adaptation.ja.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This Flutter adaptation example is source/provider onboarding material. The target preserves explicit scope, owner-approved commands, evidence binding, and shared Runtime/adopter isolation through Rust-native routes, but does not copy Flutter/Dart installation, Make presets, coverage YAML, application code, or source JSON wire shape.",
    ),
    "examples/go/README.md": (
        "reference-only",
        [
            "docs/getting-started/adopter-configuration.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This Go adaptation example is source/provider onboarding material. Go toolchain commands, Make presets, coverage patterns, and application examples remain adopter-owned; the target keeps only the general Contract, verification, evidence, and explicit repository-context boundaries.",
    ),
    "examples/java/README.md": (
        "reference-only",
        [
            "docs/getting-started/examples/java.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This Java adaptation example is source/provider onboarding material. Gradle/Spring/Android commands, coverage presets, and example application code are not Runtime requirements; the target preserves owner-declared scope, verification, evidence, and repository isolation through its Rust-native routes.",
    ),
    "examples/kotlin/README.md": (
        "reference-only",
        [
            "docs/getting-started/adopter-configuration.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This Kotlin adaptation example is source/provider onboarding material. Kotlin/Gradle commands and coverage patterns remain adopter/provider responsibilities; the target does not copy the source installer, Make bridge, SDK assumptions, or JSON contract example and retains only the generic governance boundary.",
    ),
    "examples/php/README.md": (
        "reference-only",
        [
            "docs/getting-started/adopter-configuration.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This PHP adaptation example is source/provider onboarding material. Composer, PHPUnit, PHPStan, Make presets, and application paths remain adopter/provider responsibilities; the target preserves explicit Contract scope, verification, evidence, and shared-Runtime isolation without copying source implementation or wire shape.",
    ),
}

# WI-508 compares the next five maintained stack-adaptation example readers.
# They remain source/provider onboarding material rather than Runtime code or
# a portable wire contract.  Keep each decision explicit so a future adopter
# cannot accidentally inherit a source installer, Make preset, toolchain, or
# sample Contract decision.
WI508_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "examples/python/README.md": (
        "reference-only",
        [
            "docs/reference/python-fixture-adaptation.md",
            "docs/reference/python-fixture-adaptation.zh-CN.md",
            "docs/reference/python-fixture-adaptation.ja.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This Python adaptation example is source/provider onboarding material. Python installer commands, Make presets, coverage YAML, and sample Contract/Summary decisions remain adopter-owned; Rust preserves only the generic Contract, verification, evidence, and shared-Runtime isolation boundary.",
    ),
    "examples/ruby/README.md": (
        "reference-only",
        [
            "docs/getting-started/adopter-configuration.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This Ruby adaptation example is source/provider onboarding material. Bundler/RuboCop/RSpec or Rake commands, coverage patterns, and application examples remain adopter/provider responsibilities; no source installer, Make preset, or sample Contract wire shape is copied.",
    ),
    "examples/rust/README.md": (
        "reference-only",
        [
            "docs/getting-started/adopter-configuration.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
            "docs/reference/ci-quality-gates.md",
        ],
        "This Rust adaptation example is source/provider onboarding material. Cargo commands, inline-test coverage caveats, Make presets, and sample Contract/Summary decisions are project-owned configuration; the target's own Rust Runtime and generic verification routes provide the portable governance boundary without copying the source example.",
    ),
    "examples/swift/README.md": (
        "reference-only",
        [
            "docs/reference/ios-swift-fixture-adaptation.md",
            "docs/reference/ios-swift-fixture-adaptation.zh-CN.md",
            "docs/reference/ios-swift-fixture-adaptation.ja.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This Swift adaptation example is source/provider onboarding material. SwiftPM/Xcode commands, Coverage Guard patterns, signing/platform assumptions, and sample Contract/Summary decisions remain adopter/provider responsibilities; the target preserves explicit calibration, evidence, and repository isolation without copying the source installer or application example.",
    ),
    "examples/typescript/README.md": (
        "reference-only",
        [
            "docs/reference/typescript-fixture-adaptation.md",
            "docs/reference/typescript-fixture-adaptation.zh-CN.md",
            "docs/reference/typescript-fixture-adaptation.ja.md",
            "docs/reference/contract-fields.md",
            "docs/reference/verification-route.md",
        ],
        "This TypeScript adaptation example is source/provider onboarding material. npm scripts, Node dependencies, coverage patterns, lifecycle fixture behavior, and sample Contract/Summary decisions remain adopter/provider responsibilities; Rust preserves explicit commands, evidence binding, shared Runtime isolation, and human review without copying source wire or toolchain files.",
    ),
}

# WI-512 compares the maintained governance/reference pages one by one.  The
# source checkout contains English and Japanese pages for some topics and a
# three-language weakening-guard set; the Rust target deliberately records
# every tri-language counterpart while keeping source-only commands and wire
# formats out of the Runtime contract.
WI512_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/schemas.md": (
        "implemented-different-by-design",
        [
            "docs/reference/schemas.md",
            "docs/reference/schemas.zh-CN.md",
            "docs/reference/schemas.ja.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/tests/contract_schema.rs",
        ],
        "The source schema map is represented by Rust typed records and repository validators with explicit identity, legacy, and non-wire boundaries; source YAML registries and Python validators are comparison material, not copied Runtime requirements.",
    ),
    "docs/reference/test-architecture.md": (
        "implemented-different-by-design",
        [
            "docs/reference/test-architecture.md",
            "docs/reference/test-architecture.zh-CN.md",
            "docs/reference/test-architecture.ja.md",
            "docs/reference/ci-quality-gates.md",
            "tests/ci/quality_route.py",
            "tests/ci/governance_integrity_gate.py",
        ],
        "The source layered quality responsibilities are preserved by Rust workspace, repository, adversarial, release, and documentation gates plus the dynamic quality route. Verification tier is not Evidence Assurance, and source Make/Python commands are not copied.",
    ),
    "docs/reference/test-weakening-guard.md": (
        "implemented-different-by-design",
        [
            "docs/reference/test-weakening-guard.md",
            "docs/reference/test-weakening-guard.zh-CN.md",
            "docs/reference/test-weakening-guard.ja.md",
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-repository/src/governance_controls.rs",
            "crates/cockpit-repository/tests/governance_signals.rs",
        ],
        "The source weakening decisions and recovery boundary are expressed through Rust-native governance signals and tests. Static text signals remain bounded evidence; no source Python/Make implementation or provider policy is copied.",
    ),
    "docs/reference/test-weakening-guard.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/reference/test-weakening-guard.md",
            "docs/reference/test-weakening-guard.zh-CN.md",
            "docs/reference/test-weakening-guard.ja.md",
            "crates/cockpit-repository/tests/governance_signals.rs",
        ],
        "This source translation carries the same weakening-guard semantics as the canonical page; Rust tri-language documentation and governance regressions preserve that meaning without treating locale bytes as policy or wire compatibility.",
    ),
    "docs/reference/test-weakening-guard.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/test-weakening-guard.md",
            "docs/reference/test-weakening-guard.zh-CN.md",
            "docs/reference/test-weakening-guard.ja.md",
            "crates/cockpit-repository/tests/governance_signals.rs",
        ],
        "This source translation carries the same weakening-guard semantics as the canonical page; Rust tri-language documentation and governance regressions preserve that meaning without treating locale bytes as policy or wire compatibility.",
    ),
    "docs/reference/verification-fixture-boundary.md": (
        "implemented-different-by-design",
        [
            "docs/reference/verification-fixture-boundary.md",
            "docs/reference/verification-fixture-boundary.zh-CN.md",
            "docs/reference/verification-fixture-boundary.ja.md",
            "tests/fixtures/README.md",
            "tests/release/adopter_acceptance.sh",
            "tests/release/isolation_manifest.sh",
        ],
        "The source fixture boundary is retained as a Rust-native repository/Release isolation rule: source inputs are separated from runtime state, caches, worktrees, and forbidden global roots. Fixture and adopter manifests provide evidence; the source helper implementation is not copied.",
    ),
    "docs/reference/troubleshooting.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/troubleshooting.md",
            "docs/reference/troubleshooting.zh-CN.md",
            "docs/reference/troubleshooting.ja.md",
            "docs/reference/agent-workflow.ja.md",
            "docs/reference/outcome-report.ja.md",
            "docs/reference/upgrade.ja.md",
        ],
        "This source translation is represented by the Rust tri-language recovery pages and explicit repository lifecycle. Japanese wizard/session controls and external toolchains remain outside Core and are not copied.",
    ),
    "docs/reference/upgrade.md": (
        "implemented-different-by-design",
        [
            "docs/reference/upgrade.md",
            "docs/reference/upgrade.zh-CN.md",
            "docs/reference/upgrade.ja.md",
            "docs/architecture/versioning.md",
            "docs/release/distribution.md",
            "tests/release/version_consistency.sh",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source upgrade distinction is preserved as shared Runtime upgrade versus explicit repository migration, with immutable Release identity, rollback, and historical evidence boundaries. Source installer and Make/Python bytes are not copied.",
    ),
    "docs/reference/upgrade.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/upgrade.md",
            "docs/reference/upgrade.zh-CN.md",
            "docs/reference/upgrade.ja.md",
            "docs/architecture/versioning.ja.md",
            "docs/release/distribution.ja.md",
            "tests/release/version_consistency.sh",
        ],
        "This source translation is represented by the Rust tri-language upgrade and distribution boundary. Runtime and repository migration remain separate; source installer, provider markers, and locale JSON are not copied.",
    ),
    "docs/reference/work-item-lifecycle-closure.ja.md": (
        "implemented-different-by-design",
        [
            "docs/reference/work-item-lifecycle-closure.md",
            "docs/reference/work-item-lifecycle-closure.zh-CN.md",
            "docs/reference/work-item-lifecycle-closure.ja.md",
            "docs/reference/agent-workflow.ja.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "This source translation is represented by the Rust tri-language closure and recovery boundary, including immutable historical evidence and explicit cleanup. Provider-specific Make/Python routes are not copied or claimed as wire compatibility.",
    ),
}

# WI-516 re-reads the next maintained release, adoption, calibration,
# baseline, capability, and canonical-evidence surfaces.  These files are
# Python/packaging/provider projections in the reference repository; the Rust
# target preserves their governance responsibilities through typed Runtime
# services and repository-bound release/adopter evidence, not by copying the
# source files or their wire formats.
WI516_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "next-release.json": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", ".github/workflows/release.yml", "docs/release/distribution.md", "tests/release/version_consistency.sh"],
        "The source candidate projection records a provider-owned release state and supply-chain claims. Rust binds candidate/public release truth to immutable release manifests, checksums, SBOM/provenance, and published-artifact acceptance; source JSON bytes are not a Runtime protocol.",
    ),
    "pyproject.toml": (
        "implemented-different-by-design",
        ["Cargo.toml", "Cargo.lock", ".github/workflows/ci.yml", "docs/reference/ci-quality-gates.md"],
        "The source file configures Python lint, typing, coverage, and pytest tooling. Rust uses Cargo-native metadata and the reviewed dynamic gate manifest; Python tool configuration is not an adopter or Runtime requirement.",
    ),
    "release-state.json": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", ".github/workflows/release.yml", "tests/release/version_consistency.sh", "docs/release/distribution.md"],
        "The source canonical/projection state machine is provider release bookkeeping. Rust keeps immutable tag, archive, digest, SBOM/provenance, and post-release acceptance bindings in its own release manifest and evidence chain; no source state file is copied.",
    ),
    "release.json": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", "tests/release/version_consistency.sh", "docs/release/distribution.md"],
        "The source published-release projection is authoritative only for that Python template's assets. Rust release manifests and downloaded artifact receipts carry equivalent identity and digest checks for this repository; source URLs, schema, and claims are not portable.",
    ),
    "requirements-dev.in": (
        "implemented-different-by-design",
        ["Cargo.toml", "Cargo.lock", ".github/workflows/ci.yml"],
        "The source development dependency declarations are Python-tooling inputs. Rust pins its own build/test dependency graph in Cargo manifests and lockfile; adopters supply their language toolchains explicitly.",
    ),
    "requirements-dev.lock": (
        "implemented-different-by-design",
        ["Cargo.lock", "tests/release/source_archive_policy_test.sh", "docs/security/enterprise-deployment-boundary.md"],
        "The source hash-pinned Python lock is not a Rust dependency or evidence format. Cargo.lock and the Rust supply-chain/archive gates provide the target-specific reproducibility boundary without importing Python packages or hashes.",
    ),
    "scripts/ai_adoption_evidence.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", "tests/release/adopter_acceptance.sh", "docs/getting-started/standard-adoption-guide.md", "docs/getting-started/adopter-configuration.md"],
        "The source builder validates a template-specific adopter verification record. Rust provides immutable public-release adopter acceptance, repository identity/isolation manifests, and the reader-first adoption guide; source Work Item ids and JSON wire shape are not copied.",
    ),
    "scripts/ai_archive_work_item.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/tests/archive_integrity.rs", "docs/reference/work-item-lifecycle-closure.md"],
        "The source Python archive transaction and projection leases map to Rust-native archive, manifest, evidence binding, recovery, and close services. Provider-specific helpers and path-rewrite implementation are not duplicated; immutable history and fail-closed cleanup are preserved.",
    ),
    "scripts/ai_baseline_evidence.py": (
        "implemented-different-by-design",
        ["crates/cockpit-verification/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/performance/baseline.md", "docs/reference/verification-cost.md"],
        "The source baseline captures Python repository counts, coverage, and scenarios. Rust records identity-bound performance samples, budgets, snapshot evidence, and verification cost observations; source coverage fields remain project/provider facts.",
    ),
    "scripts/ai_calibrate.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/project_governance.rs", "crates/cockpit-cli/src/main.rs", "docs/getting-started/calibration.md", "docs/getting-started/first-calibration.md"],
        "The source ten-stage calibration session is a Python presentation/state implementation. Rust keeps repository-owned profile proposal/confirmation, explicit human review, snapshot binding, and unknown preservation through typed project governance and CLI commands; no source session bytes are imported.",
    ),
    "scripts/ai_calibration_corrective.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/project_governance.rs", "crates/cockpit-repository/tests/project_governance.rs", "docs/getting-started/calibration.md"],
        "The source corrective declaration validator protects a live Python calibration session. Rust validates profile amendments and repository-bound governance decisions through typed Runtime services; source session paths and Python-only corrective schema are not copied.",
    ),
    "scripts/ai_calibration_inventory.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/project_governance.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-protocol/src/lib.rs", "docs/reference/capabilities.md"],
        "The source inventory aggregates profile, guard, quality, coverage, CI, and external evidence statuses. Rust exposes request-scoped capability truth, profile facts, evidence assurance, and explicit external exclusions; source inventory keys are not a universal wire contract.",
    ),
    "scripts/ai_calibration_profiles.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/project_governance.rs", "docs/reference/governance-profiles.md", "docs/getting-started/calibration.md"],
        "The source lite/standard/strict profile selector is Python policy data. Rust retains proportional repository policies and explicit profile confirmation in typed ProjectProfile/Policy surfaces; source YAML and profile-selection bytes are not copied.",
    ),
    "scripts/ai_calibration_wizard.py": (
        "implemented-different-by-design",
        ["crates/cockpit-cli/src/main.rs", "docs/getting-started/first-calibration.md", "docs/getting-started/standard-adoption-guide.md"],
        "The source interactive wizard is a provider-facing presentation adapter. Rust deliberately keeps authority in the CLI/Runtime and documents a reviewable propose/confirm flow; it does not ship a second interactive wizard or infer repository decisions.",
    ),
    "scripts/ai_canonical_evidence.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-repository/tests/archive_integrity.rs", "docs/reference/schemas.md"],
        "The source canonical evidence document and markdown renderer are replaced by typed Rust evidence, audit-event/export, digest, receipt, and archive schemas. The target preserves deterministic identity and status semantics without claiming source JSON wire compatibility.",
    ),
    "scripts/ai_capability_freshness.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-protocol/src/lib.rs", "docs/reference/capabilities.md", "docs/reference/how-to-read-cockpit-status.md"],
        "The source helper timestamps a local Python environment and marks records fresh/stale. Rust binds capability projections to the current repository snapshot and Runtime identity and leaves toolchain/provider freshness to explicit repository evidence; no source environment record is copied.",
    ),
    "scripts/ai_capability_truth.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "crates/cockpit-repository/tests/intelligence.rs", "docs/reference/capability-truth-matrix.md"],
        "The source capability matrix combines implementation status, evidence freshness, and absurd-case checks. Rust exposes typed repository-bound CapabilityTruth/AdopterCapabilityTruth with confidence, evidence refs, unknowns, and exclusions; source matrix rows and Python validation are not copied.",
    ),
}

# WI-539 re-reads the next ten maintained source checker modules.  These are
# source CI/reporting implementations, not portable Runtime wire contracts.
# The target preserves the governance responsibilities through typed Contract,
# status, Outcome, gate, and lifecycle services; reference-impact scanning is
# explicitly retained as a source/provider boundary rather than claimed as a
# Runtime capability.
WI539_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_check_guidelines.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/tests/contract_v2.rs",
            "docs/reference/contract-fields.md",
        ],
        "The source checker requires every human guideline to have a confirmed Summary claim and evidence. Rust keeps guidelines as human-owned Contract input and proves completion through typed acceptance/evidence bindings; it does not add an untyped guidelinesCompliance wire field or infer compliance.",
    ),
    "scripts/ai_check_pr.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/tests/archive_integrity.rs",
            "docs/reference/ci-quality-gates.md",
            "docs/reference/commands.md",
        ],
        "The source aggregate checker audits archive ownership, Summary validity, recovery chains, and a committed PR boundary. Rust distributes those responsibilities across the typed gate, immutable archive/finalization services, and reviewed provider PR boundary; GitHub PR identity and hosted checks remain external evidence, and source Python orchestration is not copied.",
    ),
    "scripts/ai_check_reference_impact.py": (
        "reference-only",
        [
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-core/tests/operation_time_policy.rs",
            "docs/reference/reference-parity.md",
            "docs/reference/commands.md",
        ],
        "The source performs language-specific static text/AST and build-reference scanning before delete/rename/deprecate operations. Shared Rust Runtime does not infer callers, dynamic references, external consumers, or monitoring; those facts stay adopter/provider or human-declared evidence and unknowns remain fail-closed. This source checker is reference material, not an omitted Runtime command.",
    ),
    "scripts/ai_check_registry.py": (
        "implemented-different-by-design",
        [
            "tests/ci/repository_gate_manifest.json",
            "tests/ci/run_repository_gates.py",
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/ci-quality-gates.md",
        ],
        "The source registry deduplicates checker IDs and records stage results. Rust uses a versioned gate manifest plus dynamic Contract-aware routing and typed receipts; unavailable gates are explicit, required gates cannot be silently skipped, and the source registry class is not a Runtime protocol type.",
    ),
    "scripts/ai_check_review_policy.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/agent-workflow.md",
            "docs/reference/ci-quality-gates.md",
        ],
        "The source reads a template-local YAML review policy and produces a changed-path focus report. Rust binds review requirements to the Contract, preflight/human decision receipts, and provider-owned PR review; it does not install a second YAML policy or treat a report-only focus list as approval.",
    ),
    "scripts/ai_check_scope.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/contract_preflight.rs",
        ],
        "The source checks changed paths against scope/outOfScope, allow patterns, dependency rules, and capability evidence. Rust performs repository-relative scope/out-of-scope validation, dependency and parallel-boundary checks, and fail-closed snapshot binding in typed lifecycle gates; source YAML policies are not copied.",
    ),
    "scripts/ai_check_serial_order.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "docs/reference/agent-workflow.md",
            "docs/reference/commands.md",
        ],
        "The source checks predecessor status, merged PR, closure, branch deletion, and synchronized base. Rust enforces the same delivery order through repository readiness, predecessor-bound recovery, finalization, close, and ready-on-base checks; provider PR fields remain external evidence.",
    ),
    "scripts/ai_check_status.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/status_projection.rs",
            "docs/reference/commands.md",
        ],
        "The source validates a generated current_status.md against Contract/Summary, ownership, calibration, and localized presentation. Rust exposes request-scoped typed WorkItemStatusSnapshot/Index and human Outcome projections instead of persisting a generated status markdown file; facts and digests remain repository-bound.",
    ),
    "scripts/ai_check_status_consistency.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/status_projection.rs",
            "docs/reference/how-to-read-cockpit-status.md",
        ],
        "The source reconciles active Contract/Summary ownership and a generated status file, with an optional repair command. Rust computes read-only repository-scoped status from current active/archive records, rejects orphaned or ambiguous state at lifecycle boundaries, and has no silent status-file repair authority.",
    ),
    "scripts/ai_check_summary.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/outcome_report.rs",
            "docs/reference/contract-fields.md",
            "docs/reference/outcome-report.md",
        ],
        "The source validates a broad Python Summary schema, changed-file ownership, documentation alignment, review readiness, and hosted-performance claims. Rust keeps strict typed Contract/evidence/archive/Outcome bindings and explicit partial fields; it does not claim byte-compatible Summary JSON or infer missing human claims.",
    ),
}

# WI-543 compares the next maintained source checker modules.  These modules
# are source-side reporting/orchestration surfaces; portable governance
# responsibilities are recorded against the existing typed Rust Runtime
# boundaries without copying Python, provider, or source wire formats.
WI543_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_check_task_outcome.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/src/outcome_render.rs",
            "crates/cockpit-repository/tests/outcome_report.rs",
            "crates/cockpit-cli/tests/outcome_handoff.rs",
        ],
        "The source validates a task-outcome JSON/Markdown projection, bindings, claims, events, and human status. Rust provides typed OutcomeV2/TaskOutcomeReport, append-only events, localized human handoff, and archive bindings; source JSON wire shape and lexical report policy are not copied.",
    ),
    "scripts/ai_check_test_weakening.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/governance_signals.rs",
            "docs/reference/test-weakening-guard.md",
        ],
        "The source emits static signals for verification-artifact removal, assertion/coverage reduction, workflow bypass, and narrow retirement evidence. Rust derives bounded snapshot signals and fail-closed unknowns at governance boundaries; source detector thresholds and Python report format remain source/provider maintenance policy.",
    ),
    "scripts/ai_classify_operation_impact.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/governance_signals.rs",
            "docs/reference/operation-time-policy-reevaluation.md",
        ],
        "The source derives compatibility/configuration/test-evidence impact from declared actions and changed paths. Rust OperationTimeRequest and repository governance signals perform explicit operation-time safety and scope evaluation; it does not infer intent or import the source report wire shape.",
    ),
    "scripts/ai_close_work_item.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/tests/resource_finalization_transition.rs",
            "docs/reference/work-item-lifecycle-closure.md",
        ],
        "The source Python close workflow validates archived evidence, PR/base synchronization, worktree/branch cleanup, and closure receipts. Rust owns these lifecycle/finalization/ready-on-base invariants with typed receipts; provider PR operations and source runner orchestration remain external.",
    ),
    "scripts/ai_common.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "tests/conformance/reference_file_inventory.py",
        ],
        "The source helper module centralizes JSON/YAML, Git, scope/glob, redaction, registry, and command utilities. Rust distributes these concerns across typed Protocol/Core/repository services and conformance tooling with explicit path and identity checks; a monolithic Python helper is not a Runtime dependency.",
    ),
    "scripts/ai_critical_domain_guards.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-repository/src/governance_controls.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/governance_signals.rs",
        ],
        "The source classifies critical-domain, bypass, evidence-forgery, and production-operation signals from Contract text. Rust keeps human-owned intent, typed operation/authority controls, prompt-injection and forgery findings, and fail-closed decisions without promoting lexical classification to authority.",
    ),
    "scripts/ai_dependabot_intake.py": (
        "not-applicable",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "docs/reference/ci-release-evidence.md",
        ],
        "Dependabot candidate event identity, locked dependency classification, and automatic bot-branch intake are provider-specific. Rust supports generic delegated evidence and explicit source binding, but does not ship a Dependabot workflow or provider event parser.",
    ),
}

WI548_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_derived_artifacts.py": (
        "implemented-different-by-design",
        [
            "docs/reference/outcome-report.md",
            "docs/reference/verification-semantics.md",
            ".ai/README.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source validator protects a fact-versus-derived-view registry. Rust preserves that authority boundary through typed Contract/evidence/archive/status/Outcome projections and repository-local derived Knowledge; derived views cannot authorize a later decision, and the source Python registry or JSON wire shape is not copied.",
    ),
    "scripts/ai_detached_uninstaller.py": (
        "reference-only",
        [
            "docs/reference/installed-lifecycle.md",
            "docs/reference/commands.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source detached-process uninstaller is an installer/provider implementation. Rust documents proposal-before-write, ownership, bounded removal, evidence retention, and fail-closed recovery, but does not provide a detached uninstaller or claim that deleting a local binary disposes repository state.",
    ),
    "scripts/ai_disable_enable.py": (
        "reference-only",
        [
            "docs/reference/installed-lifecycle.md",
            "docs/reference/commands.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source disable/enable state machine is a template installer control. Rust has no global installed-state toggle; its shared Runtime remains request-scoped and repository attachment/lifecycle controls are explicit, preserving evidence and refusing unsafe recovery without copying the source state file or commands.",
    ),
    "scripts/ai_doctor.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-cli/src/main.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/tests/doctor.rs",
            "docs/reference/commands.md",
        ],
        "The source doctor aggregates Python/Make installation, POSIX, hosted-snapshot, and adopter facts. Rust provides a repository-bound JSON doctor with protocol/runtime identity, compatibility, Runtime-code exclusion, and fail-closed repository diagnostics; provider toolchain and source Make targets remain adopter boundaries.",
    ),
    "scripts/ai_documentation_authority.py": (
        "implemented-different-by-design",
        [
            ".ai/README.md",
            "AGENTS.md",
            "docs/current/README.md",
            "docs/reference/README.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The source registry validates canonical/reference/historical documentation routing. Rust keeps one repository-owned current read set, explicit current/getting-started/reference routes, frontmatter and documentation acceptance; it does not add a second generic authority CLI or trust source plan metadata as instruction.",
    ),
    "scripts/ai_documentation_journey.py": (
        "implemented-different-by-design",
        [
            "docs/current/README.md",
            "docs/getting-started/README.md",
            "docs/reference/README.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "The source topic registry validates reader-criticality, localized paths, fallback labels, and next-topic links. Rust expresses the maintained reader journey through current/getting-started/reference indexes, tri-language links, frontmatter, and documentation gates; source registry fields and Python checker commands are not Runtime protocol.",
    ),
    "scripts/ai_domain_model.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-core/tests/domain_types.rs",
        ],
        "The source module centralizes lifecycle vocabulary, typed facts, receipts, decisions, and canonical transitions. Rust owns the same authority in typed Core/Protocol records and repository lifecycle services, with stronger repository identity, snapshot, evidence, and Runtime bindings; source dataclasses and JSON shape are not copied.",
    ),
    "scripts/ai_enterprise_control_evidence.py": (
        "implemented-different-by-design",
        [
            "docs/security/enterprise-governance.md",
            "docs/security/enterprise-governance.zh-CN.md",
            "docs/security/enterprise-governance.ja.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/tests/evidence_assurance.rs",
        ],
        "The source control evaluator keeps local records from becoming enterprise compliance verdicts and rejects expired or missing external evidence. Rust preserves this delegated boundary with typed assurance, external evidence import, expiry/retention validation, and explicit non-certification claims.",
    ),
    "scripts/ai_evidence_dependencies.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-verification/src/lib.rs",
            "crates/cockpit-repository/tests/evidence_assurance.rs",
            "crates/cockpit-repository/tests/verification_service.rs",
        ],
        "The source dependency matrix binds changed paths and generated evidence to declared capabilities and source inputs. Rust binds verification receipts to Work Item, repository, snapshot, Contract, profile, policy, command, stage, runner, and Runtime identity; source matrix files and provider-specific generated-artifact rules are not copied.",
    ),
    "scripts/ai_external_handoff.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-release/src/handoff.rs",
            "crates/cockpit-mcp/src/lib.rs",
            "docs/security/enterprise-governance.md",
            "docs/reference/outcome-report.md",
        ],
        "The source generic handoff records an external fulfiller, deadline, bindings, and receipt ingestion without contacting the provider. Rust keeps typed, digest-bound release handoff and repository-bound MCP/Outcome projections; external provider execution and identity remain delegated evidence, not local authorization.",
    ),
    "scripts/ai_external_identity.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/evidence_assurance.rs",
            "docs/security/enterprise-governance.md",
        ],
        "The source distinguishes self-declared, repository-recorded, provider, enterprise, and direct-user approval evidence and requires stronger bindings for high-risk work. Rust uses typed authority/approval evidence, policy-defined approval modes, repository identity, and delegated provider/enterprise evidence without authenticating a person locally.",
    ),
    "scripts/ai_final_north_star_acceptance.py": (
        "implemented-different-by-design",
        [
            "docs/reference/final-replacement-acceptance.md",
            "tests/conformance/final_replacement_acceptance.sh",
            "crates/cockpit-repository/src/governance_controls.rs",
        ],
        "The source evaluator requires all twenty named dimensions and refuses GO without verified adopter and provider evidence. Rust preserves the bounded North Star through final-replacement acceptance, parity evidence, and explicit external limitations; source decision JSON and historical release claims are not copied.",
    ),
    "scripts/ai_impact_classifier.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/governance_controls.rs",
            "crates/cockpit-core/src/lib.rs",
            "docs/reference/governance-profiles.md",
            "docs/reference/operation-time-policy-reevaluation.md",
        ],
        "The source path classifier is a lightweight Python routing hint. Rust derives impact only from explicit Contract, operation-time policy, changed-path, and profile facts; unknown impact remains visible/fail-closed and never authorizes a weaker route. Source lexical categories are not a wire enum.",
    ),
}

WI550_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_finish.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/outcome_render.rs",
            "docs/reference/work-item-lifecycle-closure.md",
        ],
        "The source finish workflow combines mutex, branch/base, evidence, checkpoint, recovery, and human-report orchestration. Rust owns those lifecycle and evidence invariants through typed repository services and visible Outcome projections; source Python process/provider orchestration and report wire formats are not copied.",
    ),
    "scripts/ai_generate_human_report.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/outcome_render.rs",
            "docs/reference/outcome-report.md",
        ],
        "Typed OutcomeV2 and TaskOutcomeReport preserve report phases, findings, risks, closure facts, and evidence bindings. The source report schema and Python generator are not a Rust wire contract.",
    ),
    "scripts/ai_generate_status.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            "docs/reference/commands.md",
        ],
        "Repository-bound Rust status derives lifecycle, readiness, blockers, and Outcome facts from typed records. It intentionally does not create a generated current_status.md authority or copy source status JSON.",
    ),
    "scripts/ai_generate_task_outcome.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/outcome_render.rs",
            "docs/reference/outcome-report.md",
        ],
        "Typed TaskOutcomeReport and append-only TaskOutcomeEvent records provide deterministic findings, risks, interventions, evidence, and next actions. Source task-report JSON and lexical policy remain source-specific.",
    ),
    "scripts/ai_governance_compression.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/governance_controls.rs",
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-verification/src/lib.rs",
            "docs/reference/verification-semantics.md",
        ],
        "The source compression helper combines policy signals and recommendations for presentation. Rust keeps governance decisions in typed policy, operation-time evaluation, verification routing, and evidence controls; source compression output is not authority or a copied protocol.",
    ),
    "scripts/ai_input_trust.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-repository/src/governance_controls.rs",
            "crates/cockpit-repository/tests/input_trust.rs",
            "docs/reference/input-trust-dataflow.md",
        ],
        "Source/provenance, injection, authority, and operation-time trust semantics are represented by typed request binding, untrusted-material evaluation, and fail-closed governance signals. The source provenance enum set and Python API are not copied as wire compatibility.",
    ),
    "scripts/ai_japanese_capability.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/outcome_render.rs",
            "crates/cockpit-mcp/src/lib.rs",
            "tests/conformance/reference_inventory_docs_test.py",
            "docs/capabilities.ja.md",
        ],
        "The source self-assessment script is replaced by Rust-native tri-language Outcome/MCP projections, documentation checks, and conformance tests. It does not grant capability or policy authority and is not copied.",
    ),
    "scripts/ai_lifecycle_facts.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            "docs/reference/commands.md",
        ],
        "Read-only lifecycle facts are emitted through typed status, inspect, and doctor projections with repository identity and readiness bindings; no generated Python facts file is treated as authority.",
    ),
    "scripts/ai_lifecycle_truth.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/lifecycle_order.rs",
            "docs/reference/work-item-lifecycle-closure.md",
        ],
        "Immutable lifecycle, successor, recovery, finalization, and archive truth is owned by typed Rust receipts and repository services. The source Python projection and installer-specific chain are not copied.",
    ),
    "scripts/ai_multilingual_semantic_parity.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/outcome_render.rs",
            "tests/conformance/reference_inventory_docs_test.py",
            "docs/reference/outcome-report.md",
        ],
        "Rust preserves controlled multilingual presentation while keeping Contract bytes and governance facts locale-neutral. Tri-language renderer and documentation tests replace the source utility; contract-language acceptance text is never silently translated.",
    ),
    "scripts/ai_observability.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-verification/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/verification-semantics.md",
        ],
        "Verification receipts and TaskOutcomeEvent JSONL retain deterministic timing, reuse, execution, and lifecycle facts. The source generic observability sink is not a required Runtime API; provider telemetry remains external.",
    ),
    "scripts/ai_post_archive_recovery.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/resource_finalization_transition.rs",
            "docs/reference/work-item-lifecycle-closure.md",
        ],
        "Recovery and post-archive finalization are typed, immutable, identity-bound Rust paths. Hosted/provider failure parsing and source Python orchestration remain external boundaries rather than copied Runtime behavior.",
    ),
    "scripts/ai_render_task_outcome.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/outcome_render.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "docs/reference/outcome-report.md",
        ],
        "The Rust Outcome renderer provides a deterministic human handoff with status markers, evidence, unknowns, decisions, and next action. Source Markdown renderer code and its wire shape are not copied.",
    ),
    "scripts/ai_render_task_outcome_multilingual.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/outcome_render.rs",
            "crates/cockpit-mcp/src/lib.rs",
            "docs/reference/outcome-report.md",
        ],
        "The Rust renderer localizes fixed Runtime chrome for English, Simplified Chinese, and Japanese while preserving original Contract text. MCP and CLI expose the same human handoff boundary without importing source locale tables.",
    ),
    "scripts/ai_render_task_outcome_pr.py": (
        "reference-only",
        [
            "docs/reference/outcome-report.md",
            "docs/reference/ci-release-evidence.md",
            "crates/cockpit-mcp/src/lib.rs",
        ],
        "The source PR summary renderer is a provider-facing presentation helper. Rust exposes digest-bound Outcome and release/MCP handoffs, while PR-provider formatting and hosted identity remain external; no Runtime omission is claimed.",
    ),
    "scripts/ai_required_evidence.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/governance_controls.rs",
            "crates/cockpit-verification/src/lib.rs",
            "docs/reference/verification-semantics.md",
        ],
        "The source dynamic evidence rule registry is represented by typed Contract requiredEvidenceClasses, delegated evidence, policy-bound verification routing, and release/permission controls. Source provider-specific rule identifiers and JSON payloads are not universal Rust protocol fields.",
    ),
}

WI552_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_install_facts.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "docs/reference/configuration.md", "docs/release/distribution.md"],
        "The source records installer facts, ownership, release identity, and rollback baselines inside the project. Rust keeps the shared Runtime installation external and binds repository facts through attach, inspect, compatibility, doctor, and immutable release/adopter evidence; source .ai/install bytes are not copied.",
    ),
    "scripts/ai_install_plan.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "The source read-only wizard plan is represented by explicit Rust command boundaries and repository-bound migration/adapter plans. The target does not add an interactive installer wizard or provider-specific plan wire format.",
    ),
    "scripts/ai_install_status.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "Source installer status validates local release facts and lifecycle state. Rust exposes repository/runtime status, compatibility, migration, and doctor projections without manufacturing a source current-status file.",
    ),
    "scripts/ai_install_wizard.py": (
        "implemented-different-by-design",
        ["crates/cockpit-cli/src/main.rs", "docs/getting-started/installation.md", "docs/reference/commands.md"],
        "The source TTY wizard's confirmation and localization boundary is preserved as explicit, reviewable CLI commands; interactive host conversation remains an Agent adapter concern and no implicit writes, commits, or provider configuration are introduced.",
    ),
    "scripts/ai_installer_bootstrap.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/capabilities.md"],
        "Source bootstrap creates adoption records. Rust attach and Work Item scaffolding create only the minimum repository-owned protocol skeleton and leave governance decisions to humans.",
    ),
    "scripts/ai_installer_catalog.json": (
        "reference-only",
        ["docs/reference/configuration.md", "docs/release/distribution.md"],
        "The source catalog is a provider/script inventory for its installer. Runtime command discovery is defined by the strict agent-interface manifest and CLI/MCP schemas; copying the source catalog would overclaim supported providers and commands.",
    ),
    "scripts/ai_installer_detection.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "Source detection gathers Git, tool, and adoption facts. Rust uses explicit repository observation, inspect, status, doctor, profile, and compatibility facts without inferring a source installer mode or silently choosing a provider.",
    ),
    "scripts/ai_installer_evidence.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", "tests/release/adopter_acceptance.sh", "docs/release/adopter-acceptance.md"],
        "Source summarizes installer actions and managed roots. Rust records immutable release/adopter acceptance, manifests, digests, and repository-local adapter ownership; it does not copy source action-summary JSON.",
    ),
    "scripts/ai_installer_managed_regions.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/configuration.md"],
        "Source checks managed project regions. Rust makes Agent adapter ownership explicit, verifies marked sections and regular paths, and refuses ambiguous or modified regions instead of applying source installer heuristics.",
    ),
    "scripts/ai_installer_ownership.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-protocol/src/lib.rs", "docs/reference/configuration.md"],
        "Source classifies project-owned installer files. Rust uses typed repository-local adapter ownership and strict protocol/profile records; ownership never grants governance authority or global configuration access.",
    ),
    "scripts/ai_installer_repository.py": (
        "implemented-different-by-design",
        ["crates/cockpit-git/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/architecture/runtime-topology.md"],
        "Source reads Git and repository hygiene facts with explicit roots. Rust request-scopes every operation to --repo, uses the shared Git observer, and fail-closes on dirty, ambiguous, or identity-mismatched repository state.",
    ),
    "scripts/ai_installer_transaction.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-agent/src/lib.rs", "docs/reference/commands.md"],
        "Source orders installer writes, validates paths, and takes an installer lock. Rust uses atomic repository-local writes, strict path/ownership validation, explicit adapter/migration confirmation, and no source installer transaction protocol.",
    ),
    "scripts/ai_installer_upgrade.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", "crates/cockpit-protocol/src/lib.rs", "docs/release/distribution.md"],
        "Source parses installer release versions. Rust binds immutable public Runtime artifacts and typed repository schema compatibility/migration; Runtime upgrade remains external to each attached repository.",
    ),
    "scripts/ai_upgrade_apply.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "Source applies a confirmed upgrade with drift checks and rollback. Rust separates external Runtime replacement from explicit migrate apply, preserves historical evidence/knowledge, and writes a digest-bound migration receipt only for reviewed adjacent schema changes.",
    ),
    "scripts/ai_upgrade_conflict_report.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "Source emits a stable installer conflict report. Rust exposes compatibility/migration plans and doctor safe actions with fail-closed conflicts; it does not adopt the source report wire format or auto-resolve ownership.",
    ),
    "scripts/ai_upgrade_proposal.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "Source compares template files and proposes safe/conflicting upgrade changes. Rust's migration plan is schema- and digest-bound, keeps repository-owned history immutable, and requires explicit approval; source template file categories are not copied.",
    ),
    "scripts/install_ai_cockpit.py": (
        "reference-only",
        ["docs/release/distribution.md", "docs/getting-started/installation.md"],
        "The source Python launcher is an installer entrypoint. This Rust repository ships a binary through immutable release artifacts and documents package/installer boundaries; no Python launcher or source fallback is part of the Runtime.",
    ),
}

WI557_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_issue_log.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/task_outcome_events.rs",
            "docs/features/task-outcome-report.md",
        ],
        "The source append-only issue record, sensitive-value rejection, transition checks, and reviewer overview are represented by typed TaskOutcomeEvent findings/resolutions, stable fingerprints, evidence-bound Outcome sections, and repository lifecycle validation. Rust intentionally does not copy the source issue-id/status wire or infer issue ownership from prose.",
    ),
    "scripts/ai_linked_worktree_recovery.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/resource_finalization_transition.rs",
            "docs/reference/recovery.md",
            "docs/reference/repository-workflow.md",
        ],
        "The source diagnostic-only foreign linked-worktree recovery path is preserved through Git topology checks, identity-bound finalization/recovery receipts, and explicit owner-only recovery actions. Rust keeps recovery request-scoped and non-mutating until the owning Work Item authorizes a change; the Python report shape is not copied.",
    ),
    "scripts/ai_ownership.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-agent/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/configuration.md",
        ],
        "The source ownership classes, managed-region boundaries, digest facts, and fail-closed mutation decision are represented by typed repository/adapter ownership records, regular-file and marked-region checks, and immutable historical ownership. Rust does not treat a path heuristic or ownership label as governance authority and does not copy the generic Python parser.",
    ),
    "scripts/ai_performance_budget.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-verification/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/governance-performance-budget.md",
            "tests/performance/regression_gate.sh",
        ],
        "The source local profile samples and P95 baseline are projected into identity-bound Rust PerformanceBaseline samples, budgets, and cost observations. Measurements remain advisory and never lower a verification requirement; source profile names, JSONL reports, and automatic P95 inference are not Runtime authority.",
    ),
    "scripts/ai_project_profile.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            ".ai/project.json",
            ".ai/project/profile-policy.json",
            "docs/reference/governance-profiles.md",
        ],
        "The source YAML Profile facts, suggested/approved boundaries, unknowns, and approval checks are represented by typed Project Profile facts plus repository-local profile-policy and read-only profile propose/validate commands. The target keeps candidate and approved state separate and does not copy the source YAML schema or infer approval.",
    ),
    "scripts/ai_purge.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "docs/security/enterprise-governance.md",
            "docs/reference/commands.md",
        ],
        "The source export-before-purge, protected-path filtering, double-confirmation, and digest-bound receipt are represented by retention metadata and the read-only evidence purge-plan. Rust never silently deletes repository evidence; final disposal remains an explicit external/owner action and the source purge receipt wire is not copied.",
    ),
    "scripts/ai_readiness_policy.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            "docs/reference/repository-workflow.md",
            "docs/reference/verification-route.md",
        ],
        "The source separates installed, calibrated, and production-ready states and reports static evidence without executing project commands. Rust preserves that separation through repository status/doctor, profile and policy projections, dynamic verification routing, and explicit unknowns; adopter CI/review policy remains external and source readiness JSON is not copied.",
    ),
    "scripts/ai_recovery_usability.py": (
        "reference-only",
        [
            "docs/reference/recovery.md",
            "docs/reference/troubleshooting.md",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "The source validates a complete fixed set of user-facing recovery scenarios and renders a generic guidance report. Rust provides identity-bound lifecycle/recovery receipts and human Outcome recovery conditions, but does not currently expose the source scenario registry or guarantee one generic guidance record for every scenario; this bounded projection remains reference-only rather than an equivalence claim.",
    ),
    "scripts/ai_review_readiness_policy.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "docs/reference/outcome-report.md",
            "docs/reference/repository-workflow.md",
        ],
        "The source review-readiness signal is represented by preflight/review gates, typed Outcome report sections, explicit blockers, and provider review evidence. Rust does not copy the report-only reviewReadiness field or treat a focus list as approval; missing review facts remain visible and fail closed where required.",
    ),
    "scripts/ai_risk_policy.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/task_outcome_events.rs",
            "docs/reference/outcome-report.md",
        ],
        "The source residual-risk projection is represented by typed Contract risk, Outcome residualRisks/risks, stable finding fingerprints, and explicit unknowns. Rust preserves the highest-observed risk without inventing a level or converting a local summary into an organizational approval.",
    ),
    "scripts/ai_rollback.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "docs/release/distribution.md",
            "docs/reference/upgrade.md",
        ],
        "The source snapshot, drift check, confirmation, partial-rollback, and project-config preservation semantics are represented by immutable release identity, explicit repository migration plans, rollback guidance, and preservation of historical evidence. Rust does not provide a source-compatible managed-region restore function or silently roll back project-owned content.",
    ),
    "scripts/ai_safety_gate.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/tests/operation_time_policy.rs",
            "docs/reference/operation-time-policy-reevaluation.md",
        ],
        "The source dangerous-case and verified-evidence gate is represented by operation-time policy re-evaluation, explicit operation/scope/authority/trust/freshness checks, and fail-closed lifecycle gates. Rust keeps the executor/provider boundary separate and does not copy the source case-name or result wire format.",
    ),
    "scripts/ai_schema_migration.py": (
        "implemented-different-by-design",
        [
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-cli/src/main.rs",
            "docs/reference/upgrade.md",
            "docs/reference/configuration.md",
        ],
        "The source explicit registry, adjacent transition plan, policy-impact confirmation, reverse-migration stop, and applied receipt are represented by the typed repository migration graph, compatibility/migrate plan, approved apply, preserved-history digest, and migration receipt. Rust does not copy the source registry or permit an unreviewed schema rewrite.",
    ),
}

WI559_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_onboard.py": (
        "implemented-different-by-design",
        ["crates/cockpit-cli/src/main.rs", "crates/cockpit-repository/src/lib.rs", "docs/getting-started/installation.md", "docs/reference/commands.md"],
        "The source three-phase onboarding wizard is represented by explicit shared-Runtime attach, inspect, status, doctor, and profile commands. Rust keeps calibration and approval human-owned and does not copy source Make targets or auto-confirm project policy.",
    ),
    "scripts/ai_prepare_hosted_verification.py": (
        "reference-only",
        ["docs/reference/repository-workflow.md", "docs/release/distribution.md", "AGENTS.md"],
        "The source hosted-snapshot exception is provider-specific preparation. Rust intentionally has no equivalent command; hosted, CI, and release evidence remain external and must use the published-artifact boundary.",
    ),
    "scripts/ai_project_doctor.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "The source read-only project fact scanner maps to typed RepositoryObservation, inspect/status/doctor, and profile projections. Rust reports deterministic repository facts without copying broad source heuristics or inferring policy approval.",
    ),
    "scripts/ai_projection_lease.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-repository/tests/parallel_boundary.rs", "docs/reference/cross-work-item-dedup.md"],
        "The source file-lock and projection serialization semantics map to repository-local concurrency boundaries, leases, scope-overlap checks, and bounded parallel verification. Rust uses typed request-scoped records rather than the source lock-file protocol.",
    ),
    "scripts/ai_provider_merge_state_recovery.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-protocol/src/lib.rs", "docs/reference/troubleshooting.md", "docs/reference/provider-reconciliation-boundary.md"],
        "The source provider merge-state recovery facts map to typed finalization/recovery receipts and delegated external provider evidence. Rust validates identity, ancestry, and cleanup without claiming the source provider-specific signed-PR workflow.",
    ),
    "scripts/ai_quality_architecture.py": (
        "reference-only",
        ["docs/reference/ci-quality-gates.md", "tests/ci/quality_route_test.py"],
        "The source AST scanner audits Python implementation details and is not a portable Runtime requirement. Rust uses Cargo, Clippy, workspace tests, and repository-native quality gates; no generic Python scanner is claimed.",
    ),
    "scripts/ai_resume_work_item.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/troubleshooting.md", "docs/reference/repository-workflow.md"],
        "The source resume/rebase flow maps to typed resume and synchronization history, predecessor closure evidence, identity-bound recovery, and mandatory revalidation. Rust preserves append-only history without copying the source CLI or branch automation.",
    ),
    "scripts/ai_start.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/repository-workflow.md"],
        "The source start checks map to explicit repository-bound scaffolding, duplicate reservation, base/branch/worktree identity, concurrency gates, and preflight. Rust uses its own Contract protocol and does not emit a source Start Receipt wire format.",
    ),
    "scripts/ai_start_receipt.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/repository-workflow.md"],
        "The source immutable Start Receipt bindings map to Contract base/scope/snapshot identity, resume and synchronization histories, and lifecycle receipts. Rust keeps the target protocol typed and repository-local rather than copying the source receipt schema.",
    ),
    "scripts/ai_task_event_log.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-repository/tests/task_outcome_events.rs", "docs/features/task-outcome-report.md"],
        "The source append-only event log, finding fingerprints, required fields, correction relations, and secret rejection map to typed TaskOutcomeEvent JSONL and archive/Outcome bindings. Rust does not copy the source event-name registry or wire format.",
    ),
    "scripts/ai_terminology.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "docs/reference/governance-profiles.md", "docs/reference/outcome-report.md", ".ai/glossary.md"],
        "The source governance/calibration profiles and status colors map to typed Runtime policy, Outcome decision states, and tri-language glossary labels. Rust keeps verification strength and assurance separate and does not treat source profile names as authority.",
    ),
    "scripts/ai_trust_guards.py": (
        "implemented-different-by-design",
        ["crates/cockpit-core/src/lib.rs", "crates/cockpit-repository/src/governance_controls.rs", "crates/cockpit-repository/src/project_governance.rs", "docs/reference/input-trust-dataflow.md"],
        "The source trust signals and allow/review/confirm/defer/block states map to typed operation, intent, scope, authority, unknown, and human-review evaluation. Rust rejects unsupported or ambiguous claims without copying the Python guard API.",
    ),
    "scripts/ai_trust_schema.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-core/src/lib.rs", "docs/reference/input-trust-dataflow.md"],
        "The source strict schema subset maps to serde typed records, deny-unknown-fields validation, and Rust-native trust tests. Source schema examples and validator entrypoints are not Runtime wire compatibility requirements.",
    ),
    "scripts/ai_uninstall_facts.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/installed-lifecycle.md"],
        "The source installer facts and drift checks map to typed adapter ownership, agent doctor/detach/repair, repository identity, and retention metadata. Rust does not copy a source installer manifest or provide a detached uninstaller executor.",
    ),
    "scripts/ai_uninstall_proposal.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/installed-lifecycle.md", "docs/security/enterprise-governance.md"],
        "The source disable/preserve/purge proposal maps to explicit adapter detach proposals, evidence retention and purge plans, ownership/drift checks, and human authorization. Rust never silently deletes repository evidence and does not copy source proposal JSON.",
    ),
    "scripts/ai_unknown_confirmation.py": (
        "implemented-different-by-design",
        ["crates/cockpit-core/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-repository/tests/preflight_review.rs", "docs/reference/agent-workflow.md"],
        "The source confirmation request maps to identity-bound preflight humanDecisionRequest, explicit unknowns, scope and evidence digests, expiry, and fail-closed review. Rust keeps confirmation policy typed and does not accept an OK-only shortcut.",
    ),
    "scripts/ai_validate_java_runtime.py": (
        "reference-only",
        ["docs/reference/verification-route.md", "docs/reference/python-fixture-adaptation.md"],
        "The source Java selector is a stack-specific adopter helper. Rust accepts explicit provider command argv but does not bundle Java or JAVA_HOME discovery; Java lane selection remains an adopter/provider responsibility.",
    ),
    "scripts/ai_verification_context.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-verification/src/lib.rs", "docs/reference/affected-verification.md"],
        "The source immutable verification context maps to request-scoped RepositorySnapshot/Observation, Contract and Summary bindings, changed paths, impact, and cached observation. Rust uses typed context and does not expose the source mapping-proxy API.",
    ),
    "scripts/ai_verification_policy.py": (
        "implemented-different-by-design",
        ["crates/cockpit-verification/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/lightweight-verification-and-soft-gates.md", "docs/reference/governance-profiles.md"],
        "The source tier policy, stage floors, escalation, DAG, cache key, and receipt binding map to Rust dynamic verification planning, orthogonal Tier/Assurance, dependency graph, reuse, and evidence contexts. Source policy JSON and profile names are not copied.",
    ),
    "scripts/ai_verify.py": (
        "implemented-different-by-design",
        ["crates/cockpit-cli/src/main.rs", "crates/cockpit-verification/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/verification-route.md"],
        "The source task/PR/release verification orchestration maps to Rust verify routes, checker registry, dynamic planner, Contract gates, and delegated release/adopter evidence. Rust preserves the provider boundary and does not copy source command modes or report wire.",
    ),
}

WI563_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/ai_wizard_io.py": (
        "reference-only",
        [".ai/README.md", "crates/cockpit-agent/src/lib.rs", "docs/reference/agent-workflow.md"],
        "The source TTY input primitives are a presentation-adapter implementation. The Rust Runtime exposes explicit non-interactive CLI/MCP schemas and visible Outcomes; host or Agent adapters own conversation controls, so the source wizard I/O module is not a Core capability or wire contract.",
    ),
    "scripts/ai_wizard_localization.py": (
        "implemented-different-by-design",
        ["crates/cockpit-cli/src/main.rs", "crates/cockpit-mcp/src/lib.rs", "docs/reference/outcome-report.md", "docs/reference/commands.md"],
        "Language normalization and localized Runtime chrome are provided by the CLI/MCP presentation layer. Contract intent, scope, acceptance, and other governance facts remain in their authored language; source locale files and placeholder API are not copied.",
    ),
    "scripts/ai_work_item_intelligence.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-knowledge/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/work-item-intelligence-interface.md"],
        "Fact-derived Work Item intelligence, append-only lifecycle evidence, repository-local indexing, request-scoped query, and v2 projections are represented by typed Protocol, repository, knowledge, and CLI services. Rust does not copy the source Python cache or global worktree aggregation and never infers human decisions.",
    ),
    "scripts/ai_work_item_intelligence_benchmark.py": (
        "reference-only",
        ["tests/performance", "docs/reference/verification-cost.md", "docs/reference/governance-performance-budget.md"],
        "The source benchmark harness measures a Python implementation and emits source-specific percentile reports. Rust keeps performance samples and regression gates in its native test/release boundary; benchmark numbers are advisory and cannot authorize or weaken governance.",
    ),
    "scripts/ai_work_item_status.py": (
        "implemented-different-by-design",
        ["crates/cockpit-cli/src/main.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/commands.md", "docs/reference/how-to-read-cockpit-status.md"],
        "Read-only status and intelligence are exposed through repository-bound `status` and `work-item status` projections with stable JSON and human Outcome rendering. No generated Python status file or implicit current-worktree state is used as authority.",
    ),
    "scripts/bootstrap_repository.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-git/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "Repository fact detection, remote/default-branch observation, dirty/conflict checks, drift binding, and installed Runtime identity are represented by the shared Git observer and explicit `inspect`/`observe`/`status`/`doctor` commands. The source snapshot dataclasses are not a Rust wire format.",
    ),
    "scripts/bootstrap_wizard.py": (
        "reference-only",
        [".ai/README.md", "docs/getting-started/installation.md", "docs/reference/commands.md", "crates/cockpit-agent/src/lib.rs"],
        "The source in-memory Bootstrap Wizard is an interactive presentation state machine. Rust deliberately keeps detection, proposal, confirmation, attachment, and Agent installation as explicit repository-bound commands and does not add a second wizard that could manufacture authority or readiness.",
    ),
    "scripts/bootstrap_write_boundary.py": (
        "implemented-different-by-design",
        ["crates/cockpit-repository/src/lib.rs", "crates/cockpit-agent/src/lib.rs", "docs/reference/configuration.md", "docs/reference/commands.md"],
        "Allowlisted repository writes, regular-file and symlink checks, atomic ownership, confirmation, and drift protection are enforced by typed attach/migration/adapter services. The source generic Makefile block protocol is not copied and never grants global configuration access.",
    ),
    "scripts/check_bandit_baseline.py": (
        "not-applicable",
        [],
        "This checker and its baseline are specific to the reference repository's Python/Bandit tooling. The Rust Runtime has no Python/Bandit product surface; Cargo, Clippy, and Rust-native tests are the applicable quality controls.",
    ),
    "scripts/check_changed_critical_coverage.py": (
        "implemented-different-by-design",
        [".github/workflows/ci.yml", "tests/ci/quality_route.py", "tests/ci/run_repository_gates.py", "crates/cockpit-repository/src/governance_controls.rs", "docs/reference/ci-quality-gates.md"],
        "Changed-critical coverage selection, policy binding, candidate snapshots, and fail-closed gate routing are represented by the reviewed CI gate manifest and Runtime Contract/verification controls. Rust does not run the source pytest predictor or copy its candidate-report wire.",
    ),
    "scripts/check_ci_release_evidence.sh": (
        "implemented-different-by-design",
        ["tests/release/adopter_acceptance.sh", "tests/release/adopter_upgrade_acceptance.sh", ".github/workflows/release.yml", "docs/release/distribution.md"],
        "CI/release evidence field, digest, artifact, and identity checks are provided by the Rust release/adopter harnesses, immutable archive/SHA256/SBOM/provenance checks, and post-release acceptance. The source shell checker is not a Runtime fallback.",
    ),
    "scripts/check_critical_coverage.py": (
        "reference-only",
        [".github/workflows/ci.yml", "tests/performance/regression_gate.sh", "docs/reference/ci-quality-gates.md"],
        "The source per-file Python coverage-floor checker depends on pytest/Bandit-era implementation surfaces. Rust retains applicable package/test and performance gates, but does not claim the source file-level threshold or its JSON report as a universal Runtime requirement.",
    ),
    "scripts/check_deprecated_assets.py": (
        "reference-only",
        ["crates/cockpit-repository/src/lib.rs", "docs/reference/recovery.md", "docs/security/enterprise-governance.md"],
        "The source registry scanner identifies template-specific deprecated assets and prohibited command chains. Rust keeps immutable history, explicit resource finalization, retention metadata, and owner-approved cleanup; it does not install a generic deletion authority or source registry.",
    ),
    "scripts/check_dev_tool_versions.py": (
        "implemented-different-by-design",
        ["Cargo.lock", "rust-toolchain.toml", ".github/workflows/ci.yml", "docs/reference/ci-quality-gates.md"],
        "Development tool reproducibility is expressed through Cargo lock/toolchain metadata, pinned action/tool versions, and CI quality checks. Python package pin parsing and its recovery-command output remain source-specific.",
    ),
    "scripts/check_docs_metadata.py": (
        "implemented-different-by-design",
        ["tests/docs/documentation_acceptance.sh", "tests/docs/promote_closed_work_item.py", "docs/reference/documentation-authority-boundary.md", "docs/reference/reference-file-comparison.md"],
        "Front matter, reader-route links, tri-language counterparts, command evidence, version-neutral claims, and closed-Work-Item promotion are checked by repository-native documentation acceptance and promotion gates. The source Python checker and metadata schema are not copied as Runtime protocol.",
    ),
    "scripts/check_governance_complexity.py": (
        "implemented-different-by-design",
        ["tests/ci/governance_integrity_gate.py", "docs/reference/governance-complexity.md", "docs/reference/governance-integrity-gate.md", "crates/cockpit-repository/src/lib.rs"],
        "Complexity budgets, archive integrity, repository shape, repayment findings, and governance-integrity evidence are retained at the CI/documentation boundary and bound to Rust lifecycle records. The source Python metric implementation is not a Core authority and cannot rewrite history.",
    ),
    "scripts/check_instruction_traceability.py": (
        "implemented-different-by-design",
        ["tests/ci/governance_integrity_gate.py", "docs/reference/instruction-traceability.md", "crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/lib.rs"],
        "Instruction-to-Contract-to-implementation-to-acceptance traceability is provided by typed Work Item contracts, evidence/archive manifests, and the governance integrity gate. Rust preserves explicit locators and fail-closed missing links without adopting the source audit JSON schema.",
    ),
    "scripts/check_pre_release_documentation_alignment.py": (
        "implemented-different-by-design",
        ["tests/docs/documentation_acceptance.sh", "tests/docs/promote_closed_work_item.py", "docs/reference/documentation-authority-boundary.md", "docs/release/distribution.md"],
        "Pre-release documentation alignment is represented by current tri-language metadata/link/claim checks, closed-Work-Item projection promotion, and release documentation gates. Source revision-bound report bytes remain historical evidence rather than Runtime state.",
    ),
    "scripts/check_real_absurd_injection_docs.py": (
        "implemented-different-by-design",
        ["tests/absurd", "tests/conformance", "docs/reference/real-absurd-injection-cases.md", "docs/security/enterprise-governance.md"],
        "Multilingual adversarial/absurd-case documentation and fail-closed trust regressions provide the portable responsibility. Rust keeps case records and refusal evidence explicit, while source Python assessment helpers and their fixed case registry are not copied into Core.",
    ),
    "scripts/check_release_distribution.py": (
        "implemented-different-by-design",
        ["tests/release/adopter_acceptance.sh", "tests/release/adopter_upgrade_acceptance.sh", ".github/workflows/release.yml", "docs/release/distribution.md", "docs/getting-started/installation.md"],
        "Release discovery, immutable tag/archive identity, installer behavior, checksum/SBOM/provenance, public asset validation, and adopter isolation are implemented by Rust release workflows and post-release harnesses. Provider APIs and the source Python distribution checker are not copied or used as Runtime authority.",
    ),
}

WI568_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/check_release_preflight.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", "crates/cockpit-release/src/manifest.rs", ".github/workflows/release.yml", "docs/release/distribution.md"],
        "The source release preflight combines repository cleanliness, source/tag identity, archive digests, lifecycle state, and publication policy. Rust binds these responsibilities to typed release manifests, SHA256/SBOM/provenance validation, Runtime lifecycle gates, and the reviewed release workflow; source report JSON and provider API checks are not copied.",
    ),
    "scripts/check_release_state_consistency.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/manifest.rs", "crates/cockpit-release/src/handoff.rs", ".github/workflows/release.yml", "tests/release/version_consistency.sh", "docs/release/distribution.md"],
        "The source release-state projection consistency checks map to strict typed ReleaseManifest/Handoff records, exact version/tag/commit and metadata digest bindings, checksum inventory, and release workflow consistency checks. Rust does not copy the source release-state JSON or provider bookkeeping model.",
    ),
    "scripts/check_supply_chain.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/sbom.rs", "crates/cockpit-release/src/manifest.rs", ".github/workflows/release.yml", "tests/release/adopter_acceptance.sh", "docs/getting-started/security-release-verification.md"],
        "Supply-chain inventory, archive/binary digests, SPDX binding, provenance and immutable release assets are implemented by the typed Rust release boundary and public/staged adopter acceptance. External signing and provider attestation remain delegated evidence; the source Python checker is not copied.",
    ),
    "scripts/check_system_invariants.py": (
        "implemented-different-by-design",
        ["tests/ci/governance_integrity_gate.py", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-release/src/lib.rs", "tests/docs/documentation_acceptance.sh", "docs/reference/governance-integrity-gate.md"],
        "The source cross-layer invariant audit maps to repository-native governance-integrity, documentation, lifecycle, release-manifest, dependency-lock, SBOM and workflow checks. Rust keeps each authority at its owning boundary and does not copy the source invariant registry or make provider-specific claims.",
    ),
    "scripts/check_trust_layer_docs.py": (
        "implemented-different-by-design",
        ["tests/docs/documentation_acceptance.sh", "tests/docs/parity_status_check.sh", "docs/security/enterprise-governance.md", "docs/architecture/product-boundary.md", "docs/reference/reference-parity.md"],
        "Required trust-layer concepts, architecture links, and tri-language counterparts are checked by the target documentation and parity gates. The source documentation checker is not a Runtime authority; target trust claims remain bounded by typed evidence and explicit external-control limits.",
    ),
    "scripts/cross_stack_long_cycle.py": (
        "reference-only",
        ["tests/release/adopter_acceptance.sh", "tests/release/adopter_upgrade_acceptance.sh", "docs/reference/installed-lifecycle.md"],
        "The source aggregate exercises a template-specific local fixture matrix and external adopter simulation. The target validates immutable published binaries and repository-bound lifecycle independently, but does not promise the source seven-stack matrix or claim provider/enterprise assurance from local fixtures.",
    ),
    "scripts/determine_governance_profile.py": (
        "implemented-different-by-design",
        ["crates/cockpit-verification/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "tests/ci/quality_route.py", "docs/reference/governance-profiles.md", "docs/reference/governance-profile-cost-separation.md"],
        "Changed-path classification, light/standard/strict routing, release escalation, and bounded human override are represented by Rust policy planning and the CI route during the documented shadow-comparison phase. Source YAML profile bytes and source report schema are not copied, and verification tier remains orthogonal to evidence assurance.",
    ),
    "scripts/determine_quality_scope.py": (
        "implemented-different-by-design",
        ["crates/cockpit-verification/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "tests/ci/quality_route.py", "docs/reference/ci-quality-gates.md"],
        "Fast/full quality scope selection maps to the dynamic Rust verification planner and the repository CI route, with reasons, required groups, and release preparation bound to the Contract. The source command and Python route remain a convergence/shadow boundary rather than a copied Runtime API.",
    ),
    "scripts/end_to_end_adoption_validation.py": (
        "reference-only",
        ["tests/release/adopter_acceptance.sh", "tests/release/adopter_upgrade_acceptance.sh", "tests/conformance", "docs/release/distribution.md"],
        "The source seven-project installer matrix is specific to the reference template and its in-process Python Installer. Target release acceptance covers immutable public artifacts, repository attachment, lifecycle, isolation, upgrade and rollback, while adopter stack/toolchain behavior remains adopter-owned and is not silently claimed equivalent.",
    ),
    "scripts/ensure_locked_dev_environment.py": (
        "implemented-different-by-design",
        ["Cargo.lock", "rust-toolchain.toml", ".github/workflows/ci.yml", "docs/reference/ci-quality-gates.md"],
        "The source Python virtual-environment and Ruff hash lock are replaced by Cargo.lock, the pinned Rust toolchain, locked Cargo commands, and pinned CI actions. The target does not install a Python development environment or copy the source provisioning script.",
    ),
    "scripts/external_adopter_long_cycle.py": (
        "implemented-different-by-design",
        ["tests/release/adopter_acceptance.sh", "tests/release/adopter_upgrade_acceptance.sh", "docs/release/distribution.md"],
        "The source provider-simulated long cycle maps to immutable staged/public release adopter acceptance with attach, lifecycle, upgrade, rollback, isolation manifests, cleanup proof and runtime identity. Provider APIs and local fixture assertions remain outside the target Runtime.",
    ),
    "scripts/finalize_release_freeze.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", "crates/cockpit-release/src/manifest.rs", ".github/workflows/release.yml", "docs/release/distribution.md"],
        "Release freeze validation, exact source/tag/commit identity, clean synchronized default branch, archive digest and lifecycle binding are enforced by the reviewed Rust release workflow and typed manifest boundary. Source freeze projection files are not copied into the repository protocol.",
    ),
    "scripts/fixture_harness.py": (
        "reference-only",
        ["tests/conformance", "tests/release/adopter_acceptance.sh", "docs/reference/python-fixture-adaptation.md"],
        "The source deterministic fixture phase harness is a reference-template test driver. Target conformance fixtures and immutable release adopter scripts cover portable negative governance and published-binary behavior, but do not claim the source phase names or stack matrix as a Runtime API.",
    ),
    "scripts/installed_lifecycle_e2e.py": (
        "implemented-different-by-design",
        ["tests/release/adopter_acceptance.sh", "tests/release/adopter_upgrade_acceptance.sh", "docs/reference/installed-lifecycle.md"],
        "Installed lifecycle evidence classification is provided by the published-artifact acceptance harness, which distinguishes real local execution from simulation/not-run and binds runtime/version/digests and cleanup. The source classifier module and report wire are not copied.",
    ),
    "scripts/installer/__init__.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/getting-started/installation.md"],
        "The source installer package seam maps to the Rust shared Runtime's attach and Agent adapter modules. The target keeps installation as a published binary plus explicit repository binding, not a Python package compatibility surface.",
    ),
    "scripts/installer/application.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "Read-only installation planning and typed results map to Rust agent plan/doctor, repository attach/inspect and CLI JSON projections. Conflict and ownership facts are repository-local and no source dataclass or installer manifest is copied.",
    ),
    "scripts/installer/cli.py": (
        "implemented-different-by-design",
        ["crates/cockpit-cli/src/main.rs", "crates/cockpit-agent/src/lib.rs", "docs/getting-started/installation.md"],
        "The source installer CLI presentation seam maps to the Rust CLI's explicit attach, agent install, doctor, repair and detach commands. Human confirmation and conversation UX remain adapter responsibilities; source argparse entrypoints are not wire requirements.",
    ),
    "scripts/installer/confirmation.py": (
        "implemented-different-by-design",
        ["crates/cockpit-core/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "crates/cockpit-agent/src/lib.rs", "docs/reference/agent-workflow.md"],
        "Installation and governance confirmation is represented by identity-bound preflight humanDecisionRequest, explicit authority and Agent adapter ownership checks. Rust never treats a generic yes/approved value as authorization and does not copy the source confirmation dataclass.",
    ),
    "scripts/installer/conflict_matrix.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-agent/tests/install.rs", "docs/reference/installed-lifecycle.md"],
        "Provider target discovery, regular-file/symlink protection, marker conflicts, ownership drift, nested repository and managed-section checks are enforced by the Rust Agent adapter planner/doctor and regression tests. Source conflict names and matrix JSON are not copied.",
    ),
    "scripts/installer/evidence.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-protocol/src/lib.rs", "tests/release/adopter_acceptance.sh", "docs/reference/installed-lifecycle.md"],
        "Installation evidence is represented by typed adapter receipts, repository identity, managed-section digests, Runtime evidence and immutable adopter acceptance manifests. Source installer evidence classes are not a target wire format, and external provider/enterprise assurance remains explicit.",
    ),
}

# WI-572 compares the next maintained installer, quality, adopter, release,
# and claim-boundary seams.  These source paths are Python/Make/provider
# implementations rather than portable Runtime wire contracts.  The target
# preserves the applicable governance responsibilities in its shared Rust
# Runtime, typed agent/release/evidence services, and published-artifact
# acceptance boundaries; source modules are not copied into the target.
WI572_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "scripts/installer/git_state.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/getting-started/installation.md"],
        "The source module re-exports installer Git facts. Rust derives repository identity and snapshots through the shared Runtime and exposes repository-bound Agent planning; the source Python import seam is not a target API.",
    ),
    "scripts/installer/inspection.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "Read-only target inspection is provided by typed Agent doctor/inspect and explicit repository Runtime diagnostics. Source InstallationInspection dataclass and Python path probing are not copied.",
    ),
    "scripts/installer/legacy.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/installed-lifecycle.md", "docs/getting-started/installation.md"],
        "The source legacy installer owns template-local Python/Make migration, ownership, rollback, and provider files. Rust intentionally uses one shared published binary plus explicit attach and Agent adapters; repository state remains local and no source installer is bundled.",
    ),
    "scripts/installer/ownership.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-agent/tests/install.rs", "docs/reference/installed-lifecycle.md"],
        "Managed-path ownership and conflict facts map to Rust adapter ownership records, symlink rejection, and doctor/repair tests. Source Conflict dataclass fields are not a wire contract.",
    ),
    "scripts/installer/planning.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-cli/src/main.rs", "docs/reference/commands.md"],
        "The source InstallationPlan is projected by Rust Agent adapter plans and repository attach output with explicit repository identity and bounded writes; source Python plan objects are not copied.",
    ),
    "scripts/installer/presentation.py": (
        "implemented-different-by-design",
        ["crates/cockpit-cli/src/main.rs", "crates/cockpit-agent/src/lib.rs", "docs/reference/commands.md"],
        "Human installation presentation is a CLI/Agent adapter responsibility in Rust, with stable JSON and localized handoff boundaries; source one-line plan rendering is not a target protocol.",
    ),
    "scripts/installer/rollback.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-agent/tests/install.rs", "docs/reference/installed-lifecycle.md"],
        "Bounded Rust adapter repair/detach retains ownership and rollback evidence and refuses unsafe paths. Source RollbackResult and Python filesystem transaction are not copied into Core.",
    ),
    "scripts/installer/transaction.py": (
        "implemented-different-by-design",
        ["crates/cockpit-agent/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/installed-lifecycle.md"],
        "The source transaction requires explicit Confirmation before writes. Rust enforces explicit attach/Agent ownership, human review, atomic writes, and fail-closed repair; source executor classes are not a Runtime wire format.",
    ),
    "scripts/installer/upgrade.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", "crates/cockpit-agent/src/lib.rs", "docs/reference/installed-lifecycle.md"],
        "Release semver and installed adapter upgrade are handled by the published Rust release boundary and repository-local adapter records. The source re-export seam does not require a copied Python module.",
    ),
    "scripts/quality_measurements.py": (
        "implemented-different-by-design",
        ["crates/cockpit-verification/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "tests/performance", "docs/reference/verification-cost.md"],
        "Identity-bound performance samples, budgets, baseline comparison, and p50/p95 cost observations are Rust verification evidence. Source hosted Python measurement receipt and runner fields remain provider evidence, not target wire compatibility.",
    ),
    "scripts/quality_session_lock.py": (
        "implemented-different-by-design",
        ["crates/cockpit-verification/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/verification-cost.md"],
        "Rust request-scoped verification concurrency and repository lifecycle controls provide bounded execution. Source fcntl/Make-specific lock markers are platform/provider mechanics, not a shared Runtime contract.",
    ),
    "scripts/quality_test_manifest.py": (
        "implemented-different-by-design",
        ["tests/ci/repository_gate_manifest.json", "crates/cockpit-verification/src/lib.rs", "crates/cockpit-repository/src/lib.rs", "docs/reference/ci-quality-gates.md"],
        "The target gate manifest and typed verification receipts bind commands, stages, identity, evidence, and required checks. Source pytest/JUnit/shard manifest fields remain adopter/CI-provider facts and are not copied wholesale.",
    ),
    "scripts/real_adopter_reference_validation.py": (
        "reference-only",
        ["tests/release/adopter_acceptance.sh", "tests/release/adopter_upgrade_acceptance.sh", "docs/release/distribution.md"],
        "This source module is a reference-template-specific seven-project adopter matrix. The target validates immutable public binaries, repository isolation, lifecycle, upgrade, rollback, and cleanup without claiming the source stack matrix or provider/enterprise assurance.",
    ),
    "scripts/release_archive.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/archive.rs", "crates/cockpit-release/src/manifest.rs", "docs/release/distribution.md"],
        "Rust release archive packaging is deterministic, platform-aware, and member-safe with checksum/SBOM/provenance bindings. The source Git-selected Python source archive is a different distribution boundary and is not copied.",
    ),
    "scripts/run_quality_gate.py": (
        "implemented-different-by-design",
        ["crates/cockpit-verification/src/lib.rs", "tests/ci/run_repository_gates.py", "tests/ci/repository_gate_manifest.json", "docs/reference/ci-quality-gates.md"],
        "Rust verification and the reviewed gate manifest own command identity, timeout/failure evidence, Contract bindings, and fail-closed results. The source process-group Python wrapper remains CI orchestration detail rather than a Runtime API.",
    ),
    "scripts/run_quality_session.py": (
        "implemented-different-by-design",
        ["tests/ci/run_repository_gates.py", "tests/ci/quality_route.py", "docs/reference/ci-quality-gates.md"],
        "Dynamic Rust/CI routing and canonical gate execution preserve ordered quality phases and explicit failure retention. Source Make phase runner and Python process-group implementation are not copied.",
    ),
    "scripts/summarize_quality_gates.py": (
        "implemented-different-by-design",
        ["crates/cockpit-verification/src/lib.rs", "tests/performance", "docs/reference/verification-cost.md"],
        "Target cost observations and verification performance reports retain wall/total cost, parallel efficiency, cache/repetition and budget evidence where applicable. Source summary markdown/JSON is not a Rust protocol requirement.",
    ),
    "scripts/sync_published_release_projection.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/manifest.rs", "crates/cockpit-release/src/handoff.rs", ".github/workflows/release.yml", "docs/release/distribution.md"],
        "Published release identity and version projection are bound by typed ReleaseManifest/Handoff, immutable tags, archive digests, and the reviewed workflow. Source candidate/state files and Python atomic projection are not copied.",
    ),
    "scripts/unsupported_claim_gate.py": (
        "implemented-different-by-design",
        ["crates/cockpit-protocol/src/lib.rs", "crates/cockpit-repository/src/governance_controls.rs", "crates/cockpit-repository/src/outcome_render.rs", "docs/reference/outcome-report.md"],
        "Rust typed OutcomeClaim, evidence/inference separation, unknowns, and fail-closed Outcome rendering enforce the same unsupported-claim boundary. Source lexical gate/report format is not a target wire contract.",
    ),
    "scripts/verify_quick_install_release.py": (
        "implemented-different-by-design",
        ["crates/cockpit-release/src/lib.rs", "tests/release/adopter_acceptance.sh", "tests/release/workflow_policy.sh", "docs/getting-started/security-release-verification.md"],
        "Immutable tag/archive discovery, binary and manifest digests, supported-platform checks, and downloaded-artifact verification are provided by Rust release tooling and post-release acceptance. The source quick-install Python checker is not copied.",
    ),
}

# WI-579 compares the remaining maintained template surfaces one file at a
# time.  Agent rules, glossary, and the Make entrypoint have portable
# governance responsibilities represented by the shared Rust Runtime and
# repository-local documentation.  Stack presets are source-template
# convenience commands: adopters own their toolchains and verification argv,
# so these paths remain explicit reference-only boundaries rather than copied
# Runtime code or policy.
WI579_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "templates/agents/AI_COCKPIT_RULES.md": (
        "implemented-different-by-design",
        [
            "AGENTS.md",
            ".ai/README.md",
            ".ai/glossary.md",
            "crates/cockpit-agent/src/lib.rs",
            "docs/reference/agent-workflow.md",
            "docs/reference/agent-workflow.zh-CN.md",
            "docs/reference/agent-workflow.ja.md",
        ],
        "The source Agent rules are preserved as repository-local Rust Runtime and Agent workflow boundaries: explicit repository context, Contract-first review, pause rules, evidence, Outcome, and exact cleanup. The template Markdown surface and its Make commands are not copied into the Runtime.",
    ),
    "templates/glossary.md": (
        "implemented-different-by-design",
        [
            ".ai/glossary.md",
            "docs/reference/commands.md",
            "docs/reference/agent-workflow.md",
        ],
        "The source glossary's governance vocabulary is represented by the maintained repository glossary and reader-facing command/workflow documentation. Its project-specific placeholder domain terms remain adopter-owned and are not fabricated by the shared Runtime.",
    ),
    "templates/make/Makefile.ai": (
        "implemented-different-by-design",
        [
            "crates/cockpit-cli/src/main.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-verification/src/lib.rs",
            "docs/reference/commands.md",
            "docs/reference/ci-quality-gates.md",
        ],
        "The source Make entrypoint's lifecycle, quality, and evidence responsibilities are provided by explicit Rust CLI/Runtime commands and the reviewed gate manifest. Make/Python target names, shell defaults, and source wire formats remain provider or adopter integration choices and are not copied.",
    ),
}

for _stack_name in (
    "android",
    "csharp",
    "flutter",
    "generic",
    "go",
    "java",
    "kotlin",
    "php",
    "python",
    "ruby",
    "rust",
    "swift",
    "typescript",
):
    WI579_REFERENCE_FILES[f"templates/stacks/{_stack_name}.mk"] = (
        "reference-only",
        [
            "docs/getting-started/adopter-configuration.md",
            "docs/reference/ci-quality-gates.md",
            "crates/cockpit-verification/src/lib.rs",
        ],
        "This stack preset supplies source-template command defaults and toolchain assumptions. The shared Rust Runtime keeps verification argv, formatter/linter/test ownership, platform capability, and assurance repository/adopter-owned; it does not copy or infer a stack preset.",
    )

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

WI411_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "examples/fixtures/java-multimodule/.gitignore": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "tests/release/adopter_acceptance.sh",
        ],
        "This fixture-local ignore file only protects a disposable Java/Maven sample checkout. The Rust Runtime does not bundle or attach source fixtures; adopter build directories remain the adopter's responsibility and are exercised only through explicit, isolated release harness boundaries.",
    ),
    "examples/fixtures/java-multimodule/app/src/main/java/fixture/app/Main.java": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "crates/cockpit-verification/src/lib.rs",
            "tests/release/adopter_acceptance.sh",
        ],
        "This is executable Java sample code proving an app-to-core dependency. It is conformance material for the reference repository, not Runtime governance logic; the target can execute adopter-declared argv but does not claim Java-specific support or copy this fixture.",
    ),
    "examples/fixtures/java-multimodule/app/src/test/java/fixture/app/MainTest.java": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "crates/cockpit-verification/src/lib.rs",
            "tests/release/adopter_acceptance.sh",
        ],
        "This dependency-free Java executable test is a source fixture assertion, not a portable Runtime test contract. Rust verification records the command and result supplied by an adopter; it does not ship or infer this Java test.",
    ),
    "examples/fixtures/java-multimodule/core/src/main/java/fixture/core/Decision.java": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "This deterministic policy example belongs to the reference fixture's application domain. It is not an AI Cockpit policy implementation and must not be copied into the Rust Core; repository policy remains explicit and typed in the adopter context.",
    ),
    "examples/fixtures/java-multimodule/core/src/test/java/fixture/core/DecisionTest.java": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "crates/cockpit-verification/src/lib.rs",
        ],
        "This Java unit-style executable test validates the sample Decision class only. The target preserves the evidence boundary through declared verification commands and does not treat a source fixture test as Runtime or enterprise evidence.",
    ),
    "examples/fixtures/java-multimodule/evidence.json": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "tests/release/adopter_acceptance.sh",
            "docs/reference/ci-release-evidence.md",
        ],
        "The source evidence JSON describes one local Java fixture run, including unavailable Maven/provider capabilities. Target release/adopter receipts have stricter repository, Runtime, snapshot, artifact, isolation, and cleanup bindings; source fixture evidence is not imported or promoted.",
    ),
    "examples/fixtures/java-multimodule/fixture.json": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "docs/reference/verification-cost.md",
            "tests/release/adopter_acceptance.sh",
        ],
        "This fixture metadata declares a Java stack, module paths, and platform claims for the source sample. The target keeps project facts repository-local and evidence-bound; a generic Runtime does not infer Java capability or copy stack metadata from this file.",
    ),
    "examples/fixtures/java-multimodule/pom.xml": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "tests/release/adopter_acceptance.sh",
        ],
        "The Maven parent descriptor is an executable sample build input. The Rust Runtime has no Maven dependency and must not copy or mutate adopter build manifests; any Java build is external delegated verification.",
    ),
    "examples/fixtures/java-multimodule/scripts/lifecycle.sh": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "docs/reference/agent-workflow.md",
            "tests/release/adopter_acceptance.sh",
        ],
        "The shell lifecycle script orchestrates a disposable Java fixture, local upgrade/rollback checks, and blocked capability phases. The target expresses governance lifecycle through the installed Rust Runtime and explicit adopter commands; source shell orchestration is not copied or treated as an additional authority.",
    ),
}


WI414_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "examples/fixtures/python/fixture.json": (
        "reference-only",
        [
            "docs/reference/python-fixture-adaptation.md",
            "docs/reference/configuration.md",
            "crates/cockpit-repository/src/lib.rs",
        ],
        "This metadata describes the reference sample's Python stack, local paths, and platform claims. The target keeps project facts repository-local and evidence-bound; the shared Runtime does not infer Python capability or copy fixture metadata.",
    ),
    "examples/fixtures/python/pyproject.toml": (
        "reference-only",
        [
            "docs/reference/python-fixture-adaptation.md",
            "docs/reference/verification-route.md",
            "tests/release/adopter_acceptance.sh",
        ],
        "This is the sample's Python packaging and pytest configuration. It is not a Runtime dependency or installation recipe; an adopter owns its Python environment and supplies an explicit verification command whose result is recorded by the Runtime.",
    ),
    "examples/fixtures/python/src/service.py": (
        "reference-only",
        [
            "docs/reference/python-fixture-adaptation.md",
            "crates/cockpit-verification/src/lib.rs",
        ],
        "This tiny health function is executable application sample code, not governance logic. Rust verification can execute an adopter-declared argv and bind its result to repository evidence, but the target does not ship or infer Python semantics from this file.",
    ),
    "examples/fixtures/python/tests/test_service.py": (
        "reference-only",
        [
            "docs/reference/python-fixture-adaptation.md",
            "crates/cockpit-verification/src/lib.rs",
            "docs/reference/verification-evidence-reuse.md",
        ],
        "This pytest assertion validates only the reference sample's health function. It is fixture evidence, not a portable Runtime test contract or enterprise proof; an adopter must explicitly declare and run its own test command.",
    ),
}


WI432_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "examples/fixtures/typescript-web/.gitignore": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md", "docs/reference/reference-file-comparison.md"],
        "This fixture ignore file protects Node build outputs and local state. The target does not copy it; adopter build hygiene and release-harness isolation remain separate responsibilities.",
    ),
    "examples/fixtures/typescript-web/evidence.json": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md", "docs/reference/verification-evidence-reuse.md"],
        "This source-local evidence describes npm checks and unavailable provider evidence. The target records only explicitly executed, identity-bound commands and never promotes local fixture claims to provider or enterprise assurance.",
    ),
    "examples/fixtures/typescript-web/fixture.json": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md", "crates/cockpit-repository/src/lib.rs"],
        "The metadata declares a TypeScript/web stack, toolchain, platforms, and paths. Project Observer/Profile may record confirmed adopter facts, but the Runtime does not infer capabilities or Contract scope from this fixture.",
    ),
    "examples/fixtures/typescript-web/package-lock.json": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md"],
        "This lockfile pins the fixture's npm dependency and registry integrity. It belongs to the adopter and is not a Runtime dependency or release supply-chain proof.",
    ),
    "examples/fixtures/typescript-web/package.json": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md", "docs/reference/verification-route.md"],
        "The npm manifest defines application build, test, lint, format, and lifecycle scripts. Adopters declare explicit argv in their Contract; the shared governance lifecycle is not replaced by npm orchestration.",
    ),
    "examples/fixtures/typescript-web/scripts/format-check.mjs": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md"],
        "This script checks formatting properties of the sample source only. It is not a portable governance control; an adopter owns its formatter and supplies its own evidence.",
    ),
    "examples/fixtures/typescript-web/scripts/lifecycle.mjs": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md", "docs/reference/agent-workflow.md"],
        "The Node script exercises install, configuration, blocked requests, upgrade, rollback, and release phases for the fixture. The installed Rust Runtime supplies repository-bound governance, recovery, and Outcome; source orchestration is not copied or treated as authority.",
    ),
    "examples/fixtures/typescript-web/scripts/lint.mjs": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md"],
        "This lint rule is coupled to the sample application's symbols and is not portable Runtime policy. An adopter declares and verifies its own lint command.",
    ),
    "examples/fixtures/typescript-web/src/index.ts": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md", "docs/reference/intent-scenario-binding.md"],
        "The TypeScript evaluator is application sample code. Runtime decisions, stop states, and intent/scenario bindings are typed governance records and are never inferred by importing this source.",
    ),
    "examples/fixtures/typescript-web/test/index.test.mjs": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md", "docs/reference/verification-evidence-reuse.md"],
        "These Node tests assert only the fixture evaluator's behavior. They are not Runtime or enterprise evidence; an adopter must explicitly run its own tests and bind their result.",
    ),
    "examples/fixtures/typescript-web/tsconfig.json": (
        "reference-only",
        ["docs/reference/typescript-fixture-adaptation.md"],
        "The strict NodeNext compiler settings are adopter-owned toolchain configuration. The shared Runtime accepts explicit command results but does not promise or copy a TypeScript toolchain.",
    ),
}


WI368_REFERENCE_FILES: dict[str, tuple[str, list[str], str]] = {
    "docs/reference/pre-release-documentation-alignment.md": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "This generated source alignment report is historical review evidence. The target uses its own repository-local documentation gates and must not promote source Work Item history or generated alignment bytes as current release authority.",
    ),
    "docs/reference/pre-release-documentation-review.json": (
        "reference-only",
        [
            "docs/reference/reference-file-comparison.md",
            "docs/reference/reference-parity.md",
            "tests/docs/documentation_acceptance.sh",
        ],
        "This source five-strategy review record is immutable historical assessment. Target documentation truth is re-evaluated from current files and Runtime evidence; source findings and self-declared status cannot authorize a target release.",
    ),
    "docs/reference/project-test-timing-baseline.json": (
        "implemented-different-by-design",
        [
            "docs/reference/governance-performance-budget.md",
            "docs/reference/governance-performance-budget.zh-CN.md",
            "docs/reference/governance-performance-budget.ja.md",
            "docs/reference/performance-diagnosis.md",
            "crates/cockpit-verification/src/lib.rs",
            "tests/performance/regression_gate.sh",
        ],
        "The source timing seed is mapped to identity-bound Rust PerformanceBaseline samples and advisory regression budgets. Measurements inform scheduling/cost only; they never authorize a weaker verification route and source timings are not copied.",
    ),
    "docs/reference/provider-backed-governance-validation.md": (
        "implemented-different-by-design",
        [
            "docs/reference/ci-release-evidence.md",
            "docs/reference/ci-release-evidence.zh-CN.md",
            "docs/reference/ci-release-evidence.ja.md",
            "docs/security/enterprise-governance.md",
            "docs/security/enterprise-governance.zh-CN.md",
            "docs/security/enterprise-governance.ja.md",
        ],
        "Provider configuration, branch protection, reviewer identity, and hosted controls remain delegated evidence. The target binds and displays external records without claiming that local Rust or CI checks prove provider or enterprise governance.",
    ),
    "docs/reference/real-absurd-injection-cases.md": (
        "implemented-different-by-design",
        [
            "docs/security/adversarial-validation.md",
            "tests/adversarial/manifest.json",
            "crates/cockpit-core/tests/adversarial_v2.rs",
        ],
        "The target preserves the semantic 15-case wording corpus and twelve named RAI scenarios through a Rust-native manifest and adversarial tests. Source narrative and wording variants are not Runtime authority; source language files disagree on the named-case count, so the manifest is canonical.",
    ),
    "docs/reference/real-absurd-injection-cases.zh-CN.md": (
        "implemented-different-by-design",
        [
            "docs/security/adversarial-validation.zh-CN.md",
            "tests/adversarial/manifest.json",
            "crates/cockpit-core/tests/adversarial_v2.rs",
        ],
        "中文 source 语义由 Rust 的三语 adversarial 文档、manifest 与回归测试承接；保留 15 个结构化 wording cases 与 12 个命名 RAI cases 的 canonical manifest，不复制源 prose 或把语言差异当作 capability。",
    ),
    "docs/reference/real-absurd-injection-cases.ja.md": (
        "implemented-different-by-design",
        [
            "docs/security/adversarial-validation.ja.md",
            "tests/adversarial/manifest.json",
            "crates/cockpit-core/tests/adversarial_v2.rs",
        ],
        "Japanese source semantics are projected to the Rust tri-language adversarial pages, canonical manifest, and regression tests. The target preserves the 15 structured wording cases and twelve named RAI cases without copying source narrative or claiming general language fluency.",
    ),
    "docs/reference/real-adopter-reference-validation.md": (
        "implemented-different-by-design",
        [
            "docs/release/distribution.md",
            "docs/release/distribution.zh-CN.md",
            "docs/release/distribution.ja.md",
            "tests/release/adopter_acceptance.sh",
            "tests/release/adopter_upgrade_acceptance.sh",
        ],
        "Disposable reference-clone validation is represented by the immutable public Release adopter and upgrade harness. The target records artifact, binary, repository, lifecycle, isolation, and cleanup evidence; provider identity, hosted review, and enterprise assurance remain external.",
    ),
    "docs/reference/reference-impact-gate.md": (
        "reference-only",
        [
            "docs/reference/operation-time-policy-reevaluation.md",
            "docs/reference/governance-profiles.md",
            "docs/reference/reference-parity.md",
        ],
        "The source static reference-impact scanner, schema, and Make/Python commands are not present in the Rust Runtime. Operation-time policy still evaluates declared operation, target, scope, authority, freshness, trust, and impact, but it does not infer callers or external consumers; this is an explicit bounded gap, not an equivalence claim.",
    ),
    "docs/reference/reference-impact-gate.zh-CN.md": (
        "reference-only",
        [
            "docs/reference/operation-time-policy-reevaluation.zh-CN.md",
            "docs/reference/governance-profiles.zh-CN.md",
            "docs/reference/reference-parity.zh-CN.md",
        ],
        "源静态 reference-impact scanner、schema 和 Make/Python 命令未迁入 Rust Runtime。操作时策略仍评估已声明的 operation、target、scope、authority、freshness、trust 和 impact，但不推导 callers 或 external consumers；这是明确的有界 gap，不是对等实现声明。",
    ),
    "docs/reference/reference-impact-gate.ja.md": (
        "reference-only",
        [
            "docs/reference/operation-time-policy-reevaluation.ja.md",
            "docs/reference/governance-profiles.ja.md",
            "docs/reference/reference-parity.ja.md",
        ],
        "Reference source の static impact scanner、schema、Make/Python command は Rust Runtime に移植していません。Operation-time policy は宣言された operation、target、scope、authority、freshness、trust、impact を評価しますが、caller や external consumer を推論しません。これは明示的な bounded gap であり、同等実装の主張ではありません。",
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


def git_changed_paths(repository: Path, previous_revision: str, revision: str) -> set[str]:
    """Return paths whose source bytes or tracked presence changed between commits."""
    for value in (previous_revision, revision):
        if len(value) != 40 or any(character not in "0123456789abcdef" for character in value):
            raise ValueError(f"revision must be a full lowercase commit digest: {value!r}")
    result = subprocess.run(
        [
            "git",
            "-C",
            str(repository),
            "diff",
            "--name-only",
            f"{previous_revision}..{revision}",
            "--",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return {line for line in result.stdout.splitlines() if line}


def digest_paths(paths: list[str]) -> str:
    payload = "\n".join(sorted(paths)) + "\n"
    return "sha256:" + hashlib.sha256(payload.encode()).hexdigest()


def digest_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


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
        wi437 = WI437_REFERENCE_FILES.get(path)
        if wi437 is not None:
            classification, counterparts, reason = wi437
            records.append(
                {
                    "referencePath": path,
                    "batch": WI437_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi441 = WI441_REFERENCE_FILES.get(path)
        if wi441 is not None:
            classification, counterparts, reason = wi441
            records.append(
                {
                    "referencePath": path,
                    "batch": WI441_BATCH,
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
        wi411 = WI411_REFERENCE_FILES.get(path)
        if wi411 is not None:
            classification, counterparts, reason = wi411
            records.append(
                {
                    "referencePath": path,
                    "batch": WI411_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi414 = WI414_REFERENCE_FILES.get(path)
        if wi414 is not None:
            classification, counterparts, reason = wi414
            records.append(
                {
                    "referencePath": path,
                    "batch": WI414_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi432 = WI432_REFERENCE_FILES.get(path)
        if wi432 is not None:
            classification, counterparts, reason = wi432
            records.append(
                {
                    "referencePath": path,
                    "batch": WI432_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi368 = WI368_REFERENCE_FILES.get(path)
        if wi368 is not None:
            classification, counterparts, reason = wi368
            records.append(
                {
                    "referencePath": path,
                    "batch": WI368_BATCH,
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
        wi496 = WI496_REFERENCE_FILES.get(path)
        if wi496 is not None:
            classification, counterparts, reason = wi496
            records.append(
                {
                    "referencePath": path,
                    "batch": WI496_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi504 = WI504_REFERENCE_FILES.get(path)
        if wi504 is not None:
            classification, counterparts, reason = wi504
            records.append(
                {
                    "referencePath": path,
                    "batch": WI504_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi507 = WI507_REFERENCE_FILES.get(path)
        if wi507 is not None:
            classification, counterparts, reason = wi507
            records.append(
                {
                    "referencePath": path,
                    "batch": WI507_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi508 = WI508_REFERENCE_FILES.get(path)
        if wi508 is not None:
            classification, counterparts, reason = wi508
            records.append(
                {
                    "referencePath": path,
                    "batch": WI508_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi548 = WI548_REFERENCE_FILES.get(path)
        if wi548 is not None:
            classification, counterparts, reason = wi548
            records.append(
                {
                    "referencePath": path,
                    "batch": WI548_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi550 = WI550_REFERENCE_FILES.get(path)
        if wi550 is not None:
            classification, counterparts, reason = wi550
            records.append(
                {
                    "referencePath": path,
                    "batch": WI550_BATCH,
                    "classification": classification,
                    "rustCounterparts": counterparts,
                    "reason": reason,
                }
            )
            continue
        wi552 = WI552_REFERENCE_FILES.get(path)
        if wi552 is not None:
            classification, counterparts, reason = wi552
            records.append(
                {
                    "referencePath": path,
                    "batch": WI552_BATCH,
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
        "referenceRepository": "local-git-checkout",
        "referencePathEnv": "AI_COCKPIT_REFERENCE_ROOT",
        "referenceNetworkAccess": False,
        "referenceCommit": source_commit,
        "targetRepository": "https://github.com/xinglun/ai-cockpit",
        "targetCommit": target_commit,
        "referenceTrackedFileCount": len(reference_paths),
        "referenceChangedPathCount": 0,
        "referenceChangedPaths": [],
        "retiredReferencePathCount": 0,
        "retiredReferencePaths": [],
        "targetTrackedFileCount": len(target_commit_paths),
        "targetTrackedPathDigest": digest_paths(target_commit_paths),
        "targetWorkingTreeFileCount": len(target_paths),
        "targetWorkingTreePathDigest": digest_paths(target_paths),
        "allowedClassifications": sorted(ALLOWED_CLASSIFICATIONS),
        "records": records,
    }


def rebaseline(
    previous_manifest_path: Path,
    reference: Path,
    target: Path,
    source_commit: str,
    target_commit: str,
) -> dict[str, Any]:
    """Rebind an existing ledger without silently dropping prior decisions.

    A source checkout is allowed to remove or revise files.  Current records
    are therefore projected from the previous ledger, while removed paths are
    retained as compact historical records and changed non-history paths are
    made explicitly deferred until a later semantic comparison batch reviews
    the new source bytes.
    """
    previous = json.loads(previous_manifest_path.read_text(encoding="utf-8"))
    previous_records = previous.get("records")
    if not isinstance(previous_records, list) or not previous_records:
        raise ValueError("previous manifest must contain a non-empty records list")
    previous_source = previous.get("referenceCommit")
    if not isinstance(previous_source, str):
        raise ValueError("previous manifest is missing referenceCommit")
    previous_by_path: dict[str, dict[str, Any]] = {}
    for record in previous_records:
        if not isinstance(record, dict) or not isinstance(record.get("referencePath"), str):
            raise ValueError("previous manifest contains an invalid record")
        previous_by_path[record["referencePath"]] = record

    current_paths = git_paths(reference, source_commit)
    current_set = set(current_paths)
    changed_paths = git_changed_paths(reference, previous_source, source_commit)
    retired_paths = sorted(set(previous_by_path) - current_set)
    # Preserve the complete prior ledger in ``records``.  Retired source paths
    # remain available for historical audit, while ``retiredReferencePaths``
    # explicitly removes them from the current baseline.  This avoids a large
    # destructive JSON diff and, more importantly, prevents prior decisions
    # from silently disappearing when the reference checkout changes.
    current_records: list[dict[str, Any]] = [copy.deepcopy(record) for record in previous_records]
    records_by_path = {
        record["referencePath"]: record
        for record in current_records
    }
    for path in current_paths:
        previous_record = records_by_path.get(path)
        if previous_record is None:
            record = {
                "referencePath": path,
                "batch": "rebaseline-delta",
                "classification": "deferred-next-batch",
                "rustCounterparts": [],
                "reason": (
                    f"New path at local reference commit {source_commit}; "
                    "semantic comparison is required before any parity claim."
                ),
                "sourceChangedSincePrevious": True,
            }
            current_records.append(record)
            records_by_path[path] = record
            continue
        record = previous_record
        if path in changed_paths:
            record["sourceChangedSincePrevious"] = True
            if record.get("classification") != "generated-history":
                record["previousBatch"] = record.get("batch")
                record["previousClassification"] = record.get("classification")
                record["rebaselineBatch"] = "rebaseline-delta"
                record["classification"] = "deferred-next-batch"
                record["rustCounterparts"] = record.get("rustCounterparts", [])
                record["reason"] = (
                    f"Source path changed between {previous_source} and {source_commit}; "
                    "the previous decision is retained as history and must be re-reviewed."
                )
        else:
            # Do not add a per-record false marker.  The changed-path index is
            # the authoritative set; omitting the default keeps this large
            # machine ledger reviewable while preserving an explicit marker
            # for every path whose source bytes changed.
            record.pop("sourceChangedSincePrevious", None)
    retired_records: list[str] = []
    for path in retired_paths:
        record = records_by_path[path]
        # A retired record is historical, not part of the current changed set.
        record.pop("sourceChangedSincePrevious", None)
        # The complete prior record remains in ``records``.  A compact path
        # index plus previousManifestGitRevision/digest is sufficient to bind
        # the historical bytes without duplicating 669 verbose objects.
        retired_records.append(path)

    target_paths = git_paths(target, target_commit)
    return {
        "schemaVersion": 1,
        "referenceRepository": "local-git-checkout",
        "referencePathEnv": "AI_COCKPIT_REFERENCE_ROOT",
        "referenceNetworkAccess": False,
        "referenceCommit": source_commit,
        "previousReferenceCommit": previous_source,
        "previousManifestGitRevision": target_commit,
        "previousManifestDigest": digest_bytes(previous_manifest_path),
        "referenceTrackedFileCount": len(current_paths),
        "recordsIncludeRetiredHistory": True,
        "referenceChangedPathCount": len(changed_paths & current_set),
        "referenceChangedPaths": sorted(changed_paths & current_set),
        "retiredReferencePathCount": len(retired_records),
        "retiredReferencePaths": retired_records,
        "retiredReferenceCommit": previous_source,
        "targetRepository": "https://github.com/xinglun/ai-cockpit",
        "targetCommit": target_commit,
        "targetTrackedFileCount": len(target_paths),
        "targetTrackedPathDigest": digest_paths(target_paths),
        "targetWorkingTreeFileCount": len(target_paths),
        "targetWorkingTreePathDigest": digest_paths(target_paths),
        "allowedClassifications": sorted(ALLOWED_CLASSIFICATIONS),
        "records": current_records,
    }


def historical_classification(record: dict[str, Any]) -> str | None:
    """Use the prior batch decision only for structural history checks."""
    if record.get("sourceChangedSincePrevious") and record.get("previousClassification"):
        return record.get("previousClassification")
    return record.get("classification")


def is_commit_digest(value: Any) -> bool:
    return isinstance(value, str) and len(value) == 40 and all(
        character in "0123456789abcdef" for character in value
    )


def validate(manifest: dict[str, Any], expected_source: str, expected_target: str) -> list[str]:
    errors: list[str] = []
    superseding_wi496_paths = set(WI496_REFERENCE_FILES) if expected_source == EXPECTED_REFERENCE_COMMIT else set()
    if manifest.get("schemaVersion") != 1:
        errors.append("schemaVersion must be 1")
    if manifest.get("referenceRepository") != "local-git-checkout":
        errors.append("referenceRepository must identify the local checkout policy")
    if manifest.get("referencePathEnv") != "AI_COCKPIT_REFERENCE_ROOT":
        errors.append("referencePathEnv must be AI_COCKPIT_REFERENCE_ROOT")
    if manifest.get("referenceNetworkAccess") is not False:
        errors.append("referenceNetworkAccess must be false")
    if manifest.get("referenceCommit") != expected_source:
        errors.append("referenceCommit is not the pinned source commit")
    if manifest.get("previousReferenceCommit") is not None and not is_commit_digest(
        manifest.get("previousReferenceCommit")
    ):
        errors.append("previousReferenceCommit must be a full lowercase commit digest")
    if manifest.get("previousManifestGitRevision") is not None and not is_commit_digest(
        manifest.get("previousManifestGitRevision")
    ):
        errors.append("previousManifestGitRevision must be a full lowercase commit digest")
    previous_manifest_digest = manifest.get("previousManifestDigest")
    if previous_manifest_digest is not None and (
        not isinstance(previous_manifest_digest, str)
        or not previous_manifest_digest.startswith("sha256:")
        or len(previous_manifest_digest) != len("sha256:") + 64
    ):
        errors.append("previousManifestDigest must be a sha256 digest")
    if manifest.get("targetCommit") != expected_target:
        errors.append("targetCommit is not the pinned target baseline")
    if manifest.get("targetWorkingTreeFileCount") != manifest.get("targetTrackedFileCount"):
        errors.append("target working-tree count is not normalized to the pinned commit")
    if manifest.get("targetWorkingTreePathDigest") != manifest.get("targetTrackedPathDigest"):
        errors.append("target working-tree digest is not normalized to the pinned commit")
    records = manifest.get("records")
    if not isinstance(records, list) or not records:
        return errors + ["records must be a non-empty list"]
    # Later bounded rebaseline batches own the current decision for their
    # paths while preserving earlier batch records as immutable history.
    superseded_by_wi494 = set(WI494_REFERENCE_FILES) if expected_source == EXPECTED_REFERENCE_COMMIT else set()
    tracked_paths = manifest.get("referenceTrackedPaths")
    if tracked_paths is not None:
        if not isinstance(tracked_paths, list) or any(not isinstance(path, str) for path in tracked_paths):
            errors.append("referenceTrackedPaths must be a list of strings")
            tracked_paths = None
        elif len(tracked_paths) != len(set(tracked_paths)):
            errors.append("referenceTrackedPaths contains duplicates")
    retired_paths = manifest.get("retiredReferencePaths", [])
    if not isinstance(retired_paths, list):
        errors.append("retiredReferencePaths must be a list")
        retired_paths = []
    for index, retired in enumerate(retired_paths):
        if isinstance(retired, str):
            if not retired:
                errors.append(f"retiredReferencePaths[{index}] must not be empty")
            continue
        if not isinstance(retired, dict) or not isinstance(retired.get("referencePath"), str):
            errors.append(f"retiredReferencePaths[{index}] missing referencePath")
            continue
        if not retired.get("lastSeenCommit") or not retired.get("previousClassification"):
            errors.append(f"retiredReferencePaths[{index}] is missing historical identity")
    retired_names = {
        retired if isinstance(retired, str) else retired.get("referencePath")
        for retired in retired_paths
        if isinstance(retired, str) or isinstance(retired, dict)
    }
    if manifest.get("retiredReferenceCommit") is not None and not is_commit_digest(
        manifest.get("retiredReferenceCommit")
    ):
        errors.append("retiredReferenceCommit must be a full lowercase commit digest")
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
        historical = historical_classification(record)
        if classification not in ALLOWED_CLASSIFICATIONS:
            errors.append(f"{path}: invalid classification {classification!r}")
        if not isinstance(record.get("reason"), str) or not record["reason"].strip():
            errors.append(f"{path}: missing reason")
        if not isinstance(record.get("rustCounterparts"), list):
            errors.append(f"{path}: rustCounterparts must be a list")
        if record.get("batch") == FIRST_BATCH:
            if classification == "deferred-next-batch" and historical == "deferred-next-batch":
                errors.append(f"{path}: first-batch file cannot be deferred")
            if not record.get("rustCounterparts") and historical not in {
                "reference-only",
                "not-applicable",
                "migrate-gap",
            }:
                errors.append(f"{path}: first-batch record needs a counterpart or explicit boundary classification")
        if record.get("batch") == GETTING_STARTED_BATCH:
            if classification == "deferred-next-batch" and historical == "deferred-next-batch":
                errors.append(f"{path}: getting-started file cannot remain deferred")
            if not record.get("rustCounterparts") and historical not in {
                "reference-only",
                "not-applicable",
                "migrate-gap",
            }:
                errors.append(f"{path}: getting-started record needs a counterpart or explicit gap")
    current_record_paths = paths - retired_names
    if tracked_paths is not None and current_record_paths != set(tracked_paths):
        errors.append("referenceTrackedPaths does not match non-retired record paths")
    if manifest.get("referenceTrackedFileCount") != len(current_record_paths):
        errors.append("referenceTrackedFileCount does not match non-retired record paths")
    if not retired_names <= paths:
        errors.append("retiredReferencePaths must have a preserved record")
    if manifest.get("retiredReferencePathCount") is not None and manifest.get("retiredReferencePathCount") != len(retired_paths):
        errors.append("retiredReferencePathCount does not match retiredReferencePaths")
    current_reference_paths = (
        set(tracked_paths) if tracked_paths is not None else paths - retired_names
    )
    # All conformance-batch assertions below describe the current source
    # baseline.  Historical retired records were already structurally checked
    # above and must not inflate current batch counts.
    records = [
        record
        for record in records
        if isinstance(record, dict) and record.get("referencePath") in current_reference_paths
    ]
    changed_paths = manifest.get("referenceChangedPaths", [])
    if not isinstance(changed_paths, list) or any(not isinstance(path, str) for path in changed_paths):
        errors.append("referenceChangedPaths must be a list of strings")
        changed_paths = []
    elif len(changed_paths) != len(set(changed_paths)):
        errors.append("referenceChangedPaths contains duplicates")
    changed_path_set = set(changed_paths)
    if not changed_path_set <= current_reference_paths:
        errors.append("referenceChangedPaths must be a subset of current reference paths")
    if manifest.get("referenceChangedPathCount") is not None and manifest.get("referenceChangedPathCount") != len(changed_paths):
        errors.append("referenceChangedPathCount does not match referenceChangedPaths")
    changed_records = {
        record.get("referencePath")
        for record in records
        if isinstance(record, dict) and record.get("sourceChangedSincePrevious") is True
    }
    if changed_records != changed_path_set:
        errors.append("sourceChangedSincePrevious records do not match referenceChangedPaths")
    if expected_source == EXPECTED_REFERENCE_COMMIT:
        wi437_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI437_BATCH
        ]
        expected_wi437_paths = set(WI437_REFERENCE_FILES) & current_reference_paths
        actual_wi437_paths = {
            record.get("referencePath") for record in wi437_records
        }
        if actual_wi437_paths != expected_wi437_paths:
            errors.append("WI-437 reference rebaseline records do not match the seven scoped paths")
        for record in wi437_records:
            if record.get("classification") != "implemented-different-by-design":
                errors.append(f"{record.get('referencePath')}: WI-437 must be implemented-different-by-design")
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(f"{record.get('referencePath')}: WI-437 result needs counterparts and reason")
        wi441_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI441_BATCH
        ]
        expected_wi441_paths = set(WI441_REFERENCE_FILES) & current_reference_paths
        actual_wi441_paths = {
            record.get("referencePath") for record in wi441_records
        }
        if actual_wi441_paths != expected_wi441_paths:
            errors.append(
                "WI-441 reference entrypoint records do not match the nine scoped paths: "
                f"expected {sorted(expected_wi441_paths)!r}, got {sorted(actual_wi441_paths)!r}"
            )
        if len(wi441_records) != len(expected_wi441_paths):
            errors.append(
                f"WI-441 batch must contain {len(expected_wi441_paths)} records, found {len(wi441_records)}"
            )
        for record in wi441_records:
            if record.get("classification") != "implemented-different-by-design":
                errors.append(f"{record.get('referencePath')}: WI-441 must be implemented-different-by-design")
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(f"{record.get('referencePath')}: WI-441 result needs counterparts and reason")
        wi461_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI461_BATCH
            and record.get("referencePath") in WI461_REFERENCE_FILES
        ]
        expected_wi461_paths = set(WI461_REFERENCE_FILES) & current_reference_paths
        actual_wi461_paths = {record.get("referencePath") for record in wi461_records}
        if actual_wi461_paths != expected_wi461_paths:
            errors.append(
                "WI-461 onboarding rebaseline records do not match the nine scoped paths: "
                f"expected {sorted(expected_wi461_paths)!r}, got {sorted(actual_wi461_paths)!r}"
            )
        if len(wi461_records) != len(expected_wi461_paths):
            errors.append(
                f"WI-461 batch must contain {len(expected_wi461_paths)} records, found {len(wi461_records)}"
            )
        for record in wi461_records:
            if record.get("classification") != "implemented-different-by-design":
                errors.append(f"{record.get('referencePath')}: WI-461 must be implemented-different-by-design")
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(f"{record.get('referencePath')}: WI-461 result needs counterparts and reason")
        wi464_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI464_BATCH
            and record.get("referencePath") in WI464_REFERENCE_FILES
        ]
        expected_wi464_paths = set(WI464_REFERENCE_FILES) & current_reference_paths
        actual_wi464_paths = {record.get("referencePath") for record in wi464_records}
        if actual_wi464_paths != expected_wi464_paths:
            errors.append(
                "WI-464 workflow/build records do not match the four scoped paths: "
                f"expected {sorted(expected_wi464_paths)!r}, got {sorted(actual_wi464_paths)!r}"
            )
        if len(wi464_records) != len(expected_wi464_paths):
            errors.append(
                f"WI-464 batch must contain {len(expected_wi464_paths)} records, found {len(wi464_records)}"
            )
        for record in wi464_records:
            if record.get("classification") != "implemented-different-by-design":
                errors.append(f"{record.get('referencePath')}: WI-464 must be implemented-different-by-design")
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(f"{record.get('referencePath')}: WI-464 result needs counterparts and reason")
        wi475_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI475_BATCH
            and record.get("referencePath") in WI475_REFERENCE_FILES
        ]
        expected_wi475_paths = set(WI475_REFERENCE_FILES) & current_reference_paths
        actual_wi475_paths = {record.get("referencePath") for record in wi475_records}
        if actual_wi475_paths != expected_wi475_paths:
            errors.append(
                "WI-475 Outcome/events/quality records do not match the seven scoped paths: "
                f"expected {sorted(expected_wi475_paths)!r}, got {sorted(actual_wi475_paths)!r}"
            )
        if len(wi475_records) != len(expected_wi475_paths):
            errors.append(
                f"WI-475 batch must contain {len(expected_wi475_paths)} records, found {len(wi475_records)}"
            )
        for record in wi475_records:
            if record.get("classification") != "implemented-different-by-design":
                errors.append(f"{record.get('referencePath')}: WI-475 must be implemented-different-by-design")
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(f"{record.get('referencePath')}: WI-475 result needs counterparts and reason")
        wi482_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI482_BATCH
            and record.get("referencePath") in WI482_REFERENCE_FILES
        ]
        expected_wi482_paths = set(WI482_REFERENCE_FILES) & current_reference_paths
        actual_wi482_paths = {record.get("referencePath") for record in wi482_records}
        if actual_wi482_paths != expected_wi482_paths:
            errors.append(
                "WI-482 lifecycle/trust records do not match the eight scoped paths: "
                f"expected {sorted(expected_wi482_paths)!r}, got {sorted(actual_wi482_paths)!r}"
            )
        if len(wi482_records) != len(expected_wi482_paths):
            errors.append(
                f"WI-482 batch must contain {len(expected_wi482_paths)} records, found {len(wi482_records)}"
            )
        for record in wi482_records:
            if record.get("classification") != "implemented-different-by-design":
                errors.append(f"{record.get('referencePath')}: WI-482 must be implemented-different-by-design")
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(f"{record.get('referencePath')}: WI-482 result needs counterparts and reason")
        wi494_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI494_BATCH
            and record.get("referencePath") in WI494_REFERENCE_FILES
        ]
        expected_wi494_paths = set(WI494_REFERENCE_FILES) & current_reference_paths
        actual_wi494_paths = {record.get("referencePath") for record in wi494_records}
        if actual_wi494_paths != expected_wi494_paths:
            errors.append(
                "WI-494 capability/comprehension/deprecation records do not match the seven scoped paths: "
                f"expected {sorted(expected_wi494_paths)!r}, got {sorted(actual_wi494_paths)!r}"
            )
        if len(wi494_records) != len(expected_wi494_paths):
            errors.append(
                f"WI-494 batch must contain {len(expected_wi494_paths)} records, found {len(wi494_records)}"
            )
        for record in wi494_records:
            if record.get("classification") != "reference-only":
                errors.append(f"{record.get('referencePath')}: WI-494 must be reference-only")
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(f"{record.get('referencePath')}: WI-494 result needs counterparts and reason")
        wi496_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI496_BATCH
            and record.get("referencePath") in WI496_REFERENCE_FILES
        ]
        expected_wi496_paths = set(WI496_REFERENCE_FILES) & current_reference_paths
        actual_wi496_paths = {record.get("referencePath") for record in wi496_records}
        if actual_wi496_paths != expected_wi496_paths:
            errors.append(
                "WI-496 distribution/profile/multilingual records do not match the ten scoped paths: "
                f"expected {sorted(expected_wi496_paths)!r}, got {sorted(actual_wi496_paths)!r}"
            )
        if len(wi496_records) != len(expected_wi496_paths):
            errors.append(
                f"WI-496 batch must contain {len(expected_wi496_paths)} records, found {len(wi496_records)}"
            )
        for record in wi496_records:
            expected_classification = WI496_REFERENCE_FILES[record["referencePath"]][0]
            if record.get("classification") != expected_classification:
                errors.append(
                    f"{record.get('referencePath')}: WI-496 classification must be {expected_classification}"
                )
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(f"{record.get('referencePath')}: WI-496 result needs counterparts and reason")
        if any(
            isinstance(record, dict) and record.get("batch") == WI504_BATCH
            for record in records
        ):
            wi504_records = [
                record
                for record in records
                if isinstance(record, dict)
                and record.get("batch") == WI504_BATCH
                and record.get("referencePath") in WI504_REFERENCE_FILES
            ]
            expected_wi504_paths = set(WI504_REFERENCE_FILES) & current_reference_paths
            actual_wi504_paths = {record.get("referencePath") for record in wi504_records}
            if actual_wi504_paths != expected_wi504_paths:
                errors.append(
                    "WI-504 reference documentation records do not match the five scoped paths: "
                    f"expected {sorted(expected_wi504_paths)!r}, got {sorted(actual_wi504_paths)!r}"
                )
            if len(wi504_records) != len(expected_wi504_paths):
                errors.append(
                    f"WI-504 batch must contain {len(expected_wi504_paths)} records, found {len(wi504_records)}"
                )
            for record in wi504_records:
                expected_classification = WI504_REFERENCE_FILES[record["referencePath"]][0]
                if record.get("classification") != expected_classification:
                    errors.append(
                        f"{record.get('referencePath')}: WI-504 classification must be {expected_classification}"
                    )
                if not record.get("rustCounterparts") or not record.get("reason"):
                    errors.append(f"{record.get('referencePath')}: WI-504 result needs counterparts and reason")
        if any(
            isinstance(record, dict) and record.get("batch") == WI507_BATCH
            for record in records
        ):
            wi507_records = [
                record
                for record in records
                if isinstance(record, dict)
                and record.get("batch") == WI507_BATCH
                and record.get("referencePath") in WI507_REFERENCE_FILES
            ]
            expected_wi507_paths = set(WI507_REFERENCE_FILES) & current_reference_paths
            actual_wi507_paths = {record.get("referencePath") for record in wi507_records}
            if actual_wi507_paths != expected_wi507_paths:
                errors.append(
                    "WI-507 reference example records do not match the five scoped paths: "
                    f"expected {sorted(expected_wi507_paths)!r}, got {sorted(actual_wi507_paths)!r}"
                )
            if len(wi507_records) != len(expected_wi507_paths):
                errors.append(
                    f"WI-507 batch must contain {len(expected_wi507_paths)} records, found {len(wi507_records)}"
                )
            for record in wi507_records:
                expected_classification = WI507_REFERENCE_FILES[record["referencePath"]][0]
                if record.get("classification") != expected_classification:
                    errors.append(
                        f"{record.get('referencePath')}: WI-507 classification must be {expected_classification}"
                    )
                if not record.get("rustCounterparts") or not record.get("reason"):
                    errors.append(f"{record.get('referencePath')}: WI-507 result needs counterparts and reason")
        if any(
            isinstance(record, dict) and record.get("batch") == WI508_BATCH
            for record in records
        ):
            wi508_records = [
                record
                for record in records
                if isinstance(record, dict)
                and record.get("batch") == WI508_BATCH
                and record.get("referencePath") in WI508_REFERENCE_FILES
            ]
            expected_wi508_paths = set(WI508_REFERENCE_FILES) & current_reference_paths
            actual_wi508_paths = {record.get("referencePath") for record in wi508_records}
            if actual_wi508_paths != expected_wi508_paths:
                errors.append(
                    "WI-508 reference example records do not match the five scoped paths: "
                    f"expected {sorted(expected_wi508_paths)!r}, got {sorted(actual_wi508_paths)!r}"
                )
            if len(wi508_records) != len(expected_wi508_paths):
                errors.append(
                    f"WI-508 batch must contain {len(expected_wi508_paths)} records, found {len(wi508_records)}"
                )
            for record in wi508_records:
                expected_classification = WI508_REFERENCE_FILES[record["referencePath"]][0]
                if record.get("classification") != expected_classification:
                    errors.append(
                        f"{record.get('referencePath')}: WI-508 classification must be {expected_classification}"
                    )
                if not record.get("rustCounterparts") or not record.get("reason"):
                    errors.append(f"{record.get('referencePath')}: WI-508 result needs counterparts and reason")
    scoped = {
        record.get("referencePath"): record
        for record in records
        if isinstance(record, dict)
        and record.get("referencePath") in CAPABILITY_STATUS_RECORDS
    }
    for path in CAPABILITY_STATUS_RECORDS:
        if path not in current_reference_paths:
            continue
        record = scoped.get(path)
        if record is None:
            errors.append(f"{path}: capability/status comparison record is missing")
            continue
        if historical_classification(record) in {None, "", "deferred-next-batch"}:
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
        # A later bounded batch may supersede a source path's latest decision.
        # Keep the older batch checks valid for the paths it still owns while
        # validating the newer batch separately below.
        superseded_by_wi475 = set(WI475_REFERENCE_FILES) if expected_source == EXPECTED_REFERENCE_COMMIT else set()
        expected_wi325_paths = (set(WI325_REFERENCE_FILES) - superseded_by_wi475) & current_reference_paths
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
        wi325_classifications = [historical_classification(record) for record in wi325_records]
        expected_wi325_classifications = Counter(
            WI325_REFERENCE_FILES[path][0] for path in expected_wi325_paths
        )
        if any(
            wi325_classifications.count(classification) != count
            for classification, count in expected_wi325_classifications.items()
        ):
            errors.append("WI-325 batch classifications do not match current reference paths")
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
        expected_wi326_paths = (set(WI326_REFERENCE_FILES) - superseded_by_wi475) & current_reference_paths
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
        wi326_classifications = [historical_classification(record) for record in wi326_records]
        expected_wi326_classifications = Counter(
            WI326_REFERENCE_FILES[path][0] for path in expected_wi326_paths
        )
        if any(
            wi326_classifications.count(classification) != count
            for classification, count in expected_wi326_classifications.items()
        ):
            errors.append("WI-326 batch classifications do not match current reference paths")
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
        expected_wi327_paths = set(WI327_REFERENCE_FILES) & current_reference_paths
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
        wi327_classifications = [historical_classification(record) for record in wi327_records]
        expected_wi327_classifications = Counter(
            WI327_REFERENCE_FILES[path][0] for path in expected_wi327_paths
        )
        if any(
            wi327_classifications.count(classification) != count
            for classification, count in expected_wi327_classifications.items()
        ):
            errors.append("WI-327 batch classifications do not match current reference paths")
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
        expected_wi328_paths = (set(WI328_REFERENCE_FILES) - superseded_by_wi494) & current_reference_paths
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
        wi328_classifications = [historical_classification(record) for record in wi328_records]
        expected_wi328_classifications = Counter(
            WI328_REFERENCE_FILES[path][0] for path in expected_wi328_paths
        )
        if any(
            wi328_classifications.count(classification) != count
            for classification, count in expected_wi328_classifications.items()
        ):
            errors.append("WI-328 batch classifications do not match current reference paths")
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
        expected_wi331_paths = set(WI331_REFERENCE_FILES) & current_reference_paths
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
        wi331_classifications = [historical_classification(record) for record in wi331_records]
        expected_wi331_classifications = Counter(
            WI331_REFERENCE_FILES[path][0] for path in expected_wi331_paths
        )
        if any(
            wi331_classifications.count(classification) != count
            for classification, count in expected_wi331_classifications.items()
        ):
            errors.append("WI-331 batch classifications do not match current reference paths")
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
        expected_wi332_paths = set(WI332_REFERENCE_FILES) & current_reference_paths
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
        wi332_classifications = [historical_classification(record) for record in wi332_records]
        expected_wi332_classifications = Counter(
            WI332_REFERENCE_FILES[path][0] for path in expected_wi332_paths
        )
        if any(
            wi332_classifications.count(classification) != count
            for classification, count in expected_wi332_classifications.items()
        ):
            errors.append("WI-332 batch classifications do not match current reference paths")
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
        expected_wi334_paths = set(WI334_REFERENCE_FILES) & current_reference_paths
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
        wi334_classifications = [historical_classification(record) for record in wi334_records]
        expected_wi334_classifications = Counter(
            WI334_REFERENCE_FILES[path][0] for path in expected_wi334_paths
        )
        if any(
            wi334_classifications.count(classification) != count
            for classification, count in expected_wi334_classifications.items()
        ):
            errors.append("WI-334 batch classifications do not match current reference paths")
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
        expected_wi342_paths = (set(WI342_REFERENCE_FILES) & current_reference_paths) - superseding_wi496_paths
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
        wi342_classifications = [historical_classification(record) for record in wi342_records]
        expected_wi342_classifications = Counter(
            WI342_REFERENCE_FILES[path][0] for path in expected_wi342_paths
        )
        if any(
            wi342_classifications.count(classification) != count
            for classification, count in expected_wi342_classifications.items()
        ):
            errors.append("WI-342 batch classifications do not match current reference paths")
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
        expected_wi343_paths = (set(WI343_REFERENCE_FILES) - superseded_by_wi494) & current_reference_paths
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
        wi343_classifications = [historical_classification(record) for record in wi343_records]
        expected_wi343_classifications = Counter(
            WI343_REFERENCE_FILES[path][0] for path in expected_wi343_paths
        )
        if any(
            wi343_classifications.count(classification) != count
            for classification, count in expected_wi343_classifications.items()
        ):
            errors.append("WI-343 batch classifications do not match current reference paths")
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
        expected_wi344_paths = set(WI344_REFERENCE_FILES) & current_reference_paths
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
        wi344_classifications = [historical_classification(record) for record in wi344_records]
        expected_wi344_classifications = Counter(
            WI344_REFERENCE_FILES[path][0] for path in expected_wi344_paths
        )
        if any(
            wi344_classifications.count(classification) != count
            for classification, count in expected_wi344_classifications.items()
        ):
            errors.append("WI-344 batch classifications do not match current reference paths")
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
        expected_wi346_paths = (set(WI346_REFERENCE_FILES) & current_reference_paths) - superseding_wi496_paths
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
        wi346_classifications = [historical_classification(record) for record in wi346_records]
        expected_wi346_classifications = Counter(
            WI346_REFERENCE_FILES[path][0] for path in expected_wi346_paths
        )
        if any(
            wi346_classifications.count(classification) != count
            for classification, count in expected_wi346_classifications.items()
        ):
            errors.append("WI-346 batch classifications do not match current reference paths")
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
        expected_wi347_paths = (set(WI347_REFERENCE_FILES) & current_reference_paths) - superseding_wi496_paths
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
        wi347_classifications = [historical_classification(record) for record in wi347_records]
        expected_wi347_classifications = Counter(
            WI347_REFERENCE_FILES[path][0] for path in expected_wi347_paths
        )
        if any(
            wi347_classifications.count(classification) != count
            for classification, count in expected_wi347_classifications.items()
        ):
            errors.append("WI-347 batch classifications do not match current reference paths")
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
        expected_wi348_paths = (set(WI348_REFERENCE_FILES) & current_reference_paths) - superseding_wi496_paths
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
        wi348_classifications = [historical_classification(record) for record in wi348_records]
        expected_wi348_classifications = Counter(
            WI348_REFERENCE_FILES[path][0] for path in expected_wi348_paths
        )
        if any(
            wi348_classifications.count(classification) != count
            for classification, count in expected_wi348_classifications.items()
        ):
            errors.append("WI-348 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi348_classifications
        ):
            errors.append("WI-348 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI411_BATCH
        for record in records
    ):
        wi411_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI411_BATCH
        ]
        expected_wi411_paths = set(WI411_REFERENCE_FILES) & current_reference_paths
        actual_wi411_paths = {
            record.get("referencePath")
            for record in wi411_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi411_paths != expected_wi411_paths:
            errors.append(
                "WI-411 batch paths do not match the pinned nine-file Java fixture set: "
                f"expected {sorted(expected_wi411_paths)!r}, got {sorted(actual_wi411_paths)!r}"
            )
        if len(wi411_records) != len(expected_wi411_paths):
            errors.append(
                f"WI-411 batch must contain {len(expected_wi411_paths)} records, found {len(wi411_records)}"
            )
        wi411_classifications = [historical_classification(record) for record in wi411_records]
        expected_wi411_classifications = Counter(
            WI411_REFERENCE_FILES[path][0] for path in expected_wi411_paths
        )
        if any(
            wi411_classifications.count(classification) != count
            for classification, count in expected_wi411_classifications.items()
        ):
            errors.append("WI-411 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi411_classifications
        ):
            errors.append("WI-411 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI414_BATCH
        for record in records
    ):
        wi414_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI414_BATCH
        ]
        expected_wi414_paths = set(WI414_REFERENCE_FILES) & current_reference_paths
        actual_wi414_paths = {
            record.get("referencePath")
            for record in wi414_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi414_paths != expected_wi414_paths:
            errors.append(
                "WI-414 batch paths do not match the pinned four-file Python fixture set: "
                f"expected {sorted(expected_wi414_paths)!r}, got {sorted(actual_wi414_paths)!r}"
            )
        if len(wi414_records) != len(expected_wi414_paths):
            errors.append(
                f"WI-414 batch must contain {len(expected_wi414_paths)} records, found {len(wi414_records)}"
            )
        wi414_classifications = [historical_classification(record) for record in wi414_records]
        expected_wi414_classifications = Counter(
            WI414_REFERENCE_FILES[path][0] for path in expected_wi414_paths
        )
        if any(
            wi414_classifications.count(classification) != count
            for classification, count in expected_wi414_classifications.items()
        ):
            errors.append("WI-414 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi414_classifications
        ):
            errors.append("WI-414 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI432_BATCH
        for record in records
    ):
        wi432_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI432_BATCH
        ]
        expected_wi432_paths = set(WI432_REFERENCE_FILES) & current_reference_paths
        actual_wi432_paths = {
            record.get("referencePath")
            for record in wi432_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi432_paths != expected_wi432_paths:
            errors.append(
                "WI-432 batch paths do not match the pinned eleven-file TypeScript web fixture set: "
                f"expected {sorted(expected_wi432_paths)!r}, got {sorted(actual_wi432_paths)!r}"
            )
        if len(wi432_records) != len(expected_wi432_paths):
            errors.append(
                f"WI-432 batch must contain {len(expected_wi432_paths)} records, found {len(wi432_records)}"
            )
        wi432_classifications = [historical_classification(record) for record in wi432_records]
        expected_wi432_classifications = Counter(
            WI432_REFERENCE_FILES[path][0] for path in expected_wi432_paths
        )
        if any(
            wi432_classifications.count(classification) != count
            for classification, count in expected_wi432_classifications.items()
        ):
            errors.append("WI-432 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi432_classifications
        ):
            errors.append("WI-432 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI368_BATCH
        for record in records
    ):
        wi368_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI368_BATCH
        ]
        expected_wi368_paths = (set(WI368_REFERENCE_FILES) & current_reference_paths) - superseding_wi496_paths
        actual_wi368_paths = {
            record.get("referencePath")
            for record in wi368_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi368_paths != expected_wi368_paths:
            errors.append(
                "WI-368 batch paths do not match the pinned eleven-file set: "
                f"expected {sorted(expected_wi368_paths)!r}, got {sorted(actual_wi368_paths)!r}"
            )
        if len(wi368_records) != len(expected_wi368_paths):
            errors.append(
                f"WI-368 batch must contain {len(expected_wi368_paths)} records, found {len(wi368_records)}"
            )
        wi368_classifications = [historical_classification(record) for record in wi368_records]
        expected_wi368_classifications = Counter(
            WI368_REFERENCE_FILES[path][0] for path in expected_wi368_paths
        )
        if any(
            wi368_classifications.count(classification) != count
            for classification, count in expected_wi368_classifications.items()
        ):
            errors.append("WI-368 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi368_classifications
        ):
            errors.append("WI-368 batch cannot leave deferred or migrate-gap records")
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
        expected_wi345_paths = set(WI345_REFERENCE_FILES) & current_reference_paths
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
        wi345_classifications = [historical_classification(record) for record in wi345_records]
        expected_wi345_classifications = Counter(
            WI345_REFERENCE_FILES[path][0] for path in expected_wi345_paths
        )
        if any(
            wi345_classifications.count(classification) != count
            for classification, count in expected_wi345_classifications.items()
        ):
            errors.append("WI-345 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi345_classifications
        ):
            errors.append("WI-345 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI516_BATCH
        for record in records
    ):
        wi516_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI516_BATCH
        ]
        expected_wi516_paths = set(WI516_REFERENCE_FILES) & current_reference_paths
        actual_wi516_paths = {
            record.get("referencePath")
            for record in wi516_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi516_paths != expected_wi516_paths:
            errors.append(
                "WI-516 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi516_paths)!r}, got {sorted(actual_wi516_paths)!r}"
            )
        if len(wi516_records) != len(expected_wi516_paths):
            errors.append(
                f"WI-516 batch must contain {len(expected_wi516_paths)} records, found {len(wi516_records)}"
            )
        wi516_classifications = [historical_classification(record) for record in wi516_records]
        expected_wi516_classifications = Counter(
            WI516_REFERENCE_FILES[path][0] for path in expected_wi516_paths
        )
        if any(
            wi516_classifications.count(classification) != count
            for classification, count in expected_wi516_classifications.items()
        ):
            errors.append("WI-516 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi516_classifications
        ):
            errors.append("WI-516 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI539_BATCH
        for record in records
    ):
        wi539_records = [
            record
            for record in records
            if isinstance(record, dict) and record.get("batch") == WI539_BATCH
        ]
        expected_wi539_paths = set(WI539_REFERENCE_FILES) & current_reference_paths
        actual_wi539_paths = {
            record.get("referencePath")
            for record in wi539_records
            if isinstance(record.get("referencePath"), str)
        }
        if actual_wi539_paths != expected_wi539_paths:
            errors.append(
                "WI-539 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi539_paths)!r}, got {sorted(actual_wi539_paths)!r}"
            )
        if len(wi539_records) != len(expected_wi539_paths):
            errors.append(
                f"WI-539 batch must contain {len(expected_wi539_paths)} records, found {len(wi539_records)}"
            )
        wi539_classifications = [historical_classification(record) for record in wi539_records]
        expected_wi539_classifications = Counter(
            WI539_REFERENCE_FILES[path][0] for path in expected_wi539_paths
        )
        if any(
            wi539_classifications.count(classification) != count
            for classification, count in expected_wi539_classifications.items()
        ):
            errors.append("WI-539 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi539_classifications
        ):
            errors.append("WI-539 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI543_BATCH
        for record in records
    ):
        wi543_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI543_BATCH
            and record.get("referencePath") in WI543_REFERENCE_FILES
        ]
        expected_wi543_paths = set(WI543_REFERENCE_FILES) & current_reference_paths
        actual_wi543_paths = {record.get("referencePath") for record in wi543_records}
        if actual_wi543_paths != expected_wi543_paths:
            errors.append(
                "WI-543 source checker records do not match the seven scoped paths: "
                f"expected {sorted(expected_wi543_paths)!r}, got {sorted(actual_wi543_paths)!r}"
            )
        if len(wi543_records) != len(expected_wi543_paths):
            errors.append(
                f"WI-543 batch must contain {len(expected_wi543_paths)} records, found {len(wi543_records)}"
            )
        wi543_classifications = [historical_classification(record) for record in wi543_records]
        expected_wi543_classifications = Counter(
            WI543_REFERENCE_FILES[path][0] for path in expected_wi543_paths
        )
        if any(
            wi543_classifications.count(classification) != count
            for classification, count in expected_wi543_classifications.items()
        ):
            errors.append("WI-543 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi543_classifications
        ):
            errors.append("WI-543 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI548_BATCH
        for record in records
    ):
        wi548_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI548_BATCH
            and record.get("referencePath") in WI548_REFERENCE_FILES
        ]
        expected_wi548_paths = set(WI548_REFERENCE_FILES) & current_reference_paths
        actual_wi548_paths = {record.get("referencePath") for record in wi548_records}
        if actual_wi548_paths != expected_wi548_paths:
            errors.append(
                "WI-548 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi548_paths)!r}, got {sorted(actual_wi548_paths)!r}"
            )
        if len(wi548_records) != len(expected_wi548_paths):
            errors.append(
                f"WI-548 batch must contain {len(expected_wi548_paths)} records, found {len(wi548_records)}"
            )
        wi548_classifications = [historical_classification(record) for record in wi548_records]
        expected_wi548_classifications = Counter(
            WI548_REFERENCE_FILES[path][0] for path in expected_wi548_paths
        )
        if any(
            wi548_classifications.count(classification) != count
            for classification, count in expected_wi548_classifications.items()
        ):
            errors.append("WI-548 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi548_classifications
        ):
            errors.append("WI-548 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI550_BATCH
        for record in records
    ):
        wi550_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI550_BATCH
            and record.get("referencePath") in WI550_REFERENCE_FILES
        ]
        expected_wi550_paths = set(WI550_REFERENCE_FILES) & current_reference_paths
        actual_wi550_paths = {record.get("referencePath") for record in wi550_records}
        if actual_wi550_paths != expected_wi550_paths:
            errors.append(
                "WI-550 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi550_paths)!r}, got {sorted(actual_wi550_paths)!r}"
            )
        if len(wi550_records) != len(expected_wi550_paths):
            errors.append(
                f"WI-550 batch must contain {len(expected_wi550_paths)} records, found {len(wi550_records)}"
            )
        wi550_classifications = [historical_classification(record) for record in wi550_records]
        expected_wi550_classifications = Counter(
            WI550_REFERENCE_FILES[path][0] for path in expected_wi550_paths
        )
        if any(
            wi550_classifications.count(classification) != count
            for classification, count in expected_wi550_classifications.items()
        ):
            errors.append("WI-550 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi550_classifications
        ):
            errors.append("WI-550 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI552_BATCH
        for record in records
    ):
        wi552_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI552_BATCH
            and record.get("referencePath") in WI552_REFERENCE_FILES
        ]
        expected_wi552_paths = set(WI552_REFERENCE_FILES) & current_reference_paths
        actual_wi552_paths = {record.get("referencePath") for record in wi552_records}
        if actual_wi552_paths != expected_wi552_paths:
            errors.append(
                "WI-552 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi552_paths)!r}, got {sorted(actual_wi552_paths)!r}"
            )
        if len(wi552_records) != len(expected_wi552_paths):
            errors.append(
                f"WI-552 batch must contain {len(expected_wi552_paths)} records, found {len(wi552_records)}"
            )
        wi552_classifications = [historical_classification(record) for record in wi552_records]
        expected_wi552_classifications = Counter(
            WI552_REFERENCE_FILES[path][0] for path in expected_wi552_paths
        )
        if any(
            wi552_classifications.count(classification) != count
            for classification, count in expected_wi552_classifications.items()
        ):
            errors.append("WI-552 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi552_classifications
        ):
            errors.append("WI-552 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI559_BATCH
        for record in records
    ):
        wi559_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI559_BATCH
            and record.get("referencePath") in WI559_REFERENCE_FILES
        ]
        expected_wi559_paths = set(WI559_REFERENCE_FILES) & current_reference_paths
        actual_wi559_paths = {record.get("referencePath") for record in wi559_records}
        if actual_wi559_paths != expected_wi559_paths:
            errors.append(
                "WI-559 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi559_paths)!r}, got {sorted(actual_wi559_paths)!r}"
            )
        if len(wi559_records) != len(expected_wi559_paths):
            errors.append(
                f"WI-559 batch must contain {len(expected_wi559_paths)} records, found {len(wi559_records)}"
            )
        wi559_classifications = [historical_classification(record) for record in wi559_records]
        expected_wi559_classifications = Counter(
            WI559_REFERENCE_FILES[path][0] for path in expected_wi559_paths
        )
        if any(
            wi559_classifications.count(classification) != count
            for classification, count in expected_wi559_classifications.items()
        ):
            errors.append("WI-559 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi559_classifications
        ):
            errors.append("WI-559 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI563_BATCH
        for record in records
    ):
        wi563_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI563_BATCH
            and record.get("referencePath") in WI563_REFERENCE_FILES
        ]
        expected_wi563_paths = set(WI563_REFERENCE_FILES) & current_reference_paths
        actual_wi563_paths = {record.get("referencePath") for record in wi563_records}
        if actual_wi563_paths != expected_wi563_paths:
            errors.append(
                "WI-563 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi563_paths)!r}, got {sorted(actual_wi563_paths)!r}"
            )
        if len(wi563_records) != len(expected_wi563_paths):
            errors.append(
                f"WI-563 batch must contain {len(expected_wi563_paths)} records, found {len(wi563_records)}"
            )
        wi563_classifications = [historical_classification(record) for record in wi563_records]
        expected_wi563_classifications = Counter(
            WI563_REFERENCE_FILES[path][0] for path in expected_wi563_paths
        )
        if any(
            wi563_classifications.count(classification) != count
            for classification, count in expected_wi563_classifications.items()
        ):
            errors.append("WI-563 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi563_classifications
        ):
            errors.append("WI-563 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI568_BATCH
        for record in records
    ):
        wi568_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI568_BATCH
            and record.get("referencePath") in WI568_REFERENCE_FILES
        ]
        expected_wi568_paths = set(WI568_REFERENCE_FILES) & current_reference_paths
        actual_wi568_paths = {record.get("referencePath") for record in wi568_records}
        if actual_wi568_paths != expected_wi568_paths:
            errors.append(
                "WI-568 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi568_paths)!r}, got {sorted(actual_wi568_paths)!r}"
            )
        if len(wi568_records) != len(expected_wi568_paths):
            errors.append(
                f"WI-568 batch must contain {len(expected_wi568_paths)} records, found {len(wi568_records)}"
            )
        wi568_classifications = [historical_classification(record) for record in wi568_records]
        expected_wi568_classifications = Counter(
            WI568_REFERENCE_FILES[path][0] for path in expected_wi568_paths
        )
        if any(
            wi568_classifications.count(classification) != count
            for classification, count in expected_wi568_classifications.items()
        ):
            errors.append("WI-568 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi568_classifications
        ):
            errors.append("WI-568 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI572_BATCH
        for record in records
    ):
        wi572_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI572_BATCH
            and record.get("referencePath") in WI572_REFERENCE_FILES
        ]
        expected_wi572_paths = set(WI572_REFERENCE_FILES) & current_reference_paths
        actual_wi572_paths = {record.get("referencePath") for record in wi572_records}
        if actual_wi572_paths != expected_wi572_paths:
            errors.append(
                "WI-572 batch paths do not match the pinned current-file set: "
                f"expected {sorted(expected_wi572_paths)!r}, got {sorted(actual_wi572_paths)!r}"
            )
        if len(wi572_records) != len(expected_wi572_paths):
            errors.append(
                f"WI-572 batch must contain {len(expected_wi572_paths)} records, found {len(wi572_records)}"
            )
        wi572_classifications = [historical_classification(record) for record in wi572_records]
        expected_wi572_classifications = Counter(
            WI572_REFERENCE_FILES[path][0] for path in expected_wi572_paths
        )
        if any(
            wi572_classifications.count(classification) != count
            for classification, count in expected_wi572_classifications.items()
        ):
            errors.append("WI-572 batch classifications do not match current reference paths")
        if any(
            classification in {"deferred-next-batch", "migrate-gap"}
            for classification in wi572_classifications
        ):
            errors.append("WI-572 batch cannot leave deferred or migrate-gap records")
    if any(
        isinstance(record, dict) and record.get("batch") == WI579_BATCH
        for record in records
    ):
        wi579_records = [
            record
            for record in records
            if isinstance(record, dict)
            and record.get("batch") == WI579_BATCH
            and record.get("referencePath") in WI579_REFERENCE_FILES
        ]
        expected_wi579_paths = set(WI579_REFERENCE_FILES) & current_reference_paths
        actual_wi579_paths = {record.get("referencePath") for record in wi579_records}
        if actual_wi579_paths != expected_wi579_paths:
            errors.append(
                "WI-579 template batch paths do not match the pinned sixteen-file set: "
                f"expected {sorted(expected_wi579_paths)!r}, got {sorted(actual_wi579_paths)!r}"
            )
        if len(wi579_records) != len(expected_wi579_paths):
            errors.append(
                f"WI-579 batch must contain {len(expected_wi579_paths)} records, found {len(wi579_records)}"
            )
        expected_wi579_classifications = Counter(
            WI579_REFERENCE_FILES[path][0] for path in expected_wi579_paths
        )
        wi579_classifications = [record.get("classification") for record in wi579_records]
        if Counter(wi579_classifications) != expected_wi579_classifications:
            errors.append(
                "WI-579 template classifications do not match the bounded decisions"
            )
        for record in wi579_records:
            if not record.get("rustCounterparts") or not record.get("reason"):
                errors.append(
                    f"{record.get('referencePath')}: WI-579 result needs counterparts and reason"
                )
            if record.get("classification") in {"deferred-next-batch", "migrate-gap"}:
                errors.append(
                    f"{record.get('referencePath')}: WI-579 cannot leave deferred or migrate-gap"
                )
    expected_count = manifest.get("referenceTrackedFileCount")
    if expected_count != len(current_record_paths):
        errors.append(
            f"referenceTrackedFileCount {expected_count!r} != non-retired record count {len(current_record_paths)}"
        )
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


def apply_wi441_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI441_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI441_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI441_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI441_REFERENCE_FILES)} WI-441 records, found {updated}"
        )
    return updated


def apply_wi461_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI461_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI461_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI461_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI461_REFERENCE_FILES)} WI-461 records, found {updated}"
        )
    return updated


def apply_wi464_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI464_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI464_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI464_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI464_REFERENCE_FILES)} WI-464 records, found {updated}"
        )
    return updated


def apply_wi475_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI475_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI475_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI475_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI475_REFERENCE_FILES)} WI-475 records, found {updated}"
        )
    return updated


def apply_wi482_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI482_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI482_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI482_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI482_REFERENCE_FILES)} WI-482 records, found {updated}"
        )
    return updated


def apply_wi494_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI494_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI494_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI494_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI494_REFERENCE_FILES)} WI-494 records, found {updated}"
        )
    return updated


def apply_wi496_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI496_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI496_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI496_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI496_REFERENCE_FILES)} WI-496 records, found {updated}"
        )
    return updated


def apply_wi504_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI504_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI504_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI504_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI504_REFERENCE_FILES)} WI-504 records, found {updated}"
        )
    return updated


def apply_wi507_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI507_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI507_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI507_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI507_REFERENCE_FILES)} WI-507 records, found {updated}"
        )
    return updated


def apply_wi508_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI508_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI508_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI508_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI508_REFERENCE_FILES)} WI-508 records, found {updated}"
        )
    return updated


def apply_wi512_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI512_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI512_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
            }
        )
        updated += 1
    if updated != len(WI512_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI512_REFERENCE_FILES)} WI-512 records, found {updated}"
        )
    return updated


def apply_wi516_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI516_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI516_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                # A rebaseline batch now owns the current decision. Keep the
                # source-change marker for delta accounting, but do not let
                # the old deferred classification shadow this decision.
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI516_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI516_REFERENCE_FILES)} WI-516 records, found {updated}"
        )
    return updated


def apply_wi539_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI539_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI539_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI539_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI539_REFERENCE_FILES)} WI-539 records, found {updated}"
        )
    return updated


def apply_wi543_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI543_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI543_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI543_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI543_REFERENCE_FILES)} WI-543 records, found {updated}"
        )
    return updated


def apply_wi548_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI548_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI548_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI548_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI548_REFERENCE_FILES)} WI-548 records, found {updated}"
        )
    return updated


def apply_wi550_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI550_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI550_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI550_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI550_REFERENCE_FILES)} WI-550 records, found {updated}"
        )
    return updated


def apply_wi552_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI552_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI552_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI552_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI552_REFERENCE_FILES)} WI-552 records, found {updated}"
        )
    return updated


def apply_wi557_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI557_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI557_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI557_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI557_REFERENCE_FILES)} WI-557 records, found {updated}"
        )
    return updated


def apply_wi559_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI559_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI559_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI559_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI559_REFERENCE_FILES)} WI-559 records, found {updated}"
        )
    return updated


def apply_wi563_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI563_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI563_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI563_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI563_REFERENCE_FILES)} WI-563 records, found {updated}"
        )
    return updated


def apply_wi568_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI568_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI568_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI568_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI568_REFERENCE_FILES)} WI-568 records, found {updated}"
        )
    return updated


def apply_wi572_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI572_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        record.update(
            {
                "batch": WI572_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": classification,
            }
        )
        updated += 1
    if updated != len(WI572_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI572_REFERENCE_FILES)} WI-572 records, found {updated}"
        )
    return updated


def apply_wi579_batch(manifest: dict[str, Any]) -> int:
    records = manifest.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")
    updated = 0
    for record in records:
        path = record.get("referencePath") if isinstance(record, dict) else None
        details = WI579_REFERENCE_FILES.get(path)
        if details is None:
            continue
        classification, counterparts, reason = details
        previous_classification = record.get("classification")
        record.update(
            {
                "batch": WI579_BATCH,
                "classification": classification,
                "rustCounterparts": counterparts,
                "reason": reason,
                "previousClassification": previous_classification,
            }
        )
        updated += 1
    if updated != len(WI579_REFERENCE_FILES):
        raise ValueError(
            f"expected {len(WI579_REFERENCE_FILES)} WI-579 records, found {updated}"
        )
    return updated


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference", type=Path)
    parser.add_argument("--target", type=Path)
    parser.add_argument("--manifest", type=Path, default=Path("tests/conformance/reference_file_inventory.json"))
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--rebaseline-from",
        type=Path,
        help="project an existing ledger onto a newer local reference commit",
    )
    parser.add_argument("--source-commit", default=EXPECTED_REFERENCE_COMMIT)
    parser.add_argument("--target-commit", default=EXPECTED_TARGET_COMMIT)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--apply-getting-started-batch", action="store_true")
    parser.add_argument("--apply-wi441-batch", action="store_true")
    parser.add_argument("--apply-wi461-batch", action="store_true")
    parser.add_argument("--apply-wi464-batch", action="store_true")
    parser.add_argument("--apply-wi475-batch", action="store_true")
    parser.add_argument("--apply-wi482-batch", action="store_true")
    parser.add_argument("--apply-wi494-batch", action="store_true")
    parser.add_argument("--apply-wi496-batch", action="store_true")
    parser.add_argument("--apply-wi504-batch", action="store_true")
    parser.add_argument("--apply-wi507-batch", action="store_true")
    parser.add_argument("--apply-wi508-batch", action="store_true")
    parser.add_argument("--apply-wi512-batch", action="store_true")
    parser.add_argument("--apply-wi516-batch", action="store_true")
    parser.add_argument("--apply-wi539-batch", action="store_true")
    parser.add_argument("--apply-wi543-batch", action="store_true")
    parser.add_argument("--apply-wi548-batch", action="store_true")
    parser.add_argument("--apply-wi550-batch", action="store_true")
    parser.add_argument("--apply-wi552-batch", action="store_true")
    parser.add_argument("--apply-wi557-batch", action="store_true")
    parser.add_argument("--apply-wi559-batch", action="store_true")
    parser.add_argument("--apply-wi563-batch", action="store_true")
    parser.add_argument("--apply-wi568-batch", action="store_true")
    parser.add_argument("--apply-wi572-batch", action="store_true")
    parser.add_argument("--apply-wi579-batch", action="store_true")
    args = parser.parse_args()

    # ``--check`` is a read-only operation.  Do not let an accidentally
    # combined generation/rebaseline/apply option rewrite the checked ledger
    # before validation.  This is especially important for the append-only
    # retired-history records: a check must never replace them with a fresh
    # generated projection.
    apply_options = (
        args.apply_getting_started_batch,
        args.apply_wi441_batch,
        args.apply_wi461_batch,
        args.apply_wi464_batch,
        args.apply_wi475_batch,
        args.apply_wi482_batch,
        args.apply_wi494_batch,
        args.apply_wi496_batch,
        args.apply_wi504_batch,
        args.apply_wi507_batch,
        args.apply_wi508_batch,
        args.apply_wi512_batch,
        args.apply_wi516_batch,
        args.apply_wi539_batch,
        args.apply_wi543_batch,
        args.apply_wi548_batch,
        args.apply_wi550_batch,
        args.apply_wi552_batch,
        args.apply_wi557_batch,
        args.apply_wi559_batch,
        args.apply_wi563_batch,
        args.apply_wi568_batch,
        args.apply_wi572_batch,
        args.apply_wi579_batch,
    )
    if args.check and (args.reference or args.target or args.rebaseline_from or any(apply_options)):
        parser.error(
            "--check is read-only and cannot be combined with --reference, --target, "
            "--rebaseline-from, or --apply-*; pass only --manifest and expected revisions"
        )

    if args.rebaseline_from:
        if not args.reference or not args.target:
            parser.error("--rebaseline-from requires --reference and --target")
        manifest = rebaseline(
            args.rebaseline_from,
            args.reference,
            args.target,
            args.source_commit,
            args.target_commit,
        )
        output = args.output or args.manifest
        output.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    elif args.reference and args.target:
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
    if args.apply_wi441_batch:
        try:
            apply_wi441_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi461_batch:
        try:
            apply_wi461_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi464_batch:
        try:
            apply_wi464_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi475_batch:
        try:
            apply_wi475_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi482_batch:
        try:
            apply_wi482_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi494_batch:
        try:
            apply_wi494_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi496_batch:
        try:
            apply_wi496_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi504_batch:
        try:
            apply_wi504_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi507_batch:
        try:
            apply_wi507_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi508_batch:
        try:
            apply_wi508_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi512_batch:
        try:
            apply_wi512_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi516_batch:
        try:
            apply_wi516_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi539_batch:
        try:
            apply_wi539_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi543_batch:
        try:
            apply_wi543_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi548_batch:
        try:
            apply_wi548_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi550_batch:
        try:
            apply_wi550_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi552_batch:
        try:
            apply_wi552_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi557_batch:
        try:
            apply_wi557_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi559_batch:
        try:
            apply_wi559_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi563_batch:
        try:
            apply_wi563_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi568_batch:
        try:
            apply_wi568_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi572_batch:
        try:
            apply_wi572_batch(manifest)
        except ValueError as error:
            print(f"ERROR: {error}", file=sys.stderr)
            return 1
        args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    if args.apply_wi579_batch:
        try:
            apply_wi579_batch(manifest)
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
