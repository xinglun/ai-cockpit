---
author: AI Cockpit maintainers
workItemId: WI-125-contract-schema
title: Contract V2 schema completeness
description: Add the remaining typed Contract V2 lineage and governance fields without rewriting legacy bytes.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
---

# WI-125 — Contract V2 schema completeness

## Purpose

Complete the Rust Contract boundary needed to read the reference Work Item
model while keeping the shared Runtime and repository-local Protocol separate.
This Work Item adds typed fields and deterministic cross-field checks; it does
not copy the reference Python runtime or Makefile workflow.

## Delivered

- Added typed support for `baseCommit`, `baselineDirtyPaths`,
  `archiveSequence`, `resumeHistory`, `synchronizationCheckpoint`,
  `synchronizationHistory`, `guidelines`, `preReviewWarnings`, and optional
  `acceptance`.
- Added typed repository-local authority and destructive approval evidence with
  explicit identity level, actor, scope, and evidence payload.
- Enforced Contract V2 mode semantics: `investigate`, `author_todo`, `code`,
  `review`, and `cleanup`; `code` requires empty `unknowns` and
  `notCodable: false`.
- Rejected unknown nested fields, malformed lineage, empty paths/guidelines,
  unauthorized synchronization checkpoints, non-contiguous history, and
  insufficient approval evidence.
- Kept protocol-v1 records readable and did not rewrite historical Contract
  bytes. Legacy `baseRevision` and one-line intent remain supported.

## Boundary

Summary, WIII, Outcome, evidence strictness, release checks, README, MCP, and
the reference Python/Makefile runtime remain outside this Work Item. Approval
records describe repository provenance; they do not authenticate a person or
replace provider/enterprise review.

## Verification

- `cargo test --locked -p cockpit-protocol --test contract_v2`
- `cargo test --locked -p cockpit-repository --test contract_schema`
- Full locked workspace tests and lint checks are required before merge.

The human handoff must show `Outcome: 🟢`, `Outcome: 🟡`, or `Outcome: 🔴`, with
status, unknowns, evidence, human decision, and next action visible without
relying on a folded log.
