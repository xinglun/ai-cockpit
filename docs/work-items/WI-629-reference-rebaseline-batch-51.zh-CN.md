---
author: AI Cockpit maintainers
title: WI-629——参考源重新基线第 51 批
description: 在固定本地提交逐个复核首批 60 个变更路径，不复制源实现。
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

# WI-629——参考源重新基线第 51 批

## 意图与边界

本批次逐个复核当前本地参考源重新基线后发生变化的首批 60 个非历史路径。每条路径都登记明确分类、Rust 对应物和不复制边界。参考源固定在
`a9224aed77b5c317b53c4551a9eec306d91ee330`，是规格与行为语料，不是要复制到 Rust Runtime 的源代码树。

源侧 adopter capability manifest 及其 schema 保持 `reference-only`：Rust Runtime 提供真实的 request-scoped capability/status 视图，但不宣称拥有源 manifest 或其 JSON 线格式。其余路径由共享 Rust Runtime、仓库原生测试、CI/release 边界或三语读者文档以不同设计承载。本批次未发现 `migrate-gap`。

## 逐文件决定表

| 参考路径 | 分类 | Rust 对应物 / 边界决定 |
| --- | --- | --- |
| `.ai/cockpit/README.ja.md` | implemented-different-by-design | `.ai/README.md；docs/reference/agent-workflow.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/cockpit/README.md` | implemented-different-by-design | `.ai/README.md；docs/reference/agent-workflow.md；docs/reference/outcome-report.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/cockpit/adoption.ja.md` | implemented-different-by-design | `docs/getting-started/README.ja.md；docs/getting-started/adopter-configuration.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/cockpit/work-items/index.json` | implemented-different-by-design | `crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/src/lib.rs；crates/cockpit-cli/src/main.rs；crates/cockpit-mcp/src/lib.rs；docs/reference/commands.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/cockpit/work-items/wi-06-status-interface.status.json` | implemented-different-by-design | `crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/src/lib.rs；crates/cockpit-cli/src/main.rs；crates/cockpit-mcp/src/lib.rs；docs/reference/commands.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/guards/changed_critical_coverage_policy.json` | implemented-different-by-design | `tests/conformance/reference_file_inventory.py；tests/ci/governance_integrity_gate.py；crates/cockpit-repository/src/governance_controls.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/guards/coverage_policy.yaml` | implemented-different-by-design | `tests/ci/governance_integrity_gate.py；crates/cockpit-repository/src/governance_controls.rs；docs/reference/ci-quality-gates.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/project/adopter-capability-manifest.json` | reference-only | `crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/src/lib.rs；crates/cockpit-cli/src/main.rs；crates/cockpit-mcp/src/lib.rs；crates/cockpit-repository/tests/project_governance.rs；docs/capabilities.md；源 manifest/schema 保持 reference/provider material，不宣称 Rust 有完整 adopter manifest。` |
| `.ai/quality/governance-routing.yaml` | implemented-different-by-design | `.github/workflows/ci.yml；tests/ci/quality_route.py；tests/ci/run_repository_gates.py；docs/reference/ci-quality-gates.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/adopter-capability-manifest.schema.json` | reference-only | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；源 manifest/schema 保持 reference/provider material，不宣称 Rust 有完整 adopter manifest。` |
| `.ai/schemas/cross-wi-integration-report.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/evidence-binding.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/governance-cost-report.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/implementation-knowledge-dependency-index.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/implementation-knowledge-index.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/implementation-knowledge-query.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/implementation-knowledge-record.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/parallel-verification-plan.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/performance-diagnosis-report.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/task_outcome.schema.json` | implemented-different-by-design | `crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/src/lib.rs；crates/cockpit-mcp/src/lib.rs；docs/reference/outcome-report.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.ai/schemas/work-item-status-interface.schema.json` | implemented-different-by-design | `crates/cockpit-repository/src/lib.rs；crates/cockpit-protocol/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | `.github/workflows/ci.yml；tests/ci/quality_route.py；tests/ci/run_repository_gates.py；tests/release/adopter_acceptance.sh；docs/capabilities.md；docs/release/distribution.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.github/workflows/release.yml` | implemented-different-by-design | `.github/workflows/release.yml；tests/release/workflow_policy.sh；tests/release/version_consistency.sh；tests/release/adopter_acceptance.sh；tests/release/adopter_upgrade_acceptance.sh；docs/release/distribution.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `.github/workflows/smoke.yml` | implemented-different-by-design | `.github/workflows/ci.yml；.github/workflows/release.yml；tests/ci/repository_gate_manifest.json；tests/release/adopter_acceptance.sh；tests/release/adopter_upgrade_acceptance.sh；docs/reference/reference-file-comparison.md；docs/release/distribution.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `AGENTS.md` | implemented-different-by-design | `AGENTS.md；.ai/README.md；docs/reference/agent-workflow.md；crates/cockpit-agent/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `GEMINI.md` | implemented-different-by-design | `.ai/README.md；crates/cockpit-agent/src/lib.rs；crates/cockpit-agent/tests/install.rs；docs/reference/agent-workflow.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `Makefile` | implemented-different-by-design | `.github/workflows/ci.yml；tests/ci/run_repository_gates.py；docs/reference/commands.md；Cargo.toml；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/README.ja.md` | implemented-different-by-design | `docs/README.ja.md；docs/current/README.ja.md；docs/getting-started/README.ja.md；docs/reference/README.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/README.md` | implemented-different-by-design | `docs/README.md；docs/current/README.md；docs/getting-started/README.md；docs/reference/README.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/README.zh-CN.md` | implemented-different-by-design | `docs/README.zh-CN.md；docs/current/README.zh-CN.md；docs/getting-started/README.zh-CN.md；docs/reference/README.zh-CN.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/capabilities.ja.md` | implemented-different-by-design | `docs/capabilities.ja.md；docs/reference/capability-truth-matrix.md；docs/reference/commands.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/capabilities.md` | implemented-different-by-design | `docs/capabilities.md；docs/reference/capability-truth-matrix.md；docs/reference/commands.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/capabilities.zh-CN.md` | implemented-different-by-design | `docs/capabilities.zh-CN.md；docs/reference/capability-truth-matrix.md；docs/reference/commands.zh-CN.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/human-benefit-report.ja.md` | implemented-different-by-design | `docs/features/human-benefit-report.ja.md；docs/features/task-outcome-report.ja.md；docs/reference/outcome-report.ja.md；docs/reference/task-outcome-events.ja.md；crates/cockpit-cli/tests/outcome_handoff.rs；crates/cockpit-mcp/tests/rpc.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/human-benefit-report.md` | implemented-different-by-design | `docs/features/human-benefit-report.md；docs/features/task-outcome-report.md；docs/reference/outcome-report.md；docs/reference/task-outcome-events.md；crates/cockpit-cli/tests/outcome_handoff.rs；crates/cockpit-mcp/tests/rpc.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/human-benefit-report.zh-CN.md` | implemented-different-by-design | `docs/features/human-benefit-report.zh-CN.md；docs/features/task-outcome-report.zh-CN.md；docs/reference/outcome-report.zh-CN.md；docs/reference/task-outcome-events.zh-CN.md；crates/cockpit-cli/tests/outcome_handoff.rs；crates/cockpit-mcp/tests/rpc.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/task-outcome-report.ja.md` | implemented-different-by-design | `docs/features/task-outcome-report.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/task-outcome-report.md` | implemented-different-by-design | `docs/features/task-outcome-report.md；docs/reference/outcome-report.md；crates/cockpit-repository/src/lib.rs；crates/cockpit-cli/src/main.rs；crates/cockpit-mcp/src/lib.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/task-outcome-report.zh-CN.md` | implemented-different-by-design | `docs/features/task-outcome-report.zh-CN.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/work-item-parallelism.ja.md` | implemented-different-by-design | `docs/work-items/WI-123-parallel-contract-boundary.ja.md；docs/reference/configuration.ja.md；crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/tests/parallel_boundary.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/work-item-parallelism.md` | implemented-different-by-design | `docs/work-items/WI-123-parallel-contract-boundary.md；docs/reference/configuration.md；crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/tests/parallel_boundary.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/features/work-item-parallelism.zh-CN.md` | implemented-different-by-design | `docs/work-items/WI-123-parallel-contract-boundary.zh-CN.md；docs/reference/configuration.zh-CN.md；crates/cockpit-protocol/src/lib.rs；crates/cockpit-repository/tests/parallel_boundary.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/first-work-item.ja.md` | implemented-different-by-design | `docs/getting-started/first-work-item.ja.md；docs/reference/agent-workflow.ja.md；docs/reference/outcome-report.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/first-work-item.md` | implemented-different-by-design | `docs/getting-started/first-work-item.md；docs/reference/agent-workflow.md；docs/reference/outcome-report.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/first-work-item.zh-CN.md` | implemented-different-by-design | `docs/getting-started/first-work-item.zh-CN.md；docs/reference/agent-workflow.zh-CN.md；docs/reference/outcome-report.zh-CN.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/security-release-verification.ja.md` | implemented-different-by-design | `docs/getting-started/security-release-verification.ja.md；docs/release/distribution.ja.md；docs/getting-started/installation-security.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/security-release-verification.md` | implemented-different-by-design | `docs/getting-started/security-release-verification.md；docs/release/distribution.md；docs/getting-started/installation-security.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/security-release-verification.zh-CN.md` | implemented-different-by-design | `docs/getting-started/security-release-verification.zh-CN.md；docs/release/distribution.zh-CN.md；docs/getting-started/installation-security.zh-CN.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/standard-adoption-guide.ja.md` | implemented-different-by-design | `docs/getting-started/standard-adoption-guide.ja.md；docs/getting-started/installation.ja.md；docs/getting-started/first-work-item.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/standard-adoption-guide.md` | implemented-different-by-design | `docs/getting-started/standard-adoption-guide.md；docs/getting-started/installation.md；docs/getting-started/first-work-item.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/getting-started/standard-adoption-guide.zh-CN.md` | implemented-different-by-design | `docs/getting-started/standard-adoption-guide.zh-CN.md；docs/getting-started/installation.zh-CN.md；docs/getting-started/first-work-item.zh-CN.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/maintainers/task-outcome-events.md` | implemented-different-by-design | `docs/reference/task-outcome-events.md；docs/reference/task-outcome-events.zh-CN.md；docs/reference/task-outcome-events.ja.md；docs/features/task-outcome-report.md；crates/cockpit-repository/src/lib.rs；crates/cockpit-repository/tests/task_outcome_events.rs；crates/cockpit-cli/tests/outcome_handoff.rs；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/operations/quality-gates.ja.md` | implemented-different-by-design | `docs/reference/ci-quality-gates.ja.md；docs/reference/governance-integrity-gate.ja.md；tests/ci/repository_gate_manifest.json；tests/ci/run_repository_gates.py；.github/workflows/ci.yml；docs/release/distribution.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/operations/quality-gates.md` | implemented-different-by-design | `docs/reference/ci-quality-gates.md；docs/reference/governance-integrity-gate.md；tests/ci/repository_gate_manifest.json；tests/ci/run_repository_gates.py；.github/workflows/ci.yml；docs/release/distribution.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/operations/quality-gates.zh-CN.md` | implemented-different-by-design | `docs/reference/ci-quality-gates.zh-CN.md；docs/reference/governance-integrity-gate.zh-CN.md；tests/ci/repository_gate_manifest.json；tests/ci/run_repository_gates.py；.github/workflows/ci.yml；docs/release/distribution.zh-CN.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/operations/work-item-lifecycle.ja.md` | implemented-different-by-design | `docs/reference/agent-workflow.ja.md；docs/reference/outcome-report.ja.md；docs/reference/reference-file-comparison.ja.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/operations/work-item-lifecycle.md` | implemented-different-by-design | `docs/reference/agent-workflow.md；docs/reference/outcome-report.md；docs/reference/reference-file-comparison.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/operations/work-item-lifecycle.zh-CN.md` | implemented-different-by-design | `docs/reference/agent-workflow.zh-CN.md；docs/reference/outcome-report.zh-CN.md；docs/reference/reference-file-comparison.zh-CN.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/reference/adoption-reality-report.md` | implemented-different-by-design | `docs/capabilities.md；docs/release/distribution.md；docs/security/enterprise-governance.md；crates/cockpit-repository/src/project_governance.rs；crates/cockpit-repository/tests/project_governance.rs；tests/release/adopter_acceptance.sh；不复制源字节和命令，语义由 Runtime/读者路线承载。` |
| `docs/reference/agent-parallel-work-items.md` | implemented-different-by-design | `docs/reference/cross-work-item-dedup.md；docs/reference/affected-verification.md；docs/reference/agent-workflow.md；AGENTS.md；.ai/README.md；不复制源字节和命令，语义由 Runtime/读者路线承载。` |


## 对象工程继承

已 attach 的对象工程继承同一份共享 Runtime、显式 `--repo` 请求上下文、隔离的 Contract/evidence/knowledge、动态验证路线、fail-closed 生命周期和可见的人类 Outcome。不会继承源 Python 模块、Make target、provider 全局配置、交互式技术栈安装器、生成历史或源 JSON 线格式。

## 验证

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --apply-wi629-batch --source-commit a9224aed77b5c317b53c4551a9eec306d91ee330 --target-commit 98f12b18b978db509fc884a8a6225afeb7f10df5
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit a9224aed77b5c317b53c4551a9eec306d91ee330 --target-commit 98f12b18b978db509fc884a8a6225afeb7f10df5
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/docs/reference_comparison_metadata_test.py
python3 tests/conformance/reference_inventory_docs_test.py
```

参见：[English](WI-629-reference-rebaseline-batch-51.md) · [日本語](WI-629-reference-rebaseline-batch-51.ja.md)
