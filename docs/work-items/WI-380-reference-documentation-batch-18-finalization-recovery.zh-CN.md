---
author: AI Cockpit maintainers
title: "WI-380——WI-379 provider finalization 恢复"
description: "绑定评审后的 successor 交付，在不改写 WI-379 历史的前提下关闭文档批次。"
workItemId: WI-380-reference-documentation-batch-18-finalization-recovery
canonical: docs/work-items/WI-380-reference-documentation-batch-18-finalization-recovery.md
audience: [maintainer, reviewer]
status: implemented
authority: translation
lastVerifiedBy: WI-380-reference-documentation-batch-18-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-380-reference-documentation-batch-18-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-380-reference-documentation-batch-18-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.close.json
capabilityClaims: [governance_integrity, resource_finalization]
---

# WI-380——WI-379 provider finalization 恢复

[English](WI-380-reference-documentation-batch-18-finalization-recovery.md) · [日本語](WI-380-reference-documentation-batch-18-finalization-recovery.ja.md)

## 意图与边界

WI-379 已在评审 PR #343 中交付参考文档，但在 provider PR identity 确定前就已归档。
本显式 successor 保留 WI-379 的不可变 archive、evidence、Outcome 和恢复决定，
并为恢复本身记录真实的 provider 绑定生命周期。

## 范围

- 保持 WI-379 predecessor 摘要和恢复 lineage 可见。
- 在三语 parity 文档中将 WI-379 标记为已恢复并登记本 successor。
- 在 verification 前绑定本 Work Item 的实际评审 PR context。
- 在 close 前证明精确的分支/工作树清理。

## 不在范围内

Runtime 代码、Release artifact、全局 Agent/MCP 配置，以及 WI-379 的任何不可变
archive/evidence/Outcome/PR bytes。

## 验收

- 恢复决定绑定 predecessor 的 Contract、Summary、Outcome、Events、repository 和 Runtime identity。
- WI-379 bytes 保持不变，并明确标记为 historical/recovered。
- successor PR context 在记录 verification evidence 前完成绑定。
- Hosted checks、安装版 Runtime 验证、finalization、close 和可见的人类 Outcome 全部通过。

## 验证与终态记录

使用带显式 `--repo` 的安装版 Runtime、文档/治理检查和
`cargo test --locked --workspace`。评审合并后记录：

- Archive：`.ai/work-items/archive/WI-380-reference-documentation-batch-18-finalization-recovery.contract.json`
- Verification：`.ai/evidence/WI-380-reference-documentation-batch-18-finalization-recovery.verification.json`
- Finalization：`.ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.finalize.json`
- Close：`.ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.close.json`
