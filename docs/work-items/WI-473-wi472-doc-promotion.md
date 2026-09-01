---
author: AI Cockpit maintainers
title: "WI-473 — WI-472 terminal documentation promotion"
description: "Keep terminal Work Item and parity projections complete before release."
audience: [maintainer, reviewer, adopter]
workItemId: WI-473-wi472-doc-promotion
status: in_progress
authority: authorized
lastVerifiedBy: WI-473-wi472-doc-promotion
---

# WI-473 — WI-472 terminal documentation promotion

## Intent and boundary

Promote the verified WI-472 lifecycle into reader-facing documentation and
keep the recovery and current Work Item parity registrations auditable. This
Work Item changes documentation projections only; immutable `.ai` records,
Runtime code, CI, release artifacts, and object repositories remain outside
its boundary.

## Scope

- Promote WI-472's English, Simplified Chinese, and Japanese pages after close.
- Keep WI-471's authoritative hashed recovery receipt in all parity ledgers.
- Register this Work Item and its terminal paths before archive/close.

## Acceptance

1. The three WI-472 pages and parity rows bind the terminal receipts.
2. The three WI-473 pages and pre-archive parity row pass the governance
   integrity gate.
3. Documentation and reference-inventory checks pass in a clean branch.
4. No immutable governance bytes or object repository are changed.

## Verification

- `cargo test --locked --workspace`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`

## Recovery boundary

If a projection is incomplete, preserve the immutable records and repair the
current documentation Work Item through an explicit amendment and revalidation.
