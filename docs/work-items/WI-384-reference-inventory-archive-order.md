---
author: AI Cockpit maintainers
title: "WI-384 — reference inventory archive order"
workItemId: WI-384-reference-inventory-archive-order
description: "Redeliver the reference inventory parity documentation from origin/main with a verified finish/archive ordering that preserves snapshot-bound evidence."
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-384-reference-inventory-archive-order
terminalArchive: .ai/work-items/archive/WI-384-reference-inventory-archive-order.contract.json
terminalVerification: .ai/evidence/WI-384-reference-inventory-archive-order.verification.json
terminalFinalization: .ai/decisions/WI-384-reference-inventory-archive-order.finalize.33860f23c671c0707f6b0816ba55089af33c14b244b71855c31fb51af40ac81c.json
terminalDecision: .ai/decisions/WI-384-reference-inventory-archive-order.close.json
---

# WI-384 — reference inventory archive order

## Intent and boundary

WI-384 is the explicit recovery successor of immutable WI-383. WI-383's
archive correctly rejected evidence after generated lifecycle records were
committed between `verify` and `archive`; this Work Item preserves WI-382 and
WI-383 bytes and redelivers the same bounded documentation correction from a
clean `origin/main` base.

## Scope and acceptance

The tri-language comparison pages must match the 5,119-record inventory
marker. The tri-language parity ledgers must register WI-382 and WI-383 as
recovered and WI-384 as the current delivery before verification. Linked
Work Item pages for WI-382, WI-383, and WI-384 must have consistent identity
and status metadata.

The lifecycle ordering is part of the acceptance boundary: bind the reviewed
PR, run `verify`, then `finish`, then `archive`, and only after archive
succeeds commit generated lifecycle records. No predecessor bytes, Runtime,
protocol, inventory classifications, CI/release logic, or global Agent/MCP
configuration may be changed.

## Verification

Use the installed Runtime with an explicit repository path plus the inventory,
documentation-status, and governance-integrity checks. The final handoff must
remain visible to a human; a green Outcome does not authorize merge or release.

[简体中文](WI-384-reference-inventory-archive-order.zh-CN.md) ·
[日本語](WI-384-reference-inventory-archive-order.ja.md)
