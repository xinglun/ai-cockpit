---
author: AI Cockpit maintainers
title: "WI-269 — Finalization head-order correction"
workItemId: WI-269-finalization-head-order-correction
description: "Complete finalization only after the reviewed archive/evidence commit is stable."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-269-finalization-head-order-correction
authority: canonical
---

# WI-269 — Finalization head-order correction

## Intent

WI-268 exposed a sequencing defect: a pre-merge finalization receipt was
recorded before the evidence/archive commit, so the reviewed head became stale.
This successor records the parity registration first, commits archive/evidence,
then records finalization against that stable head.

## Scope and evidence boundary

- Preserve WI-268 and WI-267 immutable recovery bytes.
- Register the successor parity row before evidence appears in Git history.
- Commit archive/evidence before recording the pre-merge finalization receipt.
- Keep the finalization commit limited to the canonical receipt and complete
  hosted review, exact cleanup, and structured close.

## Verification

- `cargo test --locked --workspace`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- installed Runtime lifecycle and visible human Outcome with explicit `--repo`

The final handoff must be a visible `Outcome: 🟢`, `Outcome: 🟡`, or
`Outcome: 🔴` with status, unknowns, evidence, human decision, and next action.
