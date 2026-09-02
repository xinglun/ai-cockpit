---
author: AI Cockpit 维护者
title: "WI-505——WI-504 终态文档投影"
description: "修正 close 后门发现的条件性状态，将 WI-504 文档与 parity 投影晋升为终态。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-505-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-505-doc-promotion
---

# WI-505——WI-504 终态文档投影

[English](WI-505-doc-promotion.md) · [日本語](WI-505-doc-promotion.ja.md)

## 边界

本限定文档 Work Item 消费 WI-504 close 后强制晋升检查发现的问题。只更新
三语 Work Item 页面和三语 parity 投影，使已关闭 Work Item 在面向读者的基线中
呈现终态。不重写 Runtime 生成的证据，也不改变 Runtime 行为。

## 范围

- 将 WI-504 英文、简体中文和日文页面晋升为有证据支持的终态。
- 将三语 reference parity 行从条件性状态晋升为 `Implemented`。
- 更新投影后重新运行文档和状态一致性门禁。

## 不在范围内

Runtime 源码、测试、对象/adopter 工程、参考源实现、版本发布、全局
Agent/MCP 配置，以及历史证据或归档重写。

## 验收

- WI-504 三语页面包含终态证据路径并使用 `status: implemented`。
- WI-504 三语 parity 行为 `Implemented`，并链接到精确终态记录。
- `promote_closed_work_item.py --repo <repo> --check-all` 通过。
- 文档、parity 和 Work Item 状态一致性检查通过。
- 不编辑或删除生成证据和历史字节。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

终态投影以该 helper 为来源；生成收据保持不可变。
