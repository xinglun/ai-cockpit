---
author: AI Cockpit maintainers
title: "WI-287 — Reference checkpoint conformance"
workItemId: WI-287-reference-checkpoint-conformance
description: "Close the reference checkpoint file-level comparison gap with Rust-native fail-closed regressions and a truthful ledger."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-287-reference-checkpoint-conformance
authority: canonical
---

# WI-287 — Reference checkpoint conformance

## Purpose

This bounded batch closes the remaining file-level comparison gap for the
reference checkpoint source. It proves the Rust-native semantics and makes the
conformance ledger truthful; it does not copy Python, Make, YAML, or V1 wire
formats into the Runtime.

## Compared source files

| Reference file | Rust-native counterpart | Boundary |
| --- | --- | --- |
| `scripts/ai_checkpoint.py` | `cockpit-protocol` typed `CheckpointPolicy`/`CheckpointEvidence`; repository checkpoint and amendment validators | Semantic parity; no Python implementation or direct JSON-wire compatibility claim |
| `tests/test_ai_checkpoint.py` | `agent_risk_checkpoint.rs`, `lifecycle_order.rs` | Rust regression corpus for ordering, stale resume, amendment lineage, and immutable evidence |
| `tests/test_outcome_lifecycle_rules.py` | `agent_rule_parity.rs`, `AGENTS.md`, `.ai/README.md`, `docs/reference/agent-workflow.md` | Project-native Agent rule projection; no template copy |

## Changes

- Reject a `before_edit` checkpoint when any verification result already
  exists, preserving the phase boundary fail-closed.
- Reject an invalid latest `resumeHistory.recordedAt` instead of silently
  treating it as absent.
- Strengthen static Agent-rule parity assertions for current-Work-Item repair,
  visible Outcome terminality, and narrow successor creation.
- Register the two checkpoint source files as
  `implemented-different-by-design` in the pinned comparison ledger.

## Object/adopter boundary

The same controls apply to a fresh adopter because they are Runtime and
repository-protocol behavior, not source-template behavior. Every command
still requires explicit repository context; unknowns remain visible and the
human Outcome remains the handoff boundary.

## Verification

Declared verification: `cargo test --locked --workspace`, conformance ledger
regression, documentation acceptance, and the repository governance gate.
