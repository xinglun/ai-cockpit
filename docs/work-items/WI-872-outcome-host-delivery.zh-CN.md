---
author: AI Cockpit maintainers
workItemId: WI-872-outcome-host-delivery
title: "归档 Outcome 的宿主交付"
description: "为归档 Work Item 提供完整 Outcome 交接并明确宿主交付边界。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-872-outcome-host-delivery
terminalArchive: .ai/work-items/archive/WI-872-outcome-host-delivery.contract.json
terminalVerification: .ai/evidence/WI-872-outcome-host-delivery.verification.json
terminalDecision: .ai/decisions/WI-872-outcome-host-delivery.close.json
---

# WI-872 — 归档 Outcome 的宿主交付

本 Work Item 收敛 Runtime 准备的完整 Outcome 与宿主 assistant 消息事件
之间的最后边界。没有宿主发送接口时，CLI/MCP 仍如实提供完整交接内容。

## 验收边界

- 普通和 JSON 归档输出使用同一份已验证完整正文。
- 宿主适配器返回实际逐消息 receipt；能力声明不能推导接受或展示。
- 中断只从身份绑定的已接受进度恢复。
- return-only 适配器明确报告 `full_handoff_only` 和未知宿主状态。
- 受控适配器事件证明顺序、分段、连续 WI、中断补投和零发送拒绝。

发布不属于本 Work Item，只有全部 Work Item 和证据完成后才执行。
