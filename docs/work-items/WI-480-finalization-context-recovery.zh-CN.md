---
author: AI Cockpit maintainers
title: "WI-480——finalization 上下文恢复门禁"
description: "在终态生命周期步骤前拒绝模糊的临时资源上下文。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: human-authorized
lastVerifiedBy: WI-480-finalization-context-recovery
terminalArchive: .ai/work-items/archive/WI-480-finalization-context-recovery.contract.json
terminalVerification: .ai/evidence/WI-480-finalization-context-recovery.verification.json
terminalFinalization: .ai/decisions/WI-480-finalization-context-recovery.finalize.json
terminalDecision: .ai/decisions/WI-480-finalization-context-recovery.close.json
workItemId: WI-480-finalization-context-recovery
---

# WI-480——finalization 上下文恢复门禁

本 Runtime 变更将裸 `pending` provider 哨兵与已有的
`pending:<stable-reference>` 规则统一为临时上下文。只有显式
`finalize-plan` 绑定真实的已评审资源后，`finish` 和 `archive` 才能继续。
WI-479 的不可变记录通过 append-only successor 恢复，不会被改写。

[English](WI-480-finalization-context-recovery.md) · [日本語](WI-480-finalization-context-recovery.ja.md)

## 范围

- 将精确的 `pending` 哨兵分类为临时上下文；
- 增加 finish 拒绝和可恢复性的 protocol/lifecycle 回归测试；
- 三语说明显式 finalization 边界。

## 不在范围内

版本发布、对象工程、foreign Runtime 策略，以及任何 WI-479 Contract、证据、archive、Outcome、事件或 recovery bytes 的改写。

## 验证

- `cargo test --locked -p cockpit-protocol --test resource_finalization`
- `cargo test --locked -p cockpit-repository --test archive_integrity`
- `cargo test --locked --workspace`
- `cargo fmt --all -- --check`
