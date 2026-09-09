---
author: AI Cockpit maintainers
title: "WI-746——WI-745 终态恢复"
description: "在不改写历史证据的前提下协调已合并 WI-745 文档 successor。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-746-wi745-terminal-recovery
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-746-wi745-terminal-recovery
predecessorWorkItem: WI-745-wi743-doc-promotion
recoveryDecision: .ai/decisions/WI-745-wi743-doc-promotion.recovery.json
---

# WI-746——WI-745 终态恢复

WI-746 是已合并 WI-745 文档晋级的当前基线恢复 successor。它保留 WI-745 和
WI-743 的历史 bytes，修复三语投影，并在 Runtime finalization 和 close 前记录新的验证。
本 Work Item 不修改生产行为或性能。

[English](WI-746-wi745-terminal-recovery.md) · [日本語](WI-746-wi745-terminal-recovery.ja.md)

## 恢复边界

Runtime recovery receipt 绑定 WI-745、其 Contract 和 Summary digest、PR #718 及本
successor。WI-743 继续由不可变的 closed evidence 表示；当前状态 prose 已与该终态事实一致。

## 证据与生命周期

- 新的验证记录为 `.ai/evidence/WI-746-wi745-terminal-recovery.verification.json`。
- archive、finalization、Outcome 和 close 记录由 Runtime 生成。
- 本恢复不声明用户可见的性能收益。
