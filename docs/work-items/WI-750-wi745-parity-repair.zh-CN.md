---
author: AI Cockpit maintainers
title: "WI-750——WI-745 parity 投影修复"
description: "在不改变治理 receipt 的前提下修复剩余的当前基线 parity 和终态页面投影。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-750-wi745-parity-repair
status: in_progress
authority: authorized
lastVerifiedBy: WI-750-wi745-parity-repair
---

[English](WI-750-wi745-parity-repair.md) · [日本語](WI-750-wi745-parity-repair.ja.md)

# WI-750——WI-745 parity 投影修复

## 目的

修复三语 parity 和 Work Item 投影，使其指向已存在于同步
`origin/main` 上的有效 supersede、finalization 和 close receipt。

## 边界

本 Work Item 仅修改文档投影，不修改生产代码、测试、治理规则、不可变
receipt、archive 或 verification bytes，也不改变其他 Work Item 的生命周期事实。

## 验证

在 finish 前通过 repository-bound Runtime 执行文档、parity、一致性、promotion
和治理完整性检查。本修复不从文档事实推断用户可见收益。
