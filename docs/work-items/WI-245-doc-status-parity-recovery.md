---
author: AI Cockpit maintainers
title: "WI-245 — Documentation status and parity recovery"
workItemId: WI-245-doc-status-parity-recovery
description: "Recover WI-240 on current main and bind stale conditional documentation to terminal repository evidence."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-252-manifest-gate-order-recovery
authority: canonical
---

# WI-245 — Documentation status and parity recovery

WI-245 was the Runtime-recorded successor to immutable failed delivery WI-240.
Its own delivery then failed hosted quality because the repository gate IDs
were not globally sorted. `.ai/decisions/WI-245-doc-status-parity-recovery.recovery.json`
preserves the immutable failure and assigns current redelivery to WI-252.

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

WI-245 is not registered as a current parity Work Item on this branch because
its immutable archive remains outside the successor delivery. WI-252 replays
only the still-applicable implementation and documentation changes, and must
complete its own Runtime verification, hosted review, and structured close.

## References

- [WI-240 predecessor](WI-240-doc-status-consistency.md)
- [Reference file comparison](../reference/reference-file-comparison.md)
- [Reference source parity](../reference/reference-parity.md)
- [Release distribution](../release/distribution.md)
