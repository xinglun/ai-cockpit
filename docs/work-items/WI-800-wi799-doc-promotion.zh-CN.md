---
author: AI Cockpit maintainers
title: "WI-800——WI-799 文档投影"
description: "使用当前 Runtime 证据晋级已关闭 WI-799 的三语 Work Item 和 reference-parity 投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-root-cause-repair-and-release
workItemId: WI-800-wi799-doc-promotion
lastVerifiedBy: WI-800-wi799-doc-promotion
---

[English](WI-800-wi799-doc-promotion.md) · [日本語](WI-800-wi799-doc-promotion.ja.md)

# WI-800——WI-799 文档投影

## 意图与边界

WI-800 是有界的文档 Work Item。它根据不可变的 archive、verification、
finalization 和 close 证据晋级 WI-799 的终态投影，不重写 WI-799 历史，也不改变
产品、Runtime、发布或 provider 行为。

## 验收

- WI-799 的英文、简体中文和日文页面绑定同一组终态证据。
- 三个 reference-parity 行绑定同一 predecessor、evidence、finalization 和 close 事实。
- WI-800 自身的三语页面和 parity 行在 archive 前完成登记。
- 文档、parity、治理完整性和状态一致性检查通过，且不重写不可变记录。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-799-wi799-doc-promotion --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
