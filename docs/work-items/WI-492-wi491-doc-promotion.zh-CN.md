---
author: AI Cockpit maintainers
title: "WI-492——WI-491 终态文档晋级"
description: "在发布 v0.2.58 前，将已关闭 WI-491 的 release 证据晋级到面向读者的文档。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-492-wi491-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-492-wi491-doc-promotion
---

# WI-492——WI-491 终态文档晋级

本限定文档 Work Item 将已关闭 WI-491 的 release 证据晋级到三语页面和 parity
台账。它保留不可变治理记录，不改变 Runtime 行为。

[English](WI-492-wi491-doc-promotion.md) · [日本語](WI-492-wi491-doc-promotion.ja.md)

## 范围

- 晋级三个 WI-491 Work Item 页面及对应的三个 parity 行。
- 每个投影绑定 WI-491 的 archive、verification、finalization 和 close 证据。
- 将本 Work Item 自身页面和 parity 行纳入同一有界生命周期。

## 验收

- 六个 WI-491 投影在不重写不可变记录的前提下由证据支持。
- 已关闭 Work Item 晋级检查和状态一致性检查通过。
- 不修改 Runtime 源码、参考清单分类或全局 Agent/MCP 配置。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `git diff --check`
