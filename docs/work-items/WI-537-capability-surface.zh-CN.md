---
title: "类型化 MCP 能力面"
workItemId: WI-537-capability-surface
author: AI Cockpit maintainers
description: "类型化、fail-closed 的 MCP 能力发现与使用说明。"
audience:
  - adopter
  - maintainer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-537-capability-surface
terminalArchive: .ai/work-items/archive/WI-537-capability-surface.contract.json
terminalVerification: .ai/evidence/WI-537-capability-surface.verification.json
terminalFinalization: .ai/decisions/WI-537-capability-surface.finalize.json
terminalDecision: .ai/decisions/WI-537-capability-surface.close.json
---

# WI-537——类型化 MCP 能力面

AI Cockpit 将 MCP 工具暴露为可发现、绑定 repository 的接口，供人和
Agent 使用。`tools/list` 描述每个工具的参数；`tools/call` 会在 dispatch
之前拒绝缺失、格式错误、相互冲突或未知参数。CLI 与三语参考文档说明同一套
发现流程和面向人的 Outcome 交接方式。

范围仅限 MCP 能力描述/校验和文档，不新增 lifecycle mutation，不配置全局
Agent/MCP，也不自动向宿主对话发送消息。

Work Item 关闭后，verification 与终态 lifecycle 记录会链接到[参考一致性
登记表](../reference/reference-parity.zh-CN.md)。
