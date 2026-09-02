---
author: AI Cockpit maintainers
title: "WI-500 — historical archive recovery integrity"
description: "Provide a bounded, auditable recovery path for immutable historical archive artifacts whose optional report bytes no longer match their manifest."
audience: [maintainer, reviewer, adopter]
workItemId: WI-500-historical-archive-recovery-integrity
status: implemented
authority: human-authorized
lastVerifiedBy: WI-500-historical-archive-recovery-integrity
terminalArchive: .ai/work-items/archive/WI-500-historical-archive-recovery-integrity.contract.json
terminalVerification: .ai/evidence/WI-500-historical-archive-recovery-integrity.verification.json
terminalFinalization: .ai/decisions/WI-500-historical-archive-recovery-integrity.finalize.json
terminalDecision: .ai/decisions/WI-500-historical-archive-recovery-integrity.close.json
canonical: docs/work-items/WI-500-historical-archive-recovery-integrity.md
---

# WI-500 — historical archive recovery integrity

[简体中文](WI-500-historical-archive-recovery-integrity.zh-CN.md) · [日本語](WI-500-historical-archive-recovery-integrity.ja.md)

## Boundary

This Work Item adds a narrow, fail-closed recovery path for an immutable
historical archive whose optional `taskReportMarkdown` bytes differ from the
recorded manifest digest. Required identity, contract, summary, outcome, and
other artifact bindings remain strict. Predecessor bytes are never rewritten.

## Delivery state

The implementation is archived and verified. The reviewed pull request was
merged, the finalization receipt was recorded, and the confirmed close receipt
is bound to the exact resource-finalization head. Historical predecessor bytes
remain immutable; recovery and close receipts are retained as separate audit
records.
