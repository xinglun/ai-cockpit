---
author: AI Cockpit maintainers
title: "WI-449 — historical finalization と adopter recovery"
workItemId: WI-449-historical-finalization-adopter-recovery
description: "過去の finalization 記録に対する誠実な recovery と projection の経路を提供します。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-449-historical-finalization-adopter-recovery
terminalArchive: .ai/work-items/archive/WI-449-historical-finalization-adopter-recovery.contract.json
terminalVerification: .ai/evidence/WI-449-historical-finalization-adopter-recovery.verification.json
terminalFinalization: .ai/decisions/WI-449-historical-finalization-adopter-recovery.finalize.json
terminalDecision: .ai/decisions/WI-449-historical-finalization-adopter-recovery.close.json
---

# WI-449 — historical finalization と adopter recovery

この Work Item は read-only の historical finalization inventory と recovery plan を
追加し、predecessor の bytes を保持し、検証サイクルが `finish_ready` に達した後の
遅い resource rebinding を拒否します。

[English](WI-449-historical-finalization-adopter-recovery.md) · [简体中文](WI-449-historical-finalization-adopter-recovery.zh-CN.md)
