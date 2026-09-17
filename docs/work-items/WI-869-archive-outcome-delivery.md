---
author: AI Cockpit maintainers
workItemId: WI-869-archive-outcome-delivery
title: Complete archived Outcome conversation delivery
description: Return and deliver the complete human Outcome for every supported Work Item archive without overstating host capabilities.
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-869-archive-outcome-delivery
terminalArchive: .ai/work-items/archive/WI-869-archive-outcome-delivery.contract.json
terminalVerification: .ai/evidence/WI-869-archive-outcome-delivery.verification.json
terminalDecision: .ai/decisions/WI-869-archive-outcome-delivery.close.json
---

# WI-869 — Complete archived Outcome conversation delivery

This Work Item closes the gap between a successful archive, a complete human
Outcome prepared from validated facts, and its delivery through supported CLI,
MCP, and Agent adapter boundaries. Archive and verification records remain
immutable; delivery retry does not rerun lifecycle work.

Query summaries remain distinct from archive delivery. The archive path returns
a versioned full delivery payload, long messages are segmented without silent
truncation, and host acceptance or display is reported only when that host
actually provides confirmation. A controllable adapter test records assistant
message events; it does not prove that an external host displayed a message or
that a person read or approved it.
