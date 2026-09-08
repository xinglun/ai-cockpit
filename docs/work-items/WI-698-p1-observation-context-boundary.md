---
author: AI Cockpit maintainers
title: "WI-698 — P1 explicit observation context boundary"
description: "Bind one governance judgment to one validated, phase-scoped repository observation context."
audience: [contributor, maintainer, reviewer]
workItemId: WI-698-p1-observation-context-boundary
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-698-p1-observation-context-boundary
---

[简体中文](WI-698-p1-observation-context-boundary.zh-CN.md) · [日本語](WI-698-p1-observation-context-boundary.ja.md)

# WI-698 — P1 explicit observation context boundary

## Intent

Complete the remaining P1-B architecture boundary without extending snapshot
validity across mutations. A governance judgment should consume one explicit,
request-scoped observation phase rather than letting lower-level helpers
silently resolve repository identity and snapshot facts again.

## Boundary

`RepositoryExecutionContext` now produces an `ObservationContext` for a named
phase (`before_governance`, `after_execution`, or `before_persistence`). The
context binds repository identity, optional Runtime identity, Contract identity,
source snapshot digest, governance configuration/policy identity, parsed
repository observation, project-governance facts, unknowns, and consistency.

`validate_current` takes a fresh boundary check. Source changes, attached
identity changes, Contract changes, or governance configuration changes reject
the old context. A phase cannot be reused for a different lifecycle phase.
The preflight governance path consumes this context and reuses its parsed
project-governance facts; compatibility wrappers and existing protocol bytes
remain available.

## Compatibility and limits

This Work Item adds no crate, trait, global cache, protocol field, governance
rule, Outcome wording, lifecycle transition, or physical-execution policy. A
context is not a transaction and does not make multi-file reads atomic. After
execution or before persistence, callers must capture a new phase context.
No benchmarked performance benefit is claimed.

## Verification

The focused context tests cover request reuse, source mutation, governance
configuration mutation, repository identity mutation, Contract mutation, and
phase separation. Lifecycle preflight/order tests and the full workspace gates
remain required before finish. Hosted checks and the Runtime archive,
finalization, and close receipts are the authority for terminal status.

## Remaining risk

Other lifecycle entry points retain compatibility wrappers and may still use
the older root-plus-snapshot form until a separately bounded migration. This
Work Item does not claim a repository-wide atomic snapshot or a cross-request
cache.
