---
author: AI Cockpit maintainers
title: "WI-399——v0.2.40 文档晋升恢复"
description: "在专用 worktree 中恢复 WI-398 交付并保留可审计的发布基线。"
workItemId: WI-399-release-v0-2-40-doc-promotion-recovery
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-399-release-v0-2-40-doc-promotion-recovery
terminalArchive: .ai/work-items/archive/WI-399-release-v0-2-40-doc-promotion-recovery.contract.json
terminalVerification: .ai/evidence/WI-399-release-v0-2-40-doc-promotion-recovery.verification.json
terminalFinalization: .ai/decisions/WI-399-release-v0-2-40-doc-promotion-recovery.finalize.json
terminalDecision: .ai/decisions/WI-399-release-v0-2-40-doc-promotion-recovery.close.json
capabilityClaims: [documentation_governance, release_distribution]
---

# WI-399——v0.2.40 文档晋升恢复

[English](WI-399-release-v0-2-40-doc-promotion-recovery.md) · [日本語](WI-399-release-v0-2-40-doc-promotion-recovery.ja.md)

## 意图

WI-398 的 finalization 在主 worktree 上被 Runtime 正确拒绝后，本 Work Item
在专用 worktree 中恢复交付。它保留 WI-398 的不可变归档与恢复决定，不改写历史证据。

## 边界

本 Work Item 只处理恢复决定、三语 WI-399 文档和 reference-parity 登记。
不修改 Runtime 语义、发布实现、公开 adopter 验收或历史 evidence bytes。

## 验证与交付

归档前必须通过文档验收、仓库治理检查和 locked workspace 全量测试。审查 PR
合并后才删除分支及专用 worktree，随后记录精确清理结果并关闭 successor。
