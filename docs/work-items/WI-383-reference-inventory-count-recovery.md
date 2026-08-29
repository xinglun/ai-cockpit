---
author: AI Cockpit maintainers
title: "WI-383 — reference inventory count recovery"
workItemId: WI-383-reference-inventory-count-recovery
description: "Redeliver the reference inventory count correction with complete tri-language parity registration after WI-382's immutable CI failure."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-383-reference-inventory-count-recovery
---

# WI-383 — reference inventory count recovery

## Intent and boundary

WI-383 is the explicit recovery successor of immutable WI-382. Hosted CI
correctly found that WI-382 corrected the three comparison pages but omitted
the required parity-ledger registration. This Work Item preserves every
WI-382 Contract, evidence, archive, Outcome, and recovery byte and adds only
the missing documentation projection.

## Scope and acceptance

The three `reference-file-comparison` pages must keep the single
5,119-record inventory marker (4,262 generated history, 292 implemented
different-by-design, one equivalent, four not-applicable, 45 reference-only,
515 deferred, and no migrate gap). The three `reference-parity` pages must
register WI-382 as recovered and WI-383 as the current in-progress delivery
before verification evidence is recorded. The three Work Item pages must have
matching identity and status metadata and remain linked to the governed
records.

No Runtime, protocol, inventory classification, CI workflow, release artifact,
or global Agent/MCP configuration is changed. The source checkout is a
semantic reference only; its files are not copied into the target repository.

## Verification

Verification uses the installed Runtime with an explicit repository path and
the repository's inventory, documentation-status, and governance-integrity
checks. WI-382 remains an immutable historical recovery predecessor; only the
WI-383 successor may be promoted after reviewed hosted checks, exact merge,
close, and cleanup.

[简体中文](WI-383-reference-inventory-count-recovery.zh-CN.md) ·
[日本語](WI-383-reference-inventory-count-recovery.ja.md)
