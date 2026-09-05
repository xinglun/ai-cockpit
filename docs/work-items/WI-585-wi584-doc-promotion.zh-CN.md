---
author: AI Cockpit maintainers
title: "WI-585——WI-584 终态文档晋级"
description: "在 WI-584 关闭后晋级其三语文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-585-wi584-doc-promotion
lastVerifiedBy: WI-585-wi584-doc-promotion
---

[English](WI-585-wi584-doc-promotion.md) · [日本語](WI-585-wi584-doc-promotion.ja.md)

# WI-585——WI-584 终态文档晋级

## 目标

仅在 WI-584 的不可变 archive、verification、finalization 和 close receipt
有效后，晋级其三语 Work Item 与 reference-parity 文档投影。本 Work Item
只修改文档投影，不修改治理事实或 Runtime 行为。

## 边界

对象工程、Runtime 实现、全局 Agent/MCP 配置及生成的 evidence/decision 字节均
不在本 Work Item 范围内。Contract 的 acceptance 仍以编写语言为权威。

## 验收

1. 三个 WI-584 Work Item 页面包含由不可变 receipt 得出的终态路径。
2. 三语 reference-parity 行将 WI-584 标记为已实现并绑定对应 evidence 路径。
3. 不修改治理事实、源代码、对象工程或生成的 receipt 字节。

## 验证

使用显式 repository context 运行 `tests/docs/documentation_acceptance.sh` 以及当前
Contract 声明的 Runtime 验证命令。
