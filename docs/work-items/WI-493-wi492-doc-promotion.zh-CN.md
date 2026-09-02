---
author: AI Cockpit maintainers
title: "WI-493——WI-492 终态文档晋级"
description: "晋级已关闭 WI-492 的文档门证据，并终止 release 文档循环。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-493-wi492-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-493-wi492-doc-promotion
---

# WI-493——WI-492 终态文档晋级

本限定文档 Work Item 晋级已关闭 WI-492 的终态证据和 parity 登记。将自身页面
纳入同一范围，使 post-close 文档检查达到自终态而不递归。

[English](WI-493-wi492-doc-promotion.md) · [日本語](WI-493-wi492-doc-promotion.ja.md)

## 范围

- 使用终态证据晋级三个 WI-492 页面和三个 parity 行。
- 在同一有界范围维护三个 WI-493 页面及 parity 行。
- 保留不可变治理记录，不改变 Runtime 行为。

## 验收

- WI-492 投影链接到 archive、verification、finalization 和 close receipt。
- post-close 晋级、治理完整性和状态一致性检查通过。
- 不修改源码、参考清单或全局 Agent/MCP 配置。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
