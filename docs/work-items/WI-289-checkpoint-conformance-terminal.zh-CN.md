---
author: AI Cockpit maintainers
title: "WI-289——Checkpoint 一致性终态恢复"
workItemId: WI-289-checkpoint-conformance-terminal
description: "在 hosted 文档真相门阻断后重新交付有界 checkpoint 一致性批次，不改写 predecessor bytes。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-289-checkpoint-conformance-terminal
authority: canonical
---

# WI-289——Checkpoint 一致性终态恢复

## 目的

WI-288 作为不可变恢复历史保留，因为 hosted quality 发现恢复后的 WI-287
文档在归档后仍标记为 `in_progress`。本 successor 保留同一有界实现，并在
验证前修正三语文档状态与 parity 投影。

## 边界

- 保留 WI-287、WI-288 的 archive、evidence、recovery 和 finalization bytes。
- 不改变 Rust-native checkpoint 与 Agent 规则实现。
- 在 archive 前修正三语文档和 parity 投影。
- 在 verify 前绑定新的 Provider PR，完成 hosted checks、finalization、close
  和精确资源清理。

## 对象工程能力一致性

本仓库与全新 adopter 工程必须使用同一份已安装 Runtime、显式 repository
context、fail-closed lifecycle 和可见的人类 Outcome。

## 验证

执行 `cargo test --locked --workspace`、conformance inventory、documentation/
governance integrity gates、hosted PR checks、Provider finalization verify、
close，以及 close 后的 status/doctor 检查。
