---
author: AI Cockpit maintainers
workItemId: WI-929-outcome-hci-public-handoff
title: 归档 Work Item 的完整 Outcome 对话交接
description: 让 CLI 和 MCP 消费者始终取得完整、按对话语言选择的人类 Outcome 与有序 assistant 消息事件，同时不把未确认的宿主展示说成已完成。
audience: [maintainer, reviewer, contributor]
status: implemented
authority: human:xinglun
lastVerifiedBy: WI-929-outcome-hci-public-handoff
terminalArchive: .ai/work-items/archive/WI-929-outcome-hci-public-handoff.contract.json
terminalVerification: .ai/evidence/WI-929-outcome-hci-public-handoff.verification.json
terminalFinalization: .ai/decisions/WI-929-outcome-hci-public-handoff.finalize.json
terminalDecision: .ai/decisions/WI-929-outcome-hci-public-handoff.close.json
---

# WI-929 — 归档 Work Item 的完整 Outcome 对话交接

本 Work Item 收敛归档 Outcome 的最后一段 HCI 交接缺口。归档返回必须携带
同一份已经验证的人类完整 Outcome，包括语言、Work Item 身份、交付身份和有序
assistant 消息事件。只读取 stdout、JSON 或 MCP content 的消费者，也不能再因为
正文原先只在 stderr 或 summary 视图中出现而丢失交付内容。

当前对话语言选择展示语言（`en`、`zh` 或 `ja`）。Contract 原文、证据事实和
receipt 身份保持不变，只本地化 Runtime 的展示。没有宿主适配器时仍为
`full_handoff_only`，展示确认是未知；只有逐条真实 receipt 才能报告
`display_confirmed`，不能从 capability 或 Agent 自填布尔值推导。

中断交付复用同一份正文和身份绑定的已接受分段进度，不重新执行验证或归档。
宿主不支持幂等或展示确认时，重复消息风险和展示状态必须明确保留未知。受控的
command-host 测试只能证明适配器协议和事件正文准确，不能证明第三方 Codex 或
Claude 对话窗口实际展示了消息。

## 验收与证据

- CLI 普通归档和 `archive --json` 从同一份已验证 `OutcomeDelivery` 暴露完整
  human handoff 正文及有序 `assistantMessageEvents`。
- MCP `delivery=true` 暴露相同正文、身份、语言、有序分段和事件；parity 测试
  比较正文与事件序列。
- 英文、简体中文、日文保留相同事实和下一步，只按当前对话语言选择展示。
- 无宿主、中断、非连续进度和连续两个 Work Item 的场景继续 fail-closed，且不
  新增验证或归档执行。
- 没有证据声明时，最终 Outcome 的用户收益保持未知。

本 Work Item 不建设通用聊天平台，不改变生命周期或授权规则，不修改对象仓库，
也不发布版本。
