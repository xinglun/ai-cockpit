---
author: AI Cockpit maintainers
title: "WI-294 — Lifecycle recovery state machine"
workItemId: WI-294-lifecycle-recovery-state-machine
description: "Make human-authorized lifecycle recovery explicit, identity-bound, and repeatable without rewriting predecessor bytes."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-294-lifecycle-recovery-state-machine
terminalArchive: .ai/work-items/archive/WI-294-lifecycle-recovery-state-machine.contract.json
terminalVerification: .ai/evidence/WI-294-lifecycle-recovery-state-machine.verification.json
terminalFinalization: .ai/decisions/WI-294-lifecycle-recovery-state-machine.finalize.json
terminalDecision: .ai/decisions/WI-294-lifecycle-recovery-state-machine.close.json
authority: canonical
---

# WI-294 — Lifecycle recovery state machine

## Intent

Make a human-authorized retry after a failed lifecycle transition explicit, safe, and repeatable.

## Scope

- Restore only a legal checkpointed retry state.
- Preserve blocked Outcome, predecessor digests, and append-only recovery history.
- Allow fresh verification and finish without reusing stale report or completion artifacts.
- Keep the recovery behavior and documentation projections consistent across the Rust Runtime and repository gates.

## Out of scope

Release packaging, adopter acceptance, CI replacement, and Runtime module decomposition remain separate boundaries.

## Acceptance

- Failed finish can be retried only through an identity-bound recovery receipt.
- A retry never fabricates a green preflight or rewrites immutable predecessor bytes.
- Stale recovery candidates do not shadow a newer valid projection.
- Superseded archives remain internally digest-bound.
- Rust, governance, documentation, and hosted checks pass before closure.

## Verification

See `.ai/evidence/WI-294-lifecycle-recovery-state-machine.verification.json` and the reviewed PR/closure receipts.

## Unknowns

The Work Item owner has not declared a user-visible benefit; this remains explicitly unknown.
