---
author: AI Cockpit maintainers
title: "WI-274 — reference inventory rebaseline recovery"
workItemId: WI-274-reference-inventory-rebaseline-recovery
description: "Redeliver the file-level reference inventory from the synchronized default branch while preserving WI-273 immutable failure truth."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-274-reference-inventory-rebaseline-recovery
terminalArchive: .ai/work-items/archive/WI-274-reference-inventory-rebaseline-recovery.contract.json
terminalVerification: .ai/evidence/WI-274-reference-inventory-rebaseline-recovery.verification.json
terminalFinalization: .ai/decisions/WI-274-reference-inventory-rebaseline-recovery.finalize.json
terminalDecision: .ai/decisions/WI-274-reference-inventory-rebaseline-recovery.close.json
authority: canonical
---

# WI-274 — reference inventory rebaseline recovery

## Intent

Re-establish the machine-readable, file-level reference comparison baseline at
`origin/main@487f01970c49e2b85d17b0cb0536f9d60c8f05e0` after the immutable WI-273
delivery was rejected because its parity registration and verification evidence
were introduced in the same commit.

## Scope

- Rebind inventory metadata, path digests, and documentation counts to the
  synchronized default branch.
- Record the WI-273 immutable failed-delivery boundary and link the successor.
- Commit parity registration before any generated verification evidence.
- Keep the three language comparison and parity documents synchronized.

## Boundary

This recovery does not rewrite WI-273 history, weaken the governance gate, add
Runtime behavior, change CI architecture, or perform the deferred architecture
cleanup. It is limited to the reference inventory comparison batch.

## Acceptance

- Inventory metadata and path digests match the pinned target commit.
- The WI-273 immutable evidence and successor relationship remain auditable.
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

- Archive: `.ai/work-items/archive/WI-274-reference-inventory-rebaseline-recovery.contract.json`
- Verification: `.ai/evidence/WI-274-reference-inventory-rebaseline-recovery.verification.json`
- Finalization: `.ai/decisions/WI-274-reference-inventory-rebaseline-recovery.finalize.json`
- Close: `.ai/decisions/WI-274-reference-inventory-rebaseline-recovery.close.json`
