---
author: AI Cockpit maintainers
title: "WI-469 — reference ledger parity order recovery"
description: "Recover the immutable WI-468 delivery and register its parity projection before new verification evidence."
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-469-reference-ledger-parity-order-recovery
predecessorWorkItemId: WI-468-reference-ledger-parity-promotion
status: in_progress
authority: authorized
lastVerifiedBy: WI-469-reference-ledger-parity-order-recovery
---

# WI-469 — reference ledger parity order recovery

## Intent and boundary

WI-469 is the explicit recovery successor for immutable WI-468. Its purpose is
to preserve every predecessor archive/evidence byte while correcting the
documentation projection order that the hosted governance gate rejected.

The fixed local reference is `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`.
It is used for semantic comparison only; no source Runtime, Python module, or
reference repository state is copied into this Work Item.

## Scope

- Keep the manifest-derived current snapshot identical across the three
  comparison pages.
- Mark WI-467 and WI-468 projections as recovered in all three language pages.
- Register the WI-469 parity row in all three ledgers before WI-469 verification
  evidence is created, with terminal record paths reserved explicitly.
- Keep predecessor records immutable and retain the recovery lineage.
- Keep the documentation and conformance gates fail-closed for count, status,
  language, and history-order drift.

## Acceptance

1. WI-467 and WI-468 documentation projections are `recovered` consistently in
   English, Simplified Chinese, and Japanese and bind their recovery evidence.
2. The WI-469 row is present in all three parity ledgers before its verification
   evidence appears in Git history.
3. The manifest-derived snapshot and three-language reader routes pass their
   regression gates; a deliberately stale table or row fails closed.
4. No predecessor archive, evidence, recovery, or historical source bytes are
   rewritten.

## Verification

- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `bash tests/docs/parity_status_check.sh`
- the Contract-declared workspace quality gate

## Recovery boundary

The CI rejection of WI-468 was a deterministic ordering defect: its own
terminal parity row was first introduced after its verification evidence. This
successor registers its row first, then performs fresh verification. The
predecessor remains historical/recovered rather than being rewritten or
reclassified as a current success.
