---
author: AI Cockpit maintainers
title: "WI-265 — Finalization and parity recovery"
workItemId: WI-265-finalization-parity-recovery
description: "Recover the WI-263 closure boundary without rewriting immutable history."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-265-finalization-parity-recovery
terminalArchive: .ai/work-items/archive/WI-265-finalization-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-265-finalization-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-265-finalization-parity-recovery.finalize.d2ffd7299322f97652f941f2ba7a640ba750d0aa9d625cbd4edd4f169a5ec20d.json
terminalDecision: .ai/decisions/WI-265-finalization-parity-recovery.close.json
authority: canonical
---

# WI-265 — Finalization and parity recovery

## Intent

The WI-263 archive is immutable, but its merged PR left a stale pre-merge
finalization head and a parity projection that still described the work as
waiting for merge. This successor owns only the new closure boundary. It does
not rewrite WI-263 or treat a missing close receipt as a completed decision.

## Scope and evidence boundary

- Record a Runtime-bound successor recovery decision for WI-263.
- Register the English, Simplified Chinese, and Japanese parity rows before
  archive, then promote them only after merge and exact cleanup evidence exists.
- Bind this Work Item's own branch, worktree, provider, and reviewed PR with
  `work-item finalize-plan` before verification and archive.
- Complete the hosted PR lifecycle and exact cleanup only from the merged
  reviewed head; a missing, stale, or foreign receipt remains blocked. The
  Runtime records the merge observation, deletion transition, and structured
  close decision.

The WI-263 archive, Outcome, Summary, Events, verification, old recovery, and
old finalization bytes remain historical and are never edited.

## Failure and recovery cases

The governance gate must fail closed when any parity language is missing, when
the finalization receipt is absent, or when the recorded head drifts beyond the
reviewed checkout. The closed successor advances the closure boundary without
changing predecessor bytes.

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/ci/docs_parity_regression_test.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- installed Runtime `inspect`, `status`, `doctor`, `agent doctor`, lifecycle,
  finalization verification, close, and `work-item outcome` with an explicit
  `--repo`

The final human handoff must show `Outcome: 🟢`, `Outcome: 🟡`, or
`Outcome: 🔴`, with status, unknowns, evidence, human decision, and next
action. The final parity rows, deletion transition, and close receipt are the
terminal records for this Work Item.
