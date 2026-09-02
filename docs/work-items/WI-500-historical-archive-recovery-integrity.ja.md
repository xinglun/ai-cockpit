---
author: AI Cockpit maintainers
title: "WI-500 — historical archive integrity recovery"
description: "任意レポート bytes が manifest digest と一致しない immutable historical archive のための監査可能な限定 recovery path を追加します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-500-historical-archive-recovery-integrity
status: implemented
authority: human-authorized
lastVerifiedBy: WI-500-historical-archive-recovery-integrity
terminalArchive: .ai/work-items/archive/WI-500-historical-archive-recovery-integrity.contract.json
terminalVerification: .ai/evidence/WI-500-historical-archive-recovery-integrity.verification.json
terminalFinalization: .ai/decisions/WI-500-historical-archive-recovery-integrity.finalize.json
terminalDecision: .ai/decisions/WI-500-historical-archive-recovery-integrity.close.json
canonical: docs/work-items/WI-500-historical-archive-recovery-integrity.ja.md
---

# WI-500 — historical archive integrity recovery

[English](WI-500-historical-archive-recovery-integrity.md) · [简体中文](WI-500-historical-archive-recovery-integrity.zh-CN.md)

## Boundary

この Work Item は、任意の `taskReportMarkdown` bytes が記録済み manifest
digest と異なる immutable historical archive に対して、限定的で fail-closed
な recovery path を追加します。必須の identity、Contract、Summary、Outcome、
その他 artifact の binding は厳格に維持し、predecessor bytes は書き換えません。

## Delivery state

実装は archive と verification を完了しています。review 済み PR は merge 済みで、
finalization receipt が記録され、confirmed close receipt は正確な resource
finalization head に bind されています。predecessor の historical bytes は不変の
まま保持し、recovery と close receipt は独立した監査記録として保存します。
