---
author: AI Cockpit maintainers
title: "WI-252 — Manifest gate-order recovery"
workItemId: WI-252-manifest-gate-order-recovery
description: "Recover the immutable failed WI-245 delivery and make repository gate IDs globally sorted and unique."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-252-manifest-gate-order-recovery
authority: canonical
---

# WI-252 — Manifest gate-order recovery

WI-252 is the Runtime-recorded successor to the immutable failed delivery
WI-245. The predecessor recovery receipt binds the WI-245 Contract, Summary,
Outcome, Events, archive, verification, and finalization digests. Those
historical bytes remain outside this delivery and are not rewritten.

## Acceptance boundary

- Gate IDs in `tests/ci/repository_gate_manifest.json` are globally lexical and
  unique. `docs_pending_parity_registry_regression` therefore precedes
  `docs_work_item_status_consistency`.
- Duplicate and out-of-order fixture manifests fail closed before route
  selection with the same validation used by hosted quality.
- Still-applicable WI-245 documentation-status, inventory, and release-truth
  changes are replayed on `origin/main@87bfd866`; the absent predecessor archive
  is not falsely registered as a current parity Work Item.
- The pinned comparison remains 720 deferred entries plus exactly four
  capability/profile `migrate-gap` entries. Provider truth remains
  identity-bound and drift-detectable rather than being presented as immutable.

## Verification and lifecycle

The regression first reproduced PR #203's
`gate IDs must be deterministic` failure, then passed the manifest and quality
route suites after ordering the IDs and adding negative fixtures. Full docs,
governance, formatting, clippy, workspace, installed Runtime, and exact-head
hosted checks remain required. This pre-archive row cites the future archived
Contract, verification evidence, canonical finalization, and structured close;
it does not claim completion before reviewed close.

## References

- [WI-245 failed predecessor](WI-245-doc-status-parity-recovery.md)
- [Reference source parity](../reference/reference-parity.md)
- [Reference file comparison](../reference/reference-file-comparison.md)
- [Release distribution](../release/distribution.md)
