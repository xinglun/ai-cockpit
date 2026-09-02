---
author: AI Cockpit maintainers
title: "WI-513——WI-512 终态文档晋级"
description: "在不改写不可变治理记录的前提下晋级 WI-512 投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-513-wi512-doc-promotion
lastVerifiedBy: WI-513-wi512-doc-promotion
---

[English](WI-513-wi512-doc-promotion.md) · [日本語](WI-513-wi512-doc-promotion.ja.md)

## 目标

在 WI-512 的关闭证据存在后，将 parity 投影从预归档登记晋级为终态
`已实现`。辅助程序必须是确定性的，不得改写 WI-512 的 Contract、Summary、
Outcome、Events、verification、finalization 或 close 字节。

## 范围

- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- 本 WI 的三语读者记录。

## 验收

- `promote_closed_work_item.py --check-all` 不再报告 WI-512 的陈旧投影。
- 文档、parity 与治理完整性检查通过。
- WI-512 不可变生成记录保持字节级不变。
- 不修改 Runtime、参考源、对象工程或全局 Agent/MCP 配置。

## 边界

这只是关闭后的文档投影，不改变治理事实、不新增审批，也不复制参考实现。
