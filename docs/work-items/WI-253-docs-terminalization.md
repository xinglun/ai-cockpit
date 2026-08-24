---
author: AI Cockpit maintainers
title: "WI-253 — Closed documentation terminalization"
workItemId: WI-253-docs-terminalization
description: "Terminalize WI-252 documentation from immutable close evidence and reject conditional status for newly closed Work Items."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-253-docs-terminalization
authority: canonical
---

# WI-253 — Closed documentation terminalization

WI-253 is the bounded Runtime-recorded successor to the validly closed WI-252.
The recovery decision binds WI-252's canonical Contract, Summary, Outcome, and
Events digests plus its archive, verification, sequence-2 finalization, and
structured close evidence. None of those immutable records is edited.

## Acceptance boundary

- The English, Simplified Chinese, and Japanese WI-252 Work Item documents and
  reference-parity rows use terminal `implemented` / `Implemented` truth and
  cite the exact persisted terminal evidence paths.
- The status-consistency regression rejects conditional lifecycle wording in
  every newly governed terminal Work Item language counterpart. Historical
  documents before the WI-252 enforcement boundary are not retroactively
  rewritten.
- The reference inventory's target working-tree count warning is an intentional
  negative-fixture result. Its canonical count and digest remain normalized to
  the pinned commit, so the production checker is unchanged.

## Verification and lifecycle

The focused regression first demonstrated that conditional wording in each
language was accepted, then failed on the real stale WI-252 projection, and
passes only after terminal evidence is projected. This active registration
names the future WI-253 archive, verification, finalization, and close paths;
it is not terminal evidence by itself.

## References

- [WI-252 predecessor](WI-252-manifest-gate-order-recovery.md)
- [Reference parity](../reference/reference-parity.md)

