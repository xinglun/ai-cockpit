---
author: AI Cockpit maintainers
title: "WI-855 — selected successor lineage recovery"
description: "Recover an already selected multi-hop successor lineage without rewriting historical evidence or creating a competing successor."
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-855-selected-successor-lineage-recovery
lastVerifiedBy: WI-855-selected-successor-lineage-recovery
terminalArchive: .ai/work-items/archive/WI-855-selected-successor-lineage-recovery.contract.json
terminalVerification: .ai/evidence/WI-855-selected-successor-lineage-recovery.verification.json
terminalDecision: .ai/decisions/WI-855-selected-successor-lineage-recovery.close.json
---

[简体中文](WI-855-selected-successor-lineage-recovery.zh-CN.md) · [日本語](WI-855-selected-successor-lineage-recovery.ja.md)

# WI-855 — selected successor lineage recovery

WI-855 adds an append-only recovery receipt for an already selected multi-hop
successor lineage. Every adjacent edge and archived node is bound to exact
repository bytes, Runtime identity, provider PR/finalization facts, and an
explicit human decision. Invalid, stale, foreign, forked, or ambiguous history
remains fail-closed; the route never creates a competing successor or rewrites
historical records.
