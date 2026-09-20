---
author: AI Cockpit maintainers
title: "WI-954——文档投影恢复"
description: "恢复发布后文档投影路径，且不重复执行发布验证。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:xinglun
workItemId: WI-954-start-verification-declaration
lastVerifiedBy: WI-954-start-verification-declaration
---

[English](WI-954-start-verification-declaration.md) · [日本語](WI-954-start-verification-declaration.ja.md)

# WI-954——文档投影恢复

## 意图

在 WI-953 被替换后恢复受阻的发布后文档投影路径，不重复执行发布验证，也不改变 Runtime 行为。

## 边界

本 Work Item 只修改三语人类文档投影及其 parity 行。发布产物、已完成验证证据和对象仓库不在范围内。

## 验收

- 投影 helper 达到稳定状态，且不启动 Cargo 测试。
- 三语页面与 parity 行保持一致。
