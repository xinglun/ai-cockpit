---
author: AI Cockpit maintainers
title: "WI-490——WI-489 终态文档投影"
description: "晋级有界的 WI-489 文档投影，并终止 post-close 文档门递归。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-490-wi489-terminal-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-490-wi489-terminal-doc-promotion
---

# WI-490——WI-489 终态文档投影

本限定文档 Work Item 使用 WI-489 的不可变终态证据，晋级其三语文档页面和
parity 登记。本 Work Item 只闭合文档投影链路，不修改 Runtime 行为、历史证据
或全局 Agent/MCP 配置。

[English](WI-490-wi489-terminal-doc-promotion.md) · [日本語](WI-490-wi489-terminal-doc-promotion.ja.md)

## 范围

- 将三个 WI-489 Work Item 页面晋级为带终态证据的元数据。
- 将三个 WI-489 parity 行晋级，并绑定 archive、verification、finalization 和 close 引用。
- 将本 Work Item 自身页面和 parity 登记纳入同一有界投影，使终态检查不会递归产生 successor。

## 验收

- 六个 WI-489 文档投影页面/行在不修改已撰写内容或不可变治理记录的前提下完成晋级。
- post-close 晋级检查和状态一致性检查识别此精确文档范围为自终态。
- 中、英、日文档检查全部通过，且不修改全局 Agent/MCP 配置。

## 验证

- `bash tests/docs/promote_closed_work_item_test.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `python3 tests/conformance/reference_file_inventory.py --check`
