---
author: AI Cockpit maintainers
title: "WI-784 — WI-783 终态文档投影"
description: "将 WI-783 的已关闭证据投影为有界的三语言文档记录。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-784-wi783-doc-promotion
lastVerifiedBy: WI-784-wi783-doc-promotion
---

[English](WI-784-wi783-doc-promotion.md) · [日本語](WI-784-wi783-doc-promotion.ja.md)

# WI-784 — WI-783 终态文档投影

## 意图与边界

WI-784 是 WI-783 关闭后的有界文档 Work Item。它把不可变的 WI-783
Contract、verification、finalization 和 close 记录投影到英文、简体中文和
日文 Work Item 页面及 reference-parity 表中。

它不修改 Runtime 源码、产品行为、发布状态、治理规则，也不修改
WI-781、WI-782 或 WI-783 的不可变记录。

## 范围

- 以三种语言完成 WI-783 的终态文档投影。
- 在关闭前登记本 Work Item 自身的三份规划页和 parity 行，使关闭后的
  投影保持有界。
- 保持 promotion helper 和文档验收检查可复现。

## 验证

声明的文档检查为：

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-783-parity-finalization-recovery --check`

`bash tests/docs/documentation_acceptance.sh --repo <repo>`

终态投影由 WI-784 不可变的 archive、verification、finalization 和 close
记录派生；在 Runtime 关闭本 Work Item 前，这些终态字段应保持缺失。
