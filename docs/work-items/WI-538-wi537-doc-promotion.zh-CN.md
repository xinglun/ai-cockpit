---
author: AI Cockpit maintainers
title: "WI-538——WI-537 终态文档晋级"
description: "晋级已完成的 WI-537 能力文档，并登记这一受边界约束的投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-538-wi537-doc-promotion
lastVerifiedBy: WI-538-wi537-doc-promotion
---

[English](WI-538-wi537-doc-promotion.md) · [日本語](WI-538-wi537-doc-promotion.ja.md)

## 目标

将三语 WI-537 读者页面与不可变的已关闭证据及 parity 行同步，同时登记本
文档 Work Item。

## 范围与边界

- WI-537 的三语读者页面与三份 parity ledger。
- 本 Work Item 的三语读者页面与 parity 登记。
- Runtime 行为、生成的 `.ai` 记录、发布产物和对象工程不在范围内。

## 验收

- WI-537 投影带有终态证据并使用 `implemented` 状态。
- WI-538 页面和 parity 行保持互相链接且语义等价。
- 文档验收、状态一致性、parity 完整性和 close 后晋级检查全部通过。

## 证据边界

晋级只修改面向读者的投影。不可变 Contract、verification、finalization 和
close 记录仍由 Runtime 管理。
