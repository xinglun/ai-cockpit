---
author: AI Cockpit maintainers
title: "WI-265 — Finalization and parity recovery"
workItemId: WI-265-finalization-parity-recovery
description: "Recover the WI-263 closure boundary without rewriting immutable history."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-265-finalization-parity-recovery
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
  archive, with an explicit in-progress status until merge and cleanup exist.
- Bind this Work Item's own branch, worktree, provider, and reviewed PR with
  `work-item finalize-plan` before verification and archive.
- Complete the hosted PR lifecycle and exact cleanup only from the merged
  reviewed head; a missing, stale, or foreign receipt remains blocked.

The WI-263 archive, Outcome, Summary, Events, verification, old recovery, and
old finalization bytes remain historical and are never edited.

## Failure and recovery cases

The governance gate must fail closed when any parity language is missing, when
the finalization receipt is absent, or when the recorded head drifts beyond the
reviewed checkout. A fresh successor recovery may advance the closure boundary
without changing predecessor bytes.

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/ci/docs_parity_regression_test.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- installed Runtime `inspect`, `status`, `doctor`, `agent doctor`, lifecycle,
  and `work-item outcome` with an explicit `--repo`

The final human handoff must show `Outcome: 🟢`, `Outcome: 🟡`, or
`Outcome: 🔴`, with status, unknowns, evidence, human decision, and next
action. “In progress” parity is not a close decision.
