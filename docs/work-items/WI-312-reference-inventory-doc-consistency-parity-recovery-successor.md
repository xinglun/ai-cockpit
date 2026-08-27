---
author: AI Cockpit maintainers
title: "WI-312 — reference inventory documentation parity recovery successor"
workItemId: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
description: "Re-deliver the manifest-derived inventory counts with prearchive tri-language parity registration after WI-311's immutable retry boundary."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
---

# WI-312 — reference inventory documentation parity recovery successor

## Intent and boundary

This immutable delivery remains historical evidence after its retained provider
finalization could not satisfy the later cleanup gate. Its Contract, Summary,
Outcome, Events, archive, verification, finalization, and close bytes are not
rewritten. WI-314 is the explicit successor that redelivers the bounded
correction and reconciliation boundary from the synchronized default branch.

## Scope and acceptance

The three comparison pages must contain one identical marker derived from the
5,119-record inventory (4,262 generated history, 182 implemented-different,
one equivalent, three not-applicable, two reference-only, 669 deferred, and no
migrate gap). A deterministic conformance test must reject stale, malformed,
missing, or divergent markers. All three parity pages must register this row
before verification evidence, and all three Work Item documents must retain
the same bounded scope and `lastVerifiedBy` metadata.

Verification uses the installed Runtime and the repository documentation and
inventory gates. The source project remains a semantic reference, not a wire
format or runtime dependency.
