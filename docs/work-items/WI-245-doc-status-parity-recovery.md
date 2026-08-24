---
author: AI Cockpit maintainers
title: "WI-245 — Documentation status and parity recovery"
workItemId: WI-245-doc-status-parity-recovery
description: "Recover WI-240 on current main and bind stale conditional documentation to terminal repository evidence."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-245-doc-status-parity-recovery
authority: canonical
---

# WI-245 — Documentation status and parity recovery

WI-245 is the Runtime-recorded successor to immutable failed delivery WI-240.
It replays only still-applicable documentation-governance content on
`origin/main@87bfd866`, preserves the pinned reference source and all
intervening repository truth, and does not rewrite predecessor or WI-241
lifecycle bytes.

## Acceptance boundary

- The deterministic reference inventory is derived from pinned Git trees,
  excludes dirty/untracked checkout metadata, and retains 720 deferred records
  plus exactly four capability/profile `migrate-gap` records.
- Tri-language Work Item status is checked against authoritative archived
  Contract plus close/recovery evidence. A closed Work Item that still carries
  conditional or after-close parity prose fails deterministically.
- WI-241, WI-249, and WI-251 terminal rows bind their archived Contract,
  verification evidence, canonical finalization, sequence-2 deleted cleanup
  transition, and structured close decision.
- v0.2.31 remains identity-bound and drift-detectable because provider truth is
  `immutable: false`; the durable adopter baseline is
  `aarch64-apple-darwin`, while hosted Linux run `32696048024` remains external
  provider-retained evidence.

## Verification and lifecycle

The Work Item is not complete merely because these projections are present.
The conditional parity registration cites the future archived Contract,
verification, canonical finalization, and structured close paths. Runtime
verification, hosted review, merge observation, exact resource cleanup, and a
structured close remain required.

## References

- [WI-240 predecessor](WI-240-doc-status-consistency.md)
- [Reference file comparison](../reference/reference-file-comparison.md)
- [Reference source parity](../reference/reference-parity.md)
- [Release distribution](../release/distribution.md)
