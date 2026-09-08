---
author: AI Cockpit 维护者
title: “WI-702——WI-701 parity 顺序恢复”
description: “在 verification evidence 之前提交 parity 注册，重新交付 WI-701 恢复。”
audience: [contributor, maintainer, reviewer]
workItemId: WI-702-wi701-parity-order-recovery
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-702-wi701-parity-order-recovery
---

[English](WI-702-wi701-parity-order-recovery.md) · [日本語](WI-702-wi701-parity-order-recovery.ja.md)

# WI-702——WI-701 parity 顺序恢复

WI-702 是不可变失败 WI-701 交付的新 successor。它绑定记录中的
`origin/main` 基线，并在生成新的 verification evidence 之前登记 parity 投影。

## 边界

本 Work Item 保留 WI-701、WI-700 和 WI-698 的 archive 与 evidence，不引入新的代码
语义、治理规则或协议格式，也不修改其他 agent 的 Work Item。

## 验收

- recovery decision 与前件 digest 绑定替代交付。
- 三语 parity 注册在提交历史上先于 WI-702 verification evidence。
- 在 Work Item 进入终态前，必须有 fresh verification、hosted review、provider
  finalization、archive、close 和准确清理证据。
