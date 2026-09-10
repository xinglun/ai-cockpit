---
author: AI Cockpit maintainers
title: "WI-770——WI-769 性能终态文档恢复"
description: "完成 WI-769 的有界恢复后继项，不改变性能行为或前置证据。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-770-performance-terminal-docs-recovery
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-770-performance-terminal-docs-recovery
terminalArchive: .ai/work-items/archive/WI-770-performance-terminal-docs-recovery.contract.json
terminalVerification: .ai/evidence/WI-770-performance-terminal-docs-recovery.verification.json
terminalFinalization: .ai/decisions/WI-770-performance-terminal-docs-recovery.finalize.json
terminalDecision: .ai/decisions/WI-770-performance-terminal-docs-recovery.close.json
---

[English](WI-770-performance-terminal-docs-recovery.md) · [日本語](WI-770-performance-terminal-docs-recovery.ja.md)

# WI-770——WI-769 性能终态文档恢复

## Recovery 边界

WI-770 是不可变 WI-769 文档尝试的有界后继项。它保持前置项 archive、证据和 recovery decision 不变，
并从已合并的 `main` revision 开始新的 Runtime 生命周期。评审后的 PR #753 是本次 provider-side 变更的
对账对象；本 WI 不增加 production、Runtime 或性能行为。

## 文档边界

后继项负责三语 Work Item 页面和匹配的 parity 行。WI-769 页面将前置项投影为 `recovered`；在 WI-770
完成新鲜 verification、archive、finalization、finalization revalidation 和 close 证据前，本页保持
`in_progress`。不声称性能收益，并保留 `user_visible_benefit_not_declared`。

## 证据与生命周期

- Recovery decision：`.ai/decisions/WI-769-performance-terminal-docs.recovery.json`。
- 新鲜 verification：`.ai/evidence/WI-770-performance-terminal-docs-recovery.verification.json`。
- Archive、finalization 和 close 记录由 Runtime 分别生成，作为独立生命周期证据。
- 前置记录保持不可变；只有本有界 projection 与后继项 Runtime 记录在范围内。
