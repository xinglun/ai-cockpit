---
author: AI Cockpit maintainers
title: "WI-567——WI-566 终态文档晋级"
description: "晋级已关闭的 WI-566 文档投影，不改写不可变治理记录。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-567-wi566-doc-promotion
lastVerifiedBy: WI-567-wi566-doc-promotion
---

[English](WI-567-wi566-doc-promotion.md) · [日本語](WI-567-wi566-doc-promotion.ja.md)

# WI-567——WI-566 终态文档晋级

## 目标

晋级已验证关闭的 WI-566 三语文档页面，并在三语参考矩阵中登记本次有界晋级。
不可变 Contract、evidence、decision 和 archive 记录保持不变。

## 范围与边界

范围包括 WI-566 三语页面、WI-567 三语页面和三份 reference-parity 页面。
Runtime 行为、发布产物、对象工程、全局 Agent/MCP 设置以及历史治理 bytes
均不在范围内。

## 验收

- 三语 WI-566 页面状态为 `implemented`，并链接 archive、verification、
  finalization、close 证据。
- 三份 parity 页面将 WI-566 标为已实现，并登记 WI-567 的有界 pre-archive 投影。
- 文档、parity、晋级和 diff 检查通过，且不重写不可变治理记录。
- WI-567 自身具备可读三语文档和一条匹配的 pre-archive parity 登记。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo . --work-item WI-566-documentation-promotion`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

