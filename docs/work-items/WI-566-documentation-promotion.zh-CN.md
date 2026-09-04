---
author: AI Cockpit maintainers
title: "WI-566——WI-565 文档投影晋级"
description: "晋级已验证关闭的 WI-565 三语文档，并登记本次有界晋级任务。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-566-documentation-promotion
lastVerifiedBy: WI-566-documentation-promotion
---

[English](WI-566-documentation-promotion.md) · [日本語](WI-566-documentation-promotion.ja.md)

# WI-566——WI-565 文档投影晋级

## 目标

将已验证并关闭的 WI-565 发布任务的三语文档投影晋级为终态，并在同一组
投影中登记本次文档 Work Item。只引用不可变 Runtime 证据，不重写证据。

## 范围与边界

范围包括 WI-565 三语页面、WI-566 三语页面、三份 reference-parity 页面和
已关闭 Work Item 晋级 helper。Runtime 行为、发布产物、对象工程、全局
Agent/MCP 设置，以及不可变 Contract、evidence、decision、archive bytes
均不在范围内。

## 验收

- 三语 WI-565 页面状态为 `implemented`，并链接 archive、verification、
  finalization、close 证据。
- 三份 parity 页面将 WI-565 标为已实现，并登记 WI-566 的有界 pre-archive 投影。
- 文档、parity、晋级和 diff 检查通过，且不重写历史治理记录。
- WI-566 自身具备可读三语文档和一条匹配的 pre-archive parity 登记。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

