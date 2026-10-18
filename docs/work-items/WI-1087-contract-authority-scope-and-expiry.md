---
author: AI Cockpit maintainers
title: "WI-1087 — finite goal authority scope and expiry"
description: "P1/P2a primitives and P2b denial wiring; production authority remains Unsupported."
audience:
  - maintainer
  - reviewer
workItemId: WI-1087-contract-authority-scope-and-expiry
status: in_progress
authority: p2b-software-development-only
lastVerifiedBy: WI-1087-contract-authority-scope-and-expiry
---

[简体中文](WI-1087-contract-authority-scope-and-expiry.zh-CN.md) · [日本語](WI-1087-contract-authority-scope-and-expiry.ja.md)

# WI-1087 — finite goal authority scope and expiry

Current stage: P2b software wiring on the same Work Item, reusing P1 exact UTC/Clock primitives and the P2a offline evaluator. The production source is Unsupported; no real grant, trusted enrollment or system permission is configured. Full-feature acceptance remains not_run.

## Goal

One verifiable finite goal authorization may cover explicitly allowed sequential/parallel Work Items, repair, retry and recovery without per-WI human reapproval. An old goal cannot authorize a new goal. notBefore/expiresAt are mandatory and finite; completion, revocation or expiry terminates future execution while preserving history.

## Versioned design

The versioned Chinese design is canonical; these three summaries refer to it. It specifies current source identity/goal coverage, narrowing only, locks/effects/time checks, bounded safe stopping, metadata-only extension requests, one approved extension for applicable active WIs, no self-renewal or revival, local-clock/identity limitations, and separate legacy migration.

- [Design v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/design-v1.zh-CN.md)
- [Acceptance v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/acceptance-v1.json)
- [Staged scope v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/implementation-stages-v1.json)

## Dependency and ownership

WI-1086 is the implementation dependency. Shared amendment/status/admission/receipt/lifecycle and ledgers serialize; no second build or parallel shared edits. Independent Clock/type work requires later design review, real ownership isolation and normal admission.

## Version 2 clarifications

The v1 history remains unchanged. The v2 addendum defines revocation/commit ordering and external fencing limits, source-version replacement versus terminal grant/WI state, inherited versus fixed earlier deadlines, and truthful in-flight stop evidence. Current approved software scope is described below; production activation remains excluded.

- [Design v2 addendum](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v2-20261016/addendum-v2.zh-CN.md)
- [Acceptance v2 — planned / not_run](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v2-20261016/acceptance-v2.json)

## v3

Scope narrowing that removes current coverage and actual actor invalidation trigger the same in-flight stopping rules as revocation; R04 is parameterized, all variants not_run.

- [Design v3 clarification](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v3-20261016/addendum-v3.zh-CN.md)
- [R04 parameterized acceptance — not_run](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v3-20261016/acceptance-delta-v3.json)

## P1 progress

Local TDD and module regression evidence covers Clock boundaries, strict model shapes and pure UTC normalization. Initial suites passed: core 12, protocol 12, repository 164 (including 12 new normalization tests). Full workspace verification and CI are recorded separately when actually run. The complete feature acceptance matrices remain not_run.

The executor is `agent:Codex`; the technical plan reviewer is `agent:Raydot`. `lastVerifiedBy` references this Work Item, not a person. Human review provenance preserves the original response in the [decision source](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/p1-activation-20261017/human-decision-source-20261017.json). [Module test evidence](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/p1/module-results.json). P1 originally excluded later stages; the approved P2b substage is described below.

## P2b wiring substage

The optional strict `governanceProfile.goalAuthority` declaration requests enforcement; it does not authenticate an issuer, actor, grant, or enabled mode. Its reference reuses P1 shape and exact precision checks. No default TTL or grant is created. Independently trusted enrollment is required to prevent enabled-mode downgrade; editable declarations cannot prove that assurance.

A shared private check rejects required requests before repository business reads, controls, amendment recovery, shared Store/context operations, verification dispatch and CLI/MCP business handling. Cached contexts recheck. Static help/schema and MCP discovery remain available. Durable write helpers recheck immediately before their commit boundary; the byte-pinned historical material recipe is preserved through external wrappers.

The private ordered-source protocol reuses current P2a facts, fresh Clock samples and the original MonotonicBudget after waits. Existing operation locks precede the bounded source fence, which covers one durable commit or dispatch. Isolated test-owned adapters exercise real linked worktrees and cross-process commit/revocation ordering. These fixtures do not authenticate a production source. A local lock does not freeze time or establish an external atomic commit; unsupported strong external guarantees are rejected.

Directed tests and regression evidence are recorded privately; final Native verification and CI must bind the actual candidate. Temporary process diagnostics were removed by an ordinary reverse commit. Historical failed/unknown attempts remain preserved. This is a development-stage projection, not WI completion or production qualification. P3 managed stopping, metadata-only recovery and source writers, and P4 real adapter qualification/activation remain not_run.
