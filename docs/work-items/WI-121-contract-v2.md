---
author: AI Cockpit maintainers
workItemId: WI-121-contract-v2
title: Contract V2 semantics, strict validation, and fail-closed preflight
description: Add typed Contract V2 semantics, strict parsing, and fail-closed preflight review.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-121-contract-v2
---

# WI-121 — Contract V2 semantics and fail-closed review

## Purpose

Align the Rust Contract boundary with the reference source without copying its
runtime. A Contract must preserve explicit intent, scope, authority, evidence,
and human decisions, and malformed or unknown governance input must stop before
implementation.

## In scope

- typed Contract V2 additions with additive legacy reading;
- structured intent, sources, verification, capability and execution declarations;
- strict unknown-field, duplicate-key, schema and cross-field validation;
- structured preflight human-decision request and repository-bound review receipt;
- fail-closed checkpoint and lifecycle transition validation;
- CLI/MCP machine and human projections in three languages.

## Out of scope

Scenario/final-dimension aggregation belongs to WI-122. Contract-level parallel
slots and serialized projection leases belong to WI-123. Contract source text is
never machine-translated and historical bytes are never rewritten.

## Verification

Focused protocol, preflight, lifecycle and projection regressions plus the full
locked Rust workspace quality gates. The final human Outcome must retain its
traffic-light marker, unknowns, evidence, decision, and next action.
