---
author: AI Cockpit maintainers
title: "WI-275 — reference inventory finalization recovery"
workItemId: WI-275-reference-inventory-rebaseline-finalization-recovery
description: "Redeliver the bounded file-level reference inventory after preserving WI-274's immutable stale-finalization failure."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-275-reference-inventory-rebaseline-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-275-reference-inventory-rebaseline-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-275-reference-inventory-rebaseline-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.finalize.6447db8eaff82a97764a341b733710a51f6574664c28398b40f2026c52f4469b.json
terminalDecision: .ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.close.json
authority: canonical
---

# WI-275 — reference inventory finalization recovery

## Intent

Re-establish the machine-readable, file-level reference comparison baseline at
`origin/main@487f01970c49e2b85d17b0cb0536f9d60c8f05e0`. WI-274 remains an
immutable predecessor because its pre-merge finalization receipt was recorded
before the final documentation correction and therefore binds a stale head.

## Scope

- Rebind inventory metadata, path digests, and documentation counts to the
  synchronized default branch.
- Preserve WI-274's immutable failure and recovery lineage without rewriting it.
- Commit parity registration before any generated verification evidence.
- Keep the three language comparison and parity documents synchronized.
- Record provider finalization only after the final commit is stable.

## Boundary

This recovery does not rewrite WI-274 history, weaken the governance gate, add
Runtime behavior, change CI architecture, or perform the deferred architecture
cleanup. It is limited to the reference inventory comparison batch.

## Acceptance

- Inventory metadata and path digests match the pinned target commit.
- WI-274's immutable evidence and successor relationship remain auditable.
- The parity prearchive row is present in an earlier commit than verification
  evidence, and the governance gate proves that ordering.
- English, Chinese, and Japanese documents have identical baseline and counts.
- Inventory, documentation, governance, workspace, hosted, finalization, and
  cleanup checks pass.

## Verification

- installed Runtime with explicit `--repo`
- reference inventory and documentation acceptance scripts
- repository governance and release policy gates
- `cargo test --locked --workspace`
- hosted PR checks and finalization/cleanup evidence

## Terminal evidence (planned)

- Archive: `.ai/work-items/archive/WI-275-reference-inventory-rebaseline-finalization-recovery.contract.json`
- Verification: `.ai/evidence/WI-275-reference-inventory-rebaseline-finalization-recovery.verification.json`
- Finalization: `.ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.finalize.json`
- Close: `.ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.close.json`
