---
author: AI Cockpit maintainers
title: "WI-243 — pre-merge finalization reason policy"
workItemId: WI-243-finalization-reason-policy
description: "Require a meaningful, Runtime-bound reason for pre-merge finalization without inventing an undocumented token."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-243-finalization-reason-policy
terminalArchive: .ai/work-items/archive/WI-243-finalization-reason-policy.contract.json
terminalVerification: .ai/evidence/WI-243-finalization-reason-policy.verification.json
terminalFinalization: .ai/decisions/WI-243-finalization-reason-policy.finalize.json
terminalDecision: .ai/decisions/WI-243-finalization-reason-policy.close.json
authority: canonical
---

# WI-243 — pre-merge finalization reason policy

WI-243 makes the pre-merge finalization reason an explicit, non-empty audit
field. The governance gate consumes the Runtime-validated text and does not
require an undocumented magic token. Repository, Contract, evidence, Runtime,
PR, base, head, resource-context, and blocked/unmerged bindings remain
fail-closed.

The archived Contract, verification evidence, finalization chain, close
decision, and three-language parity rows are the authoritative records.
