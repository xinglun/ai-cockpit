---
author: AI Cockpit maintainers
title: "WI-340 — archived finalization recovery"
workItemId: WI-340-finalization-recovery
description: "Provide a bounded, append-only recovery path for an archived Work Item whose provider finalization is still pending."
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-340-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-340-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-340-finalization-recovery.verification.json
---

# WI-340 — archived finalization recovery

WI-340 makes recovery explicit for an archived Work Item that has a bound
provider context but no valid provider-side finalization receipt. A normal
archived item remains non-green until that receipt is recorded; a valid
append-only `supersede` recovery decision is the only bounded exception for a
historical predecessor.

The original Contract, Summary, Outcome, Events, archive, and verification
evidence remain immutable. Invalid, foreign, malformed, or missing recovery
records cannot bypass finalization or evidence checks. Existing finalized
Work Items retain their green path.

The tri-language entry points are:

- [简体中文](WI-340-finalization-recovery.zh-CN.md)
- [日本語](WI-340-finalization-recovery.ja.md)

## Acceptance boundary

1. A valid supersede recovery decision permits the explicit close flow without
   rewriting predecessor archive bytes.
2. A missing or invalid recovery decision cannot bypass finalization or
   verification gates.
3. A pending provider finalization is rendered as a visible yellow Outcome,
   never as verified or green.
4. A normally finalized Work Item keeps the existing green path.
5. Recovery, pending-finalization, invalid-decision, and finalized-path
   regressions pass in the locked workspace.
