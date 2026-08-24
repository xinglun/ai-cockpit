---
author: AI Cockpit maintainers
title: "WI-244 — Pending parity registry"
workItemId: WI-244-pending-parity-registry
description: "Add a typed, fail-closed pre-merge registry for parity rows that must be delivered by a separate documentation change."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-246-pending-parity-merge-ref-recovery
authority: canonical
---

# WI-244 — Pending parity registry

Code Work Items can reach a valid archive and pre-merge finalization boundary
without authority to edit the tri-language parity ledger. Requiring those rows
in the same PR creates a scope and finalization-head deadlock. WI-244 adds a
strict pending registry without copying or rewriting predecessor `.ai` bytes.

## Boundary

- The registry is empty by default and is not a general exemption list.
- A pending entry binds repository, full Work Item, provider PR, Contract base,
  canonical finalization head, registry append parent, exact record paths,
  three exact `In progress` rows, and creation time.
- Normal archive, verification, and finalization checks remain authoritative.
- Only the three absent parity rows are deferred. Foreign, malformed, missing,
  mismatched, symlink, duplicate, stale, merged, partial, or unrelated inputs
  fail closed.
- After merge, a documentation change adds all three rows and removes the
  pending entry atomically. It does not modify predecessor history.

## Verification

The focused registry regression exercises the valid Git topology plus foreign,
head/base/PR/path/row mismatch, duplicate-key, missing-record, symlink,
unrelated-append, partial-row, and default-branch cases. Manifest and route
tests require this regression at light, standard, and strict profiles.

## Recovery

WI-244 reached verified archive and pre-merge finalization on PR #196. Its
hosted merge ref later combined the feature tree with the authoritative
WI-243 close receipt from the default branch, exposing a tri-language parity
drift outside the immutable predecessor snapshot. Runtime recovery receipt
`.ai/decisions/WI-244-pending-parity-registry.recovery.json` binds the exact
Contract, Summary, Outcome, and Events digests to WI-246. The predecessor
archive, evidence, finalization, PR, and hosted-run bytes remain unchanged.
