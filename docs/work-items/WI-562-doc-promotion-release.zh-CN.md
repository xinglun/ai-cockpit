---
author: AI Cockpit maintainers
title: "WI-562——WI-561 终态文档晋级"
description: "晋级已关闭 WI-561 的发布文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-562-doc-promotion-release
lastVerifiedBy: WI-562-doc-promotion-release
---

[English](WI-562-doc-promotion-release.md) · [日本語](WI-562-doc-promotion-release.ja.md)

# WI-562——WI-561 终态文档晋级

## 目标

仅依据不可变终态记录，晋级已验证关闭的 WI-561 三语 Work Item 页面和
reference-parity 投影。

## 范围与边界

范围仅限三个 WI-561 语言页面、三个对应的 reference-parity 页面，以及本
Work Item 的三个语言页面。终态状态只能由已关闭 Work Item 晋级 helper 写入。
Runtime 行为、对象仓库、全局 Agent/MCP 配置、源清单语义和无关文档均不在范围内。

## 验收

- WI-561 投影引用终态 archive、verification、finalization 和 close 证据，且不改变治理事实。
- 三语 parity 页面在本 Work Item 自身验证关闭前登记其进行中状态。
- closed Work Item 检查、文档验收、parity gate 和声明的验证命令全部通过。
- 不修改不可变 receipt 或无关投影。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
