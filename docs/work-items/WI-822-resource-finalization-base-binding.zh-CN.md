---
author: AI Cockpit 维护者
title: "WI-822——resource finalization base binding"
description: "将 provider finalization 绑定到实际审查 PR 的 base，并保留 close recovery。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-822-resource-finalization-base-binding
lastVerifiedBy: WI-822-resource-finalization-base-binding
terminalArchive: .ai/work-items/archive/WI-822-resource-finalization-base-binding.contract.json
terminalVerification: .ai/evidence/WI-822-resource-finalization-base-binding.verification.json
terminalFinalization: .ai/decisions/WI-822-resource-finalization-base-binding.finalize.json
terminalDecision: .ai/decisions/WI-822-resource-finalization-base-binding.close.json
---

[English](WI-822-resource-finalization-base-binding.md) · [日本語](WI-822-resource-finalization-base-binding.ja.md)

# WI-822——resource finalization base binding

## 意图与边界

WI-822 分离 Contract base、审查 PR 的比较 base 与发布执行身份。只有在已合并 PR、准确分支
和准确工作树状态都验证后，才记录 provider cleanup。前置项 archive 及其历史 evidence
保持不变。

## 验证

Runtime 正式验证通过计划中的全部 12 个 workspace 节点。provider finalization receipt
记录了已合并 PR 及准确分支/工作树清理，close decision 绑定该 finalization head。
