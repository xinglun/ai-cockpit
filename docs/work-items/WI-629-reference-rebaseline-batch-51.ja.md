---
author: AI Cockpit maintainers
title: WI-629 — Reference rebaseline batch 51
description: 固定した local commit の source-changed path 60 件を一件ずつ再確認し、source implementation はコピーしない。
audience: [maintainer, reviewer, adopter]
workItemId: WI-629-reference-rebaseline-batch-51
status: implemented
authority: canonical
lastVerifiedBy: WI-629-reference-rebaseline-batch-51
terminalArchive: .ai/work-items/archive/WI-629-reference-rebaseline-batch-51.contract.json
terminalVerification: .ai/evidence/WI-629-reference-rebaseline-batch-51.verification.json
terminalFinalization: .ai/decisions/WI-629-reference-rebaseline-batch-51.finalize.json
terminalDecision: .ai/decisions/WI-629-reference-rebaseline-batch-51.close.json
---

# WI-629 — Reference rebaseline batch 51

## Intent と boundary

現行 local reference rebaseline 後に bytes が変わった最初の非履歴 path 60 件を一件ずつ再確認します。各 path に明示的な分類、Rust 対応物、copy しない境界を記録します。Reference checkout は `a9224aed77b5c317b53c4551a9eec306d91ee330` に固定され、Rust Runtime にコピーする source tree ではありません。

source 側 adopter capability manifest/schema は `reference-only` です。Rust Runtime は truthful な request-scoped capability/status view を提供しますが、source manifest や JSON wire format の完全互換を主張しません。その他の path は shared Rust Runtime、repository-native test、CI/release boundary、tri-language reader documentation が意図した別設計で担います。本 batch に `migrate-gap` はありません。

## File-by-file decision table

| Reference path | 分類 | Rust 対応物 / 境界 |
| --- | --- | --- |
| `.ai/cockpit/README.ja.md` | implemented-different-by-design | `.ai/README.md；docs/reference/agent-workflow.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/cockpit/README.md` | implemented-different-by-design | `.ai/README.md；docs/reference/agent-workflow.md；docs/reference/outcome-report.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/cockpit/adoption.ja.md` | implemented-different-by-design | `docs/getting-started/README.ja.md；docs/getting-started/adopter-configuration.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/cockpit/work-items/index.json` | implemented-different-by-design | `crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/src/lib.rs；crates/cockpit-cli/src/main.rs；crates/cockpit-mcp/src/lib.rs；docs/reference/commands.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/cockpit/work-items/wi-06-status-interface.status.json` | implemented-different-by-design | `crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/src/lib.rs；crates/cockpit-cli/src/main.rs；crates/cockpit-mcp/src/lib.rs；docs/reference/commands.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/guards/changed_critical_coverage_policy.json` | implemented-different-by-design | `tests/conformance/reference_file_inventory.py；tests/ci/governance_integrity_gate.py；crates/cockpit-repository/src/governance_controls.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/guards/coverage_policy.yaml` | implemented-different-by-design | `tests/ci/governance_integrity_gate.py；crates/cockpit-repository/src/governance_controls.rs；docs/reference/ci-quality-gates.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/project/adopter-capability-manifest.json` | reference-only | `crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/src/lib.rs；crates/cockpit-cli/src/main.rs；crates/cockpit-mcp/src/lib.rs；crates/cockpit-repository/tests/project_governance.rs；docs/capabilities.md；source manifest/schema は reference/provider material とし、完全な adopter manifest を主張しない。` |
| `.ai/quality/governance-routing.yaml` | implemented-different-by-design | `.github/workflows/ci.yml；tests/ci/quality_route.py；tests/ci/run_repository_gates.py；docs/reference/ci-quality-gates.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/adopter-capability-manifest.schema.json` | reference-only | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source manifest/schema は reference/provider material とし、完全な adopter manifest を主張しない。` |
| `.ai/schemas/cross-wi-integration-report.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/evidence-binding.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/governance-cost-report.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/implementation-knowledge-dependency-index.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/implementation-knowledge-index.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/implementation-knowledge-query.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/implementation-knowledge-record.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/parallel-verification-plan.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/performance-diagnosis-report.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/task_outcome.schema.json` | implemented-different-by-design | `crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/src/lib.rs；crates/cockpit-mcp/src/lib.rs；docs/reference/outcome-report.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.ai/schemas/work-item-status-interface.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | `.github/workflows/ci.yml；tests/ci/quality_route.py；tests/ci/run_repository_gates.py；tests/release/adopter_acceptance.sh；docs/capabilities.md；docs/release/distribution.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.github/workflows/release.yml` | implemented-different-by-design | `.github/workflows/release.yml；tests/release/workflow_policy.sh；tests/release/version_consistency.sh；tests/release/adopter_acceptance.sh；tests/release/adopter_upgrade_acceptance.sh；docs/release/distribution.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `.github/workflows/smoke.yml` | implemented-different-by-design | `.github/workflows/ci.yml；.github/workflows/release.yml；tests/ci/repository_gate_manifest.json；tests/release/adopter_acceptance.sh；tests/release/adopter_upgrade_acceptance.sh；docs/reference/reference-file-comparison.md；docs/release/distribution.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `AGENTS.md` | implemented-different-by-design | `AGENTS.md；.ai/README.md；docs/reference/agent-workflow.md；crates/cockpit-agent/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `GEMINI.md` | implemented-different-by-design | `.ai/README.md；crates/cockpit-agent/src/lib.rs；crates/cockpit-agent/tests/install.rs；docs/reference/agent-workflow.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `Makefile` | implemented-different-by-design | `.github/workflows/ci.yml；tests/ci/run_repository_gates.py；docs/reference/commands.md；Cargo.toml；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/README.ja.md` | implemented-different-by-design | `docs/README.ja.md；docs/current/README.ja.md；docs/getting-started/README.ja.md；docs/reference/README.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/README.md` | implemented-different-by-design | `docs/README.md；docs/current/README.md；docs/getting-started/README.md；docs/reference/README.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/README.zh-CN.md` | implemented-different-by-design | `docs/README.zh-CN.md；docs/current/README.zh-CN.md；docs/getting-started/README.zh-CN.md；docs/reference/README.zh-CN.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/capabilities.ja.md` | implemented-different-by-design | `docs/capabilities.ja.md；docs/reference/capability-truth-matrix.md；docs/reference/commands.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/capabilities.md` | implemented-different-by-design | `docs/capabilities.md；docs/reference/capability-truth-matrix.md；docs/reference/commands.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/capabilities.zh-CN.md` | implemented-different-by-design | `docs/capabilities.zh-CN.md；docs/reference/capability-truth-matrix.md；docs/reference/commands.zh-CN.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/human-benefit-report.ja.md` | implemented-different-by-design | `docs/features/human-benefit-report.ja.md；docs/features/task-outcome-report.ja.md；docs/reference/outcome-report.ja.md；docs/reference/task-outcome-events.ja.md；crates/cockpit-cli/tests/outcome_handoff.rs；crates/cockpit-mcp/tests/rpc.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/human-benefit-report.md` | implemented-different-by-design | `docs/features/human-benefit-report.md；docs/features/task-outcome-report.md；docs/reference/outcome-report.md；docs/reference/task-outcome-events.md；crates/cockpit-cli/tests/outcome_handoff.rs；crates/cockpit-mcp/tests/rpc.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/human-benefit-report.zh-CN.md` | implemented-different-by-design | `docs/features/human-benefit-report.zh-CN.md；docs/features/task-outcome-report.zh-CN.md；docs/reference/outcome-report.zh-CN.md；docs/reference/task-outcome-events.zh-CN.md；crates/cockpit-cli/tests/outcome_handoff.rs；crates/cockpit-mcp/tests/rpc.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/task-outcome-report.ja.md` | implemented-different-by-design | `docs/features/task-outcome-report.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/task-outcome-report.md` | implemented-different-by-design | `docs/features/task-outcome-report.md；docs/reference/outcome-report.md；crates/cockpit-repository/src/lib.rs；crates/cockpit-cli/src/main.rs；crates/cockpit-mcp/src/lib.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/task-outcome-report.zh-CN.md` | implemented-different-by-design | `docs/features/task-outcome-report.zh-CN.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/work-item-parallelism.ja.md` | implemented-different-by-design | `docs/work-items/WI-123-parallel-contract-boundary.ja.md；docs/reference/configuration.ja.md；crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/tests/parallel_boundary.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/work-item-parallelism.md` | implemented-different-by-design | `docs/work-items/WI-123-parallel-contract-boundary.md；docs/reference/configuration.md；crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/tests/parallel_boundary.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/features/work-item-parallelism.zh-CN.md` | implemented-different-by-design | `docs/work-items/WI-123-parallel-contract-boundary.zh-CN.md；docs/reference/configuration.zh-CN.md；crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/tests/parallel_boundary.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/first-work-item.ja.md` | implemented-different-by-design | `docs/getting-started/first-work-item.ja.md；docs/reference/agent-workflow.ja.md；docs/reference/outcome-report.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/first-work-item.md` | implemented-different-by-design | `docs/getting-started/first-work-item.md；docs/reference/agent-workflow.md；docs/reference/outcome-report.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/first-work-item.zh-CN.md` | implemented-different-by-design | `docs/getting-started/first-work-item.zh-CN.md；docs/reference/agent-workflow.zh-CN.md；docs/reference/outcome-report.zh-CN.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/security-release-verification.ja.md` | implemented-different-by-design | `docs/getting-started/security-release-verification.ja.md；docs/release/distribution.ja.md；docs/getting-started/installation-security.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/security-release-verification.md` | implemented-different-by-design | `docs/getting-started/security-release-verification.md；docs/release/distribution.md；docs/getting-started/installation-security.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/security-release-verification.zh-CN.md` | implemented-different-by-design | `docs/getting-started/security-release-verification.zh-CN.md；docs/release/distribution.zh-CN.md；docs/getting-started/installation-security.zh-CN.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/standard-adoption-guide.ja.md` | implemented-different-by-design | `docs/getting-started/standard-adoption-guide.ja.md；docs/getting-started/installation.ja.md；docs/getting-started/first-work-item.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/standard-adoption-guide.md` | implemented-different-by-design | `docs/getting-started/standard-adoption-guide.md；docs/getting-started/installation.md；docs/getting-started/first-work-item.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/getting-started/standard-adoption-guide.zh-CN.md` | implemented-different-by-design | `docs/getting-started/standard-adoption-guide.zh-CN.md；docs/getting-started/installation.zh-CN.md；docs/getting-started/first-work-item.zh-CN.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/maintainers/task-outcome-events.md` | implemented-different-by-design | `docs/reference/task-outcome-events.md；docs/reference/task-outcome-events.zh-CN.md；docs/reference/task-outcome-events.ja.md；docs/features/task-outcome-report.md；crates/cockpit-repository/src/lib.rs；crates/cockpit-repository/tests/task_outcome_events.rs；crates/cockpit-cli/tests/outcome_handoff.rs；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/operations/quality-gates.ja.md` | implemented-different-by-design | `docs/reference/ci-quality-gates.ja.md；docs/reference/governance-integrity-gate.ja.md；tests/ci/repository_gate_manifest.json；tests/ci/run_repository_gates.py；.github/workflows/ci.yml；docs/release/distribution.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/operations/quality-gates.md` | implemented-different-by-design | `docs/reference/ci-quality-gates.md；docs/reference/governance-integrity-gate.md；tests/ci/repository_gate_manifest.json；tests/ci/run_repository_gates.py；.github/workflows/ci.yml；docs/release/distribution.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/operations/quality-gates.zh-CN.md` | implemented-different-by-design | `docs/reference/ci-quality-gates.zh-CN.md；docs/reference/governance-integrity-gate.zh-CN.md；tests/ci/repository_gate_manifest.json；tests/ci/run_repository_gates.py；.github/workflows/ci.yml；docs/release/distribution.zh-CN.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/operations/work-item-lifecycle.ja.md` | implemented-different-by-design | `docs/reference/agent-workflow.ja.md；docs/reference/outcome-report.ja.md；docs/reference/reference-file-comparison.ja.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/operations/work-item-lifecycle.md` | implemented-different-by-design | `docs/reference/agent-workflow.md；docs/reference/outcome-report.md；docs/reference/reference-file-comparison.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/operations/work-item-lifecycle.zh-CN.md` | implemented-different-by-design | `docs/reference/agent-workflow.zh-CN.md；docs/reference/outcome-report.zh-CN.md；docs/reference/reference-file-comparison.zh-CN.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/reference/adoption-reality-report.md` | implemented-different-by-design | `docs/capabilities.md；docs/release/distribution.md；docs/security/enterprise-governance.md；crates/cockpit-repository/src/project_governance.rs；crates/cockpit-repository/tests/project_governance.rs；tests/release/adopter_acceptance.sh；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |
| `docs/reference/agent-parallel-work-items.md` | implemented-different-by-design | `docs/reference/cross-work-item-dedup.md；docs/reference/affected-verification.md；docs/reference/agent-workflow.md；AGENTS.md；.ai/README.md；source bytes/command は copy せず、Runtime/reader route が意味を担う。` |


## Adopter inheritance

attach 済みの object repository は shared Runtime、明示的な `--repo` context、isolated Contract/evidence/knowledge、dynamic verification、fail-closed lifecycle、visible human Outcome を継承します。source Python module、Make target、provider-global config、interactive stack installer、generated history、source JSON wire format は継承しません。

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --apply-wi629-batch --source-commit a9224aed77b5c317b53c4551a9eec306d91ee330 --target-commit 98f12b18b978db509fc884a8a6225afeb7f10df5
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit a9224aed77b5c317b53c4551a9eec306d91ee330 --target-commit 98f12b18b978db509fc884a8a6225afeb7f10df5
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/docs/reference_comparison_metadata_test.py
python3 tests/conformance/reference_inventory_docs_test.py
```

参照：[English](WI-629-reference-rebaseline-batch-51.md) · [中文](WI-629-reference-rebaseline-batch-51.zh-CN.md)
