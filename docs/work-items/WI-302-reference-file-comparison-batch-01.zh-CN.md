---
author: AI Cockpit maintainers
title: "WI-302——第一批延后参考文件对比"
workItemId: WI-302-reference-file-comparison-batch-01
description: "将前十个延后的参考源文件与 Rust 目标逐项对比，并记录有边界的语义结论。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-302-reference-file-comparison-batch-01
terminalArchive: .ai/work-items/archive/WI-302-reference-file-comparison-batch-01.contract.json
terminalVerification: .ai/evidence/WI-302-reference-file-comparison-batch-01.verification.json
terminalFinalization: .ai/decisions/WI-302-reference-file-comparison-batch-01.finalize.json
terminalDecision: .ai/decisions/WI-302-reference-file-comparison-batch-01.close.json
authority: canonical
---

# WI-302——第一批延后参考文件对比

## 意图

以固定源提交 `e5acb677` 为基线，按字典序逐项对比前十个延后的参考源记录，
保持可迁移的治理语义与源语言或 provider 专属实现之间的边界。

## 范围与结果

本批次覆盖 `.ai/cockpit/bandit_low_risk_baseline.json`、`.gitattributes`、选定的
三个 GitHub 元数据/工作流文件、`.gitignore`、`LICENSE` 与 `Makefile`。inventory
为每个文件记录源职责、Rust 对应物或缺失、分类和理由。兼容性与 smoke 工作流矩阵
明确延后，因为它们需要独立的多技术栈与第二 adopter 对比。

同步后的 ledger 与三语报告为：

- `tests/conformance/reference_file_inventory.json`
- `docs/reference/reference-file-comparison.md`
- `docs/reference/reference-file-comparison.zh-CN.md`
- `docs/reference/reference-file-comparison.ja.md`

## 证据边界

目标基线为 `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`；验证由已安装 Runtime `0.2.33`
执行，binary digest 为
`sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`。归档与验证
记录负责生命周期事实；本文档是可读投影，不新增源语言 Runtime 或 provider 所有权策略。
