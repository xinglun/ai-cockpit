---
author: AI Cockpit maintainers
title: "WI-340——归档 finalization 恢复"
workItemId: WI-340-finalization-recovery
description: "为 provider finalization 尚未完成的 archived Work Item 提供有界、追加式恢复路径。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-340-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-340-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-340-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-340-finalization-recovery.finalize.db551e5edf1e88fde01c18898a6a81b58692f425d427d71aeee3442b4e90d613.json
terminalDecision: .ai/decisions/WI-340-finalization-recovery.close.json
---

# WI-340——归档 finalization 恢复

WI-340 为已归档但仍绑定 provider context、尚无有效 provider-side
finalization receipt 的 Work Item 明确定义恢复边界。普通 archived Work Item
在 receipt 记录前保持非绿色；只有有效的追加式 `supersede` recovery decision
可以为历史前项提供有界例外。

原始 Contract、Summary、Outcome、Events、archive 和 verification evidence
保持不可变。缺失、无效、foreign 或格式错误的 recovery 记录不能绕过
finalization 或 evidence 校验；已正常 finalization 的 Work Item 保持绿色路径。

文档入口：[English](WI-340-finalization-recovery.md) · [日本語](WI-340-finalization-recovery.ja.md)

## 验收边界

1. 有效 supersede recovery decision 可以在不改写前项 archive bytes 的情况下进入明确的关闭流程。
2. 缺失或无效 recovery decision 不能绕过 finalization 或 verification gate。
3. provider finalization pending 必须显示为人类可见的黄色 Outcome，不能显示为 verified 或绿色。
4. 正常 finalization 的 Work Item 保持既有绿色路径。
5. recovery、pending-finalization、无效 decision 和已 finalization 路径的 locked workspace 回归测试通过。
