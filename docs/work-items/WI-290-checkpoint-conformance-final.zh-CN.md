---
author: AI Cockpit maintainers
title: "WI-290——Checkpoint 一致性最终交付"
workItemId: WI-290-checkpoint-conformance-final
description: "从最新远端默认分支重新交付有界 checkpoint 一致性批次，不改写 predecessor bytes。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-290-checkpoint-conformance-final
terminalArchive: .ai/work-items/archive/WI-290-checkpoint-conformance-final.contract.json
terminalVerification: .ai/evidence/WI-290-checkpoint-conformance-final.verification.json
terminalFinalization: .ai/decisions/WI-290-checkpoint-conformance-final.finalize.85d018bce51ab697f9c5221fe5cb72440386baacf0b44063371653ec221a254c.json
terminalDecision: .ai/decisions/WI-290-checkpoint-conformance-final.close.json
authority: canonical
---

# WI-290——Checkpoint 一致性最终交付

## 目的

WI-287、WI-288 和 WI-289 作为不可变恢复历史保留，因为 hosted gate 发现
交付绑定存在问题。本 successor 保留同一有界实现，从最新远端默认分支开始，
并在验证前登记完整的三语文档生命周期证据。

## 边界

- 保留 WI-287、WI-288、WI-289 的 archive、evidence、recovery 和 finalization bytes。
- 不改变 Rust-native checkpoint 与 Agent 规则实现。
- 在 archive 前登记完整的三语文档和 parity 生命周期路径。
- 在 verify 前绑定新的 Provider PR，完成 hosted checks、finalization、close
  和精确资源清理。

## 对象工程能力一致性

本仓库与全新 adopter 工程必须使用同一份已安装 Runtime、显式 repository
context、fail-closed lifecycle 和可见的人类 Outcome。

## 验证

执行 `cargo test --locked --workspace`、conformance inventory、documentation/
governance integrity gates、hosted PR checks、Provider finalization verify、
close，以及 close 后的 status/doctor 检查。
