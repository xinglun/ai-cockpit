---
author: AI Cockpit maintainers
title: "WI-427——parity governance recovery"
description: 在托管 CI 发现注册缺失后，重新交付 recovery binding 并修复三语 parity 台账。
workItemId: WI-427-parity-governance-recovery
audience: [contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-427-parity-governance-recovery
terminalArchive: .ai/work-items/archive/WI-427-parity-governance-recovery.contract.json
terminalVerification: .ai/evidence/WI-427-parity-governance-recovery.verification.json
terminalFinalization: .ai/decisions/WI-427-parity-governance-recovery.finalize.json
terminalDecision: .ai/decisions/WI-427-parity-governance-recovery.close.json
---

# WI-427——parity governance recovery

本 successor 保留不可变的 recovery 历史，同时重新交付 binding，并在每份
reference-parity 台账中登记选中的 decision 与 evidence 路径。它不会改写前置项
归档 bytes，也不会放宽文档门禁。

parity 行是归档前登记；只有在 verification、评审合并、finalization 和 close
receipt 都存在后，状态才会变为“已实现”。

[English](WI-427-parity-governance-recovery.md) · [日本語](WI-427-parity-governance-recovery.ja.md)
