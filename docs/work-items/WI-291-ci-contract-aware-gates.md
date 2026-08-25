---
author: AI Cockpit maintainers
title: "WI-291 — CI Contract-aware quality gate"
workItemId: WI-291-ci-contract-aware-gates
description: "Immutable failed delivery preserved after hosted stale-parity rejection; WI-292 re-delivers the same bounded batch."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-291-ci-contract-aware-gates
authority: canonical
---

# WI-291 — CI Contract-aware quality gate

## Purpose

WI-291 delivered the bounded Rust Contract-aware CI gate, but hosted quality
rejected its documentation projection because parity was registered after
verification. Its lifecycle bytes remain immutable; WI-292 is the explicit
successor and does not rewrite this attempt.

## Boundary

- Preserve the WI-291 archive, verification, blocked finalization, and recovery
  records exactly.
- Do not treat the failed PR as merged or released.
- Re-deliver the same implementation only through WI-292 from the latest remote
  default branch.

## Object/adopter parity

The same installed Runtime, explicit repository context, fail-closed lifecycle,
and visible human Outcome apply to this repository and a fresh adopter.

## Verification

The hosted PR result is retained as failed delivery evidence; WI-292 owns the
fresh verification and provider lifecycle.
