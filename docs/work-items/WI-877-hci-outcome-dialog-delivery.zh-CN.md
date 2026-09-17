---
author: AI Cockpit maintainers
workItemId: WI-877-hci-outcome-dialog-delivery
title: 面向对话的归档 Outcome 交付
description: 为每个归档 Outcome 暴露完整且身份绑定的 assistant-message 事件，同时如实保留宿主能力边界。
audience: [adopter, contributor, maintainer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-877-hci-outcome-dialog-delivery
---

# WI-877——面向对话的归档 Outcome 交付

本 Work Item 收敛归档后的 HCI 交接缺口。Runtime 继续以 `OutcomeDelivery`
作为唯一已验证事实源，并暴露有序的 `assistantMessageEvents`，使对话层可以
把每个完整分段逐字转发为独立 assistant 消息。

## 验收边界

- 普通归档和受支持的历史归档在合法 CLI JSON 与 MCP 交付输出中保留完整正文。
- 每个对话事件都从同一准备载荷保留 WI、delivery、归档、语言、顺序、分段摘要哈希和正文事实。
- 宿主接受/展示只来自逐消息 receipt。没有宿主 API 时仍为 `full_handoff_only`、展示为 `unknown`，不声称有内置 Codex 或 Claude 连接器。
- 中断交付复用同一身份绑定的已接受进度，不重跑验证或归档；连续 WI 保持独立。

本 Work Item 不改变授权、验证、版本发布、Issue #851 recovery 语义或对象仓库。
