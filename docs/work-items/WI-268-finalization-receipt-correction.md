---
author: AI Cockpit maintainers
title: "WI-268 — Finalization receipt correction"
workItemId: WI-268-finalization-receipt-correction
description: "Correct an immutable invalid pre-merge finalization receipt through an explicit successor."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-268-finalization-receipt-correction
authority: canonical
---

# WI-268 — Finalization receipt correction

## Intent

WI-267 is preserved as immutable recovery history because its generated
pre-merge receipt used a worktree identity that the governance gate rejects.
This successor records the protocol-valid receipt and makes that recovery
visible in all parity documents without rewriting WI-267.

## Scope and evidence boundary

- Bind the successor Contract, branch, worktree, PR, repository, Runtime, and
  archived Contract identity exactly.
- Keep WI-267 archive, verification, invalid finalization, and recovery bytes
  unchanged.
- Update the three parity documents and this Work Item documentation so the
  recovered predecessor and successor relationship are explicit.
- Complete hosted review, verification, finalization, exact cleanup, and
  structured close before promotion.

## Verification

- `cargo test --locked --workspace`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- installed Runtime lifecycle and visible human Outcome with explicit `--repo`

The final handoff must be a visible `Outcome: 🟢`, `Outcome: 🟡`, or
`Outcome: 🔴` with status, unknowns, evidence, human decision, and next action.
