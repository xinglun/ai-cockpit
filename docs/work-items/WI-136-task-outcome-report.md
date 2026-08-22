---
author: AI Cockpit maintainers
workItemId: WI-136-task-outcome-report
title: Rust-native Task Outcome and Human Benefit report
description: Add an evidence-bound report projection, append-only event source, and lifecycle-bound report artifacts.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-136-close-verification
---

# WI-136 — Rust-native Task Outcome and Human Benefit report

## Intent

The current Rust Runtime has a narrow OutcomeV2 and human handoff. The reference
surface additionally makes delivered work, findings, stops, resolutions, risks,
unknowns, evidence, and recovery conditions explicit. This WI adds that
projection without turning presentation into authority.

## Boundaries

- New OutcomeV2 records contain a strict additive `taskOutcomeReport` with
  evidence-bound claims and stable section names.
- `finish` writes a typed report JSON, Markdown projection, and append-only
  `<id>.events.jsonl`; `archive` moves and digests them; `close` records a
  validated `finalReport` and digest in the repository-bound decision receipt.
- Event identity, repository/Work Item binding, relationship order, unsafe
  paths, and secret-like details fail closed. Historical bytes remain unchanged.
- CLI and MCP use the same localized human renderer. Contract source language,
  human decisions, external provider claims, and release truth are unchanged.

## Out of scope

New lifecycle states, full event-sourced paused/blocked/stale/cancelled/rollback
reconstruction, adopter capability manifests, second-stack acceptance, provider
identity, global Agent/MCP configuration, and copied reference Python/Make/V1
assets remain separate boundaries.

## Acceptance

- Protocol tests prove strict report schema, unknown-field rejection, and claim
  provenance.
- Repository tests prove report/event generation, malformed or foreign event
  rejection, archive digest binding, and close final-report binding.
- CLI and MCP expose the same report while preserving three-language headings
  and Contract-language acceptance text.
- English, Simplified Chinese, and Japanese feature/reference documentation
  accurately describes the implemented and deferred boundaries.

## Verification

Archived evidence and the close decision are now the authoritative verification
record:

- `.ai/evidence/WI-136-task-outcome-report.verification.json`
- `.ai/work-items/archive/WI-136-task-outcome-report.archive.json`
- `.ai/decisions/WI-136-task-outcome-report.close.json`
