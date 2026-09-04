---
author: AI Cockpit maintainers
title: "WI-571——WI-570 终态文档晋级"
description: "在不改写不可变治理记录的前提下，晋级已关闭 WI-570 的文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-571-wi570-doc-promotion
lastVerifiedBy: WI-571-wi570-doc-promotion
---

[English](WI-571-wi570-doc-promotion.md) · [日本語](WI-571-wi570-doc-promotion.ja.md)

# WI-571——WI-570 终态文档晋级

## 目标

晋级 WI-570 的验证关闭文档页面，并在三语 parity 矩阵中登记本次文档投影。
不可变治理记录保持不变。

## 范围与边界

范围是三份 WI-570 页面、三份 WI-571 页面和三份 reference-parity 页面。
Runtime 行为、发布制品、对象工程、全局 Agent/MCP 设置和历史治理字节不在范围内。

## 验收

- 三语 WI-570 页面状态为 `implemented`，并链接 archive、verification、finalization 和
  close 证据。
- 三语 parity 页面将 WI-570 标记为已实现，并登记带证据路径的 WI-571 终态投影。
- 文档、parity、promotion 和 diff 检查通过，且不改写不可变治理记录。
- WI-571 具有可读且相互对应的英语、简体中文和日语页面。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
