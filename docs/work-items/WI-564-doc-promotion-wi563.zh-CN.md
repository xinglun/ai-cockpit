---
author: AI Cockpit maintainers
title: "WI-564——WI-563 终态文档晋级"
description: "晋级 WI-563，并在三语治理投影中登记本次文档晋级任务。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-564-doc-promotion-wi563
lastVerifiedBy: WI-564-doc-promotion-wi563
---

[English](WI-564-doc-promotion-wi563.md) · [日本語](WI-564-doc-promotion-wi563.ja.md)

# WI-564——WI-563 终态文档晋级

## 目标

将已验证关闭的 WI-563 文档投影晋级为终态，并登记本晋级任务自身，
使文档治理门能够审计当前批次的每个 Work Item。

## 范围与边界

范围仅包含 WI-563 三语页面、WI-564 三语页面和三份对应的
reference-parity 页面。WI-563 的终态链接由 Runtime 晋级 helper 提供；
WI-564 页面记录有界的自身投影，在本任务完成验证并关闭前保持进行中。

Runtime 行为、对象工程、本地参考 checkout、发布产物、全局 Agent/MCP
配置，以及不可变 Contract/evidence/decision/archive bytes 均不在范围内。

## 验收

- WI-563 页面晋级为已实现，并包含 archive、verification、finalization、
  close 链接。
- WI-564 页面说明范围，并从三份 parity 页面互相链接。
- 三份 parity 页面为两个 Work Item 使用一致的状态和证据路径，
  文档、台账和治理门全部通过。
- 不改写前驱 Contract、evidence、decision 或 archive bytes。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `git diff --check`
