---
author: AI Cockpit maintainers
title: "WI-834——WI-833 文档 promotion"
description: "为已关闭的 WI-833 lifecycle 登记绑定证据的阅读文档。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-834-wi833-doc-promotion
lastVerifiedBy: WI-834-wi833-doc-promotion
---

[English](WI-834-wi833-doc-promotion.md) · [日本語](WI-834-wi833-doc-promotion.ja.md)

# WI-834——WI-833 文档 promotion

## 意图与边界

WI-834 为已关闭的 WI-833 发布脚本来源 Work Item 登记阅读文档和
reference-parity 行。投影必须指向 Runtime 所拥有的准确 archive、verification、
finalization 和 close 记录，不能修改这些历史字节。

Runtime 行为、发布制品、历史 Work Item 和无关源码验证不在本 Work Item 范围内。

## 验收

- English、简体中文和日本語三种 WI-834 页面都存在，并绑定同一 Work Item 身份。
- 每个 parity ledger 在验证前都恰好包含一条 WI-834 pre-archive 行。
- WI-833 promotion 检查及仓库级文档检查通过。
- 关闭后，promotion 在同一行补充终态证据，但不改变 WI-833 历史记录。

## 验证

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`

