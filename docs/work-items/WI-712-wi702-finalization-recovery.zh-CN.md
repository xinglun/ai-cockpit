---
author: AI Cockpit maintainers
title: "WI-712——WI-702 合并后 finalization 恢复"
description: "在不改写不可变历史的前提下修复已合并 WI-702 的资源边界。"
workItemId: WI-712-wi702-finalization-recovery
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-712-wi702-finalization-recovery
predecessorWorkItem: WI-702-p2-incremental-merkle-trust-audit
recoveryDecision: .ai/decisions/WI-702-p2-incremental-merkle-trust-audit.recovery.json
terminalArchive: .ai/work-items/archive/WI-712-wi702-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-712-wi702-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-712-wi702-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-712-wi702-finalization-recovery.close.json
---

# WI-712——WI-702 合并后 finalization 恢复

WI-712 是 WI-702 合并后资源边界的有界 successor。WI-702 的不可变 archive 和
pre-merge finalization root 继续作为历史事实；WI-712 记录 recovery、新的验证、
parity 注册和 PR #700 合并后的准确 cleanup。

[English](WI-712-wi702-finalization-recovery.md) · [日本語](WI-712-wi702-finalization-recovery.ja.md)

## 恢复事实

PR #700 从 reviewed head `4eedf23ddf1e4a0491fb978127d61d852e6a5a1f` 合并为
`bd00a7ce888c2d0dba012da21ba1616eeeab0014`。前项 root 绑定 `86f8535f`，而其间
range 包含 pending-parity-registry 修改，因此 Runtime 拒绝伪造 append-only
transition。successor 保留前项 bytes，不声称性能收益。

## 范围与证据

- 保留 WI-702 的 archive、evidence、Outcome、Events、Contract 和 finalization。
- 绑定 `.ai/decisions/WI-702-p2-incremental-merkle-trust-audit.recovery.json`。
- 增加并验证三语 parity projection，然后消费临时 `pending-parity-registry.json` 条目。
- 完成 WI-712 Runtime lifecycle 和准确资源 finalization。
- 验证证据为 `.ai/evidence/WI-712-wi702-finalization-recovery.verification.json`；
  finalization 与 close 记录由 Runtime 生成。
