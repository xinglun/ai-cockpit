---
author: AI Cockpit maintainers
workItemId: WI-903-precheckpoint-contract-scope
title: Pre-checkpoint Contract scope amendment
description: Allow an active Work Item with no checkpoint or verification to append scope through the Runtime when a required initial projection was omitted.
audience: [maintainer, reviewer, contributor]
status: implemented
authority: human:xinglun
lastVerifiedBy: WI-903-precheckpoint-contract-scope
terminalArchive: .ai/work-items/archive/WI-903-precheckpoint-contract-scope.contract.json
terminalVerification: .ai/evidence/WI-903-precheckpoint-contract-scope.verification.json
terminalDecision: .ai/decisions/WI-903-precheckpoint-contract-scope.close.json
---

# WI-903 — Pre-checkpoint Contract scope amendment

This Work Item repairs a lifecycle deadlock discovered while starting WI-902.
The Runtime requires a declared documentation projection before checkpoint, but
previously rejected the only supported scope amendment before that checkpoint
existed. The repair is intentionally narrow: it permits only additive scope
entries before the first checkpoint and verification result. It does not permit
changes to identity, authority, base revision, mode, existing acceptance
criteria, or any post-checkpoint evidence boundary.

## Acceptance and evidence

- A fixture can append a missing declared path before checkpoint and proceed
  through normal preflight/checkpoint handling.
- Existing post-checkpoint amendment revalidation stays unchanged.
- Attempts to mutate immutable Contract fields remain rejected.
