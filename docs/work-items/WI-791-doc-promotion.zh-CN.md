---
author: AI Cockpit maintainers
title: "WI-791——WI-785 与 WI-787 文档投影"
description: "在 Runtime 恢复与关闭验证后修复有界的三语文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-791-doc-promotion
lastVerifiedBy: WI-791-doc-promotion
---

[English](WI-791-doc-promotion.md) · [日本語](WI-791-doc-promotion.ja.md)

# WI-791——WI-785 与 WI-787 文档投影

## 意图与边界

WI-791 是有界的文档 Work Item。它使用不可变的 Runtime 记录晋级 WI-787
终态投影，并修复 WI-785 的恢复态 parity 投影。不重写 archive、evidence、
finalization、close 或 recovery 字节，也不改变产品行为、Runtime 行为或发布状态。

## 验收

- 三语 WI-787 页面绑定实际的终态 archive、verification、finalization 和 close 路径。
- 三语 WI-785 页面及 parity 行将不可变尝试标记为已恢复，并绑定实际哈希化的 supersede 决定和 verification。
- WI-791 自身三语页面和 parity 行在 archive 前完成注册。
- 文档、parity、治理完整性和状态一致性检查通过，且不重写不可变记录。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-787-parity-finalization-recovery`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test --locked --workspace`
