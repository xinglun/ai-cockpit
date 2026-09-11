---
author: AI Cockpit maintainers
title: "WI-795——WI-794 已关闭文档晋级"
description: "将 WI-794 的 Runtime 关闭证据投影为有边界的三语文档。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-795-wi794-doc-promotion
lastVerifiedBy: WI-795-wi794-doc-promotion
---

[English](WI-795-wi794-doc-promotion.md) · [日本語](WI-795-wi794-doc-promotion.ja.md)

# WI-795——WI-794 已关闭文档晋级

## 意图与边界

WI-795 是 WI-794 关闭后的有边界文档 Work Item。它依据不可变的 WI-794
Contract、verification、finalization 和 close 记录，更新英文、简体中文和
日文 Work Item 页面及 reference-parity 表。

它不修改 Runtime 源码、产品行为、发布状态、治理规则或 WI-794 的不可变记录。

## 范围

- 晋级三种语言中的 WI-794 终态文档投影。
- 在 close 前登记本 Work Item 自身的三语规划页和 parity 行，使 close 后投影
  仍保持有边界。
- 保持 promotion helper 和文档验收检查可复现。

## 验收

- WI-794 的三语页面绑定实际的 terminal archive、verification、finalization 和
  close 路径。
- WI-795 自身的三语页面和 parity 行在 archive 前登记，并且只在 verified close
  后晋级。
- 文档、parity、governance-integrity 和 status-consistency 检查通过，且不改写
  不可变记录。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-794-release-v0-2-90-closure --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
