---
author: AI Cockpit maintainers
title: "WI-281 — recovery integrity gate"
workItemId: WI-281-recovery-integrity-gate
description: "Make CI resolve append-only recovery heads and require complete current-cycle Work Item projections."
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

# WI-281 — recovery integrity gate

This Work Item closes the hosted-governance gap found when an immutable
predecessor has a canonical retry plus a digest-suffixed successor or
supersession receipt. The gate must select the valid recovery head
deterministically, keep invalid candidates fail-closed, and require the
tri-language Work Item and parity projections that the current release cycle
declares.
