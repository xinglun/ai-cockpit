---
author: AI Cockpit maintainers
title: "WI-281——recovery integrity gate"
workItemId: WI-281-recovery-integrity-gate
description: "让 CI 解析 append-only recovery head，并要求当前周期完整的 Work Item 投影。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-281-recovery-integrity-gate
terminalArchive: .ai/work-items/archive/WI-281-recovery-integrity-gate.contract.json
terminalVerification: .ai/evidence/WI-281-recovery-integrity-gate.verification.json
terminalFinalization: .ai/decisions/WI-281-recovery-integrity-gate.finalize.75797b8a6607897f2f36b13ee0fa30e60a3cd6902b4adf0562390da662cb1ed1.json
terminalDecision: .ai/decisions/WI-281-recovery-integrity-gate.close.json
authority: canonical
---

# WI-281——recovery integrity gate

本 Work Item 修复 hosted governance 的缺口：不可变 predecessor 可能同时有
canonical retry 与 digest-suffixed successor 或 supersession receipt。gate 必须
确定性选择有效 recovery head，对无效候选保持 fail-closed，并要求当前 release
cycle 声明的三语 Work Item 与 parity 投影完整存在。
