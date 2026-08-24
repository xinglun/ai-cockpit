---
author: AI Cockpit maintainers
title: "WI-251 — Outcome handoff base-binding recovery"
workItemId: WI-251-outcome-handoff-base-binding-recovery
description: "Redeliver the direct lifecycle Outcome and make resource finalization reject an archived Contract/PR base mismatch."
audience:
  - adopter
  - maintainer
status: current
lastVerifiedBy: WI-251-outcome-handoff-base-binding-recovery
authority: canonical
---

# WI-251 — Outcome handoff base-binding recovery

WI-250 produced an immutable verified archive and canonical finalization
receipt, but hosted governance found that its archived Contract base and the
provider PR base differed after a rebase. The installed Runtime had reported
that sequence-0 receipt as verified. WI-251 preserves those predecessor bytes,
binds the recovery decision, and redelivers the Outcome handoff from the
correct current base.

## Behavior

- The direct lifecycle handoff remains backward compatible: `finish`,
  `archive`, and `close` preserve stdout JSON, render the validated human
  Outcome on stderr by default, and suppress that handoff with `--json`.
- A blocked `finish` renders the persisted red or yellow Outcome and keeps its
  original nonzero result.
- Resource-finalization recording rejects a receipt whose
  `pullRequest.baseRevision` differs from the archived Contract
  `baseRevision`, before writing a canonical or transition decision.
- `finalize-verify` repeats the same cross-binding check, including canonical
  sequence 0, so a stored mismatch can never be reported as verified.

## Immutable boundary

Archive freezes the Contract base. A rebase must happen while the Work Item is
active and be followed by a renewed Contract binding and review. A base change
after archive fails closed and uses recovery; neither the archive nor the
finalization receipt may be rewritten. WI-250's archive, evidence, Outcome,
events, and finalization bytes remain historical truth, while its recovery
decision points to WI-251.

## Verification

Repository regressions cover record rejection without a decision file,
sequence-0 verify rejection after controlled fixture tampering, matching-base
success, and the existing transition controls. CLI tests cover all three
languages, stdout compatibility, `--json`, structured decisions, and blocked
handoff behavior. Documentation, parity, governance, formatting, Clippy, and
the locked workspace suite remain required.
