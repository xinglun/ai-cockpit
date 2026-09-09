---
author: AI Cockpit maintainers
title: "WI-765 — WI-764 关闭后文档 promotion"
description: "在不改变历史治理证据的前提下，提升已关闭 WI-764 的发布边界文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-765-wi764-doc-promotion
lastVerifiedBy: WI-765-wi764-doc-promotion
---

[English](WI-765-wi764-doc-promotion.md) · [日本語](WI-765-wi764-doc-promotion.ja.md)

# WI-765 — WI-764 关闭后文档 promotion

## 意图

将已验证关闭的 WI-764 发布边界投影提升到终态文档，同时保留失败发布
标签及全部历史治理证据。

## 边界

本 Work Item 只修改 WI-764 文档投影、reference-parity 台账及自身治理文档。
不修改 Runtime 行为、发布工作流、版本元数据，也不改写 WI-764 的历史 archive、
evidence、finalization、cleanup 或 close 字节。

## 验证

Runtime 验证证据及
`python3 tests/docs/promote_closed_work_item.py --check-all` 必须通过。Hosted
验证前先登记 pre-archive parity 行；只有本 Work Item 验证并关闭后才投影终态链接。
