---
author: AI Cockpit maintainers
title: "WI-247——WI-246 close parity 登记"
workItemId: WI-247-parity-close-registration
description: "把不可变 WI-246 终态决定链投影到三语 parity ledger。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-247-parity-close-registration
authority: canonical
---

# WI-247——WI-246 close parity 登记

PR #197 合并、governance append 被观察、准确 branch 与 feature worktree 被移除后，
WI-246 正确关闭。权威 close receipt 持久化后暴露 ledger 顺序缺口：三语 parity 行仍把
WI-246 描述为“进行中”，并且只列 canonical 合并前 receipt。因此 gate 正确报告
`missing_parity_decision` 与 `stale_parity_status`。

## 恢复边界

Runtime 生成的 recovery receipt 绑定准确 WI-246 Contract、Summary、Outcome、Events、
finalization chain 与 close identity。这些记录、PR #197 与 merge commit `98d6575` 均不可变。
WI-247 只修改英文、简体中文与日文 parity/Work Item 投影；不修改 Runtime、CI、release、
tests、crates 或 WI-241。

## 验收与验证

每条 WI-246 parity 行改为“已实现”，保留 canonical 合并前 receipt，并加入 sequence-1
合并观察、sequence-2 cleanup、close 与 recovery 路径。聚焦 parity、governance、manifest、
documentation 检查与 canonical strict repository runner 必须通过。真实 draft PR #198 在
verification 前绑定，Runtime lifecycle records 与文档变更保持分离。
