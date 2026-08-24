---
author: AI Cockpit maintainers
title: "WI-258 — Governance fixture registry regression"
workItemId: WI-258-governance-fixture-regression
description: "Keep governance-integrity fixtures schema-complete after pending parity validation was introduced."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-258-governance-fixture-regression
authority: canonical
---

# WI-258 — Governance fixture registry regression

## Intent

Make every generated governance-integrity fixture explicit about its empty
pending-parity registry. A fixture must not fail because the fixture builder
omitted a repository-owned control file.

## Scope

The change is limited to the fixture builder, its gate regressions, and the
three-language Work Item/parity projections. It does not change the Runtime
validator or production governance semantics.

## Acceptance

- Every generated fixture has a regular `docs/reference/pending-parity-registry.json`
  with `schemaVersion: 1` and `entries: []` unless the test explicitly adds a
  pending entry.
- Valid and adversarial governance-integrity and pending-registry tests pass
  with deterministic reports.
- The implementation and its evidence remain bound to the archived Contract,
  verification receipt, finalization receipt, and close decision after review.

## Evidence boundary

The empty registry is a fixture baseline, not a declaration that a real Work
Item is pending. Tests that exercise a pending registration write that entry
explicitly and continue to validate its identity, parity rows, and lifecycle.

## Recovery boundary

WI-258 remains immutable historical delivery. Its Runtime close is confirmed,
but the human decision is descriptive prose rather than the canonical
`approved` value required by the documentation promotion gate. The exact
records are preserved and the bounded successor [WI-259](WI-259-close-decision-recovery.md)
projects this predecessor as recovered; no WI-258 `.ai` byte is rewritten.
