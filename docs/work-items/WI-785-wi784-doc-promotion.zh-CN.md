---
author: AI Cockpit maintainers
title: "WI-785 — WI-784 文档投影恢复"
description: "通过 Runtime 有效的 WI-784 successor 完成 WI-783 终态文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-successor-recovery
workItemId: WI-785-wi784-doc-promotion
lastVerifiedBy: WI-785-wi784-doc-promotion
---

[English](WI-785-wi784-doc-promotion.md) · [日本語](WI-785-wi784-doc-promotion.ja.md)

# WI-785 — WI-784 文档投影恢复

## 意图与边界

WI-785 是在 `finish.preflight` 停止的 WI-784 尝试的明确 successor。它使用
`authority: authorized` 及受支持的 `verification` evidence class，完成已关闭
WI-783 证据的有界三语言文档投影。

WI-784 的 Contract、Summary、Outcome、verification 和 recovery 记录属于不可变
的 predecessor evidence。本 Work Item 不改写这些记录，也不修改 Runtime 源码、
产品行为、发布状态或无关文档。

## 验证

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-783-parity-finalization-recovery --check`

`bash tests/docs/documentation_acceptance.sh --repo <repo>`

Successor 绑定为 `.ai/decisions/WI-784-wi783-doc-promotion.recovery.json`。
