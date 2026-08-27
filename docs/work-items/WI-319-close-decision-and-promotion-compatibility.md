---
author: AI Cockpit maintainers
title: "WI-319 — close decision and promotion compatibility"
workItemId: WI-319-close-decision-and-promotion-compatibility
description: "Keep static promotion and governance consumers aligned with the installed Runtime's close and finalization bindings."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-319-close-decision-and-promotion-compatibility
---

# WI-319 — close decision and promotion compatibility

## Intent and boundary

The installed Runtime permits two explicit positive close decisions (`approved`
and `confirmed`) and can append a deleted sequence-1 finalization transition
before `close`. Static documentation and governance consumers must recognize
those current records while preserving the older post-close reconciliation
path. This Work Item changes only those consumers and their tri-language
documentation; immutable Runtime records are never rewritten.

## Scope and acceptance

- Promotion, status, and governance checks accept current sequence-1
  cleanup-before-close records and the historical root-bound reconciliation
  shape, while rejecting any predecessor, identity, path, or digest mismatch.
- `approved` and explicit `confirmed` structured close decisions are positive;
  `rejected` never promotes a Work Item to Implemented.
- W317's closed projection is represented in all three Work Item documents and
  parity ledgers without changing its immutable archive, verification,
  finalization, or close bytes.
- Regression fixtures cover both finalization paths and the `confirmed`
  decision token; documentation acceptance and governance gates remain strict.
- The Work Item follows the installed Runtime lifecycle and will be finalized,
  reviewed, merged, closed, and exactly cleaned only after hosted checks pass.

## Terminal history

The Runtime checkpoint lifecycle exposed a real snapshot-binding defect before
this Work Item could finish. The immutable W319 records are therefore closed
as a superseded historical item; W320 owns the bounded checkpoint correction.
The predecessor Contract, evidence, blocked Outcome, recovery receipts, and
close decision remain unchanged.

## Verification

Run the focused promotion, status-consistency, governance-integrity, and
documentation regressions, then the locked workspace tests and the hosted
checks for the reviewed branch using the installed Runtime with an explicit
repository context.

## Out of scope

Rust Runtime production code, release/adopter harnesses, immutable `.ai`
archive/decision bytes, global Agent or MCP configuration, and unrelated
reference-comparison batches are outside this bounded compatibility repair.

[简体中文](WI-319-close-decision-and-promotion-compatibility.zh-CN.md) ·
[日本語](WI-319-close-decision-and-promotion-compatibility.ja.md)
