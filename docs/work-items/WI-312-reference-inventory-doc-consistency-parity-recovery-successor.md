---
author: AI Cockpit maintainers
title: "WI-312 — reference inventory documentation parity recovery successor"
workItemId: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
description: "Re-deliver the manifest-derived inventory counts with prearchive tri-language parity registration after WI-311's immutable retry boundary."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
---

# WI-312 — reference inventory documentation parity recovery successor

## Intent and boundary

This successor re-delivers the bounded inventory documentation correction from
the latest `origin/main`. WI-311 remains immutable historical evidence after
the installed Runtime rejected a second completion event during recovery. This
Work Item does not change inventory classifications or Runtime behavior.

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

