---
author: AI Cockpit maintainers
title: "WI-755 — P1 observation context successor"
description: "Complete the explicit observation-context boundary from the current default base."
audience: [contributor, maintainer, reviewer]
workItemId: WI-755-p1-observation-context-successor
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-758-wi755-finalization-recovery
recoveryDecision: .ai/decisions/WI-755-p1-observation-context-successor.recovery.ccd6ce5cf8f1c2a563578437cc313079462d08a88b04fb8b61b734f54bf237a2.json
---

[简体中文](WI-755-p1-observation-context-successor.zh-CN.md) · [日本語](WI-755-p1-observation-context-successor.ja.md)

# WI-755 — P1 observation context successor

## Intent

Complete the remaining P1-B architecture boundary from the current
`origin/main` without reviving the failed PR #697 or PR #698. One governance
judgment must consume one explicit, request-scoped and phase-scoped observation
context rather than allowing lower-level helpers to silently re-observe the
repository.

## Boundary

`RepositoryExecutionContext` produces an `ObservationContext` for
`before_governance`, `after_execution`, or `before_persistence`. The context
binds repository identity, optional Runtime identity, Contract identity and
model identity, source snapshot digest, governance configuration and policy
identity, parsed repository observation, project-governance facts, unknowns,
and consistency. `validate_current` performs a fresh boundary check so source,
attached identity, Contract, or governance-configuration changes reject the
old context. A phase cannot be reused for a different lifecycle phase.

The preflight governance path consumes the context and reuses its parsed
project-governance facts. Compatibility wrappers and existing protocol bytes
remain available.

## Compatibility and limits

This Work Item adds no crate, trait, global cache, protocol field, governance
rule, Outcome wording, lifecycle transition, or physical-execution policy. A
context is not a transaction and does not make multi-file reads atomic. After
execution or before persistence, callers must capture a new phase context. No
benchmarked performance benefit is claimed.

## Verification

Focused tests cover request reuse, source mutation, governance configuration
mutation, repository identity mutation, Contract mutation, phase separation,
and Contract/context mismatch. Repository preflight, project-governance,
observer, workspace format, clippy, and full workspace tests remain required.

## Remaining risk

Other lifecycle entry points retain compatibility wrappers and may still use
the older root-plus-snapshot form until separately bounded migrations. This
Work Item does not claim a repository-wide atomic snapshot or a cross-request
cache.

## Post-merge recovery boundary

PR #735 passed hosted checks and merged at `8dcac7ec`. The reviewed PR head is
`06f7d03f`, while the immutable pre-merge finalization receipt records the
intermediate head `251c3867`. Runtime correctly rejects treating that range as
an unbound finalization transition. The recovery decision above preserves this
history; WI-758 owns the fresh post-merge governance boundary.
