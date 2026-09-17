---
author: AI Cockpit maintainers
workItemId: WI-869-archive-outcome-delivery
title: 归档后的完整 Outcome 对话交付
description: 为每个受支持的 Work Item 归档返回并交付完整人类 Outcome，同时不夸大宿主能力。
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-869-archive-outcome-delivery
---

# WI-869 — 归档后的完整 Outcome 对话交付

本 Work Item 收敛归档成功、从已验证事实准备完整人类 Outcome，以及通过
CLI、MCP 和 Agent 适配边界交付之间的缺口。归档和验证记录保持不可变；
交付补投不会重新执行生命周期工作。

查询摘要与归档交付保持区分。归档路径返回版本化完整交付数据，长正文分段
时不得静默截断，并且只有宿主实际提供确认时才报告接受或展示。可控适配器测试
会记录 assistant 消息事件；这不证明第三方宿主展示，也不证明人已阅读或批准。
