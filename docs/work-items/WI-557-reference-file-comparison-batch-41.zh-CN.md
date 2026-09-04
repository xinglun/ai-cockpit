---
author: AI Cockpit maintainers
title: "WI-557——参考脚本比对批次 41"
description: "逐个比对 13 个延期的参考脚本，记录 Rust-native 语义边界。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-557-reference-file-comparison-batch-41
lastVerifiedBy: WI-557-reference-file-comparison-batch-41
---

[English](WI-557-reference-file-comparison-batch-41.md) · [日本語](WI-557-reference-file-comparison-batch-41.ja.md)

# WI-557——参考脚本比对批次 41

## 目标

针对 pinned local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`，逐个比对本批次指定的 13 个脚本，
记录可迁移职责、Rust-native 对应能力和有意保留的边界。这里做的是语义一致性，
不是复制源代码或 JSON wire format。

## 范围

`scripts/ai_issue_log.py`、`scripts/ai_linked_worktree_recovery.py`、
`scripts/ai_ownership.py`、`scripts/ai_performance_budget.py`、
`scripts/ai_project_profile.py`、`scripts/ai_purge.py`、
`scripts/ai_readiness_policy.py`、`scripts/ai_recovery_usability.py`、
`scripts/ai_review_readiness_policy.py`、`scripts/ai_risk_policy.py`、
`scripts/ai_rollback.py`、`scripts/ai_safety_gate.py`、
`scripts/ai_schema_migration.py`，以及 Contract 指定的目标台账、检查脚本和三语比对/对齐文档。

## 边界

Python 模块、源测试、源 registry 和源 JSON wire format 仍是参考资料。共享 Rust
Runtime、repository-local Protocol、外部 provider 边界和对象工程不在本批次修改。
目标没有通用的固定 recovery 场景 registry，因此 `ai_recovery_usability.py` 明确标记
为 reference-only，而不是伪装成等价实现。

## 验收

- 每个指定源路径都有且只有一个明确的 `WI-557` 台账记录。
- 每条记录都有非空分类、Rust 对应路径和基于证据的理由；不得保留 deferred 或 migrate gap。
- 台账回归检查与三语比对/对齐文档对 13 个路径和 pinned source commit 保持一致。
- 不复制参考实现，不修改对象工程，也不修改全局 Agent 配置。
- 参考台账、文档、治理完整性和 diff 检查全部通过。

## 验证

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
