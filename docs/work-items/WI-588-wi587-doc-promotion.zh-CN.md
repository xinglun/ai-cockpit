---
author: AI Cockpit maintainers
title: "WI-588——WI-587 终态文档 promotion"
description: "在 WI-587 关闭后将其三语文档投影提升为终态。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-588-wi587-doc-promotion
lastVerifiedBy: WI-588-wi587-doc-promotion
---

[English](WI-588-wi587-doc-promotion.md) · [日本語](WI-588-wi587-doc-promotion.ja.md)

# WI-588——WI-587 终态文档 promotion

## 目标

在 WI-587 的不可变 archive、verification、finalization 和 close 回执有效
后，提升其三语 Work Item 与 reference-parity 文档投影。本 WI 只修改文档投影。

## 边界

Runtime 行为、对象工程、全局 Agent/MCP 配置以及生成的 evidence/decision 回执
不在范围内。Contract acceptance 仍以其编写语言为准。

## 验收

1. 三个 WI-587 Work Item 页面包含由不可变回执导出的终态路径。
2. 三个 reference-parity 行将 WI-587 报告为已实现，并包含匹配的终态证据路径。
3. 不修改治理事实、源实现、对象工程或生成的回执字节。

## 验证

在显式 repository context 下运行 `tests/docs/promote_closed_work_item.py --check`、
`tests/docs/documentation_acceptance.sh` 和 `tests/docs/parity_status_check.sh`。
