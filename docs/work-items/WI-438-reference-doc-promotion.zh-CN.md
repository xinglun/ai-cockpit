---
author: AI Cockpit maintainers
title: "WI-438——已关闭 WI-437 文档投影晋级"
workItemId: WI-438-reference-doc-promotion
description: "晋级已关闭 WI-437 治理重新比对的三语文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-438-reference-doc-promotion
---

# WI-438——已关闭 WI-437 文档投影晋级

这是一个仅文档 Work Item，在 WI-437 完成评审合并、资源终结和关闭后运行仓库自有的 promotion helper。
语义参考源保持为维护者本地 checkout
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`，不访问公开参考仓库，也不改变 Runtime 行为。

[English](WI-438-reference-doc-promotion.md) · [日本語](WI-438-reference-doc-promotion.ja.md)

## 范围

- 晋级 WI-437 的三语 Work Item 文档和三语 reference-parity 行。
- 保持本 Work Item 自身三语文档及 pre-archive parity 行，使文档门能够审计自身生命周期。
- 不重写不可变的 archive、verification、finalization 或 close bytes。

## 验证

必须通过 `tests/docs/promote_closed_work_item.py --work-item
WI-437-reference-rebaseline-governance` 与 `--check-all`、文档验收、parity/status 检查、governance integrity
以及 Contract 声明的 Runtime 验证。改动必须保持在本 Contract 范围内。
