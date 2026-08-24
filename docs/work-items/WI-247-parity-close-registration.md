---
author: AI Cockpit maintainers
title: "WI-247 — WI-246 close parity registration"
workItemId: WI-247-parity-close-registration
description: "Preserve the immutable WI-247 archive and recover its post-archive parity-ordering defect."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-249-parity-finalization-registration
authority: canonical
---

# WI-247 — WI-246 close parity registration

WI-247 verified and archived the documentation change intended to project the
authoritative WI-246 close chain. After archive, its own parity row was changed
from an active Contract projection to archive/evidence/finalization paths. That
documentation mutation was not part of the archived verification snapshot, so
PR #198 remains an unmerged immutable predecessor rather than a green delivery.

## Recovery boundary

Runtime receipt
`.ai/decisions/WI-247-parity-close-registration.recovery.json` binds the exact
Contract, Summary, Outcome, Events, archive manifest, verification evidence,
repository identity, and Runtime v0.2.31 digests. WI-249 imports those bytes
unchanged from recovery bootstrap `f59ff36`; it does not replay the WI-247
implementation or manufacture a finalization receipt.

## Root correction

WI-249 retains WI-247 as `Recovered`, completes the WI-246 terminal ledger
projection, and adds a conditional pre-archive control. Only a Work Item whose
Contract, Summary, or acceptance owns the parity ledger must publish all three
lifecycle-bound rows before verification. Ordinary code Work Items remain
outside this documentation obligation.
