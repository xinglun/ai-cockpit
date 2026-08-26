---
author: AI Cockpit maintainers
title: "WI-296 — Governance closure recovery"
workItemId: WI-296-governance-closure-recovery
description: "Re-deliver consumed retry-history handling with complete parity and terminal finalization checks."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-296-governance-closure-recovery
terminalArchive: .ai/work-items/archive/WI-296-governance-closure-recovery.contract.json
terminalVerification: .ai/evidence/WI-296-governance-closure-recovery.verification.json
terminalFinalization: .ai/decisions/WI-296-governance-closure-recovery.finalize.json
terminalDecision: .ai/decisions/WI-296-governance-closure-recovery.close.json
authority: canonical
---

# WI-296 — Governance closure recovery

## Intent

Keep a consumed retry visible as historical evidence after its predecessor is
closed, while making the documentation and finalization gates agree with the
Runtime's actual terminal receipts.

## Scope

- Preserve consumed retry history as historical rather than an active error.
- Accept a fully bound direct terminal finalization receipt when merge and
  exact cleanup were observed atomically.
- Keep partial, malformed, foreign, and forked evidence fail-closed.
- Synchronize WI-294 terminal documentation and all three parity ledgers.

## Boundary

Rust Core behavior, release/adopter harnesses, and historical archive bytes are
outside this recovery.

## Acceptance

- Consumed retry history remains historical after a confirmed close.
- A direct terminal finalization receipt is accepted only with merged, deleted
  resource states and a merge identity; transition chains still require
  sequences 1 and 2.
- WI-294 documentation is promoted from immutable closure evidence.
- The complete repository gate and hosted checks pass.

## Verification

The installed Runtime lifecycle, repository governance gates, documentation
acceptance, and hosted quality checks are required before closure.

## Unknowns

User-visible benefit remains explicitly unknown until declared by the Work Item
owner.
