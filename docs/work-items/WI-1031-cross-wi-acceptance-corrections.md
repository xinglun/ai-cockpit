---
author: AI Cockpit maintainers
title: WI-1031 — Cross-WI acceptance corrections
description: Correct collaboration identity, evidence, action admission, and Outcome gaps found by independent acceptance.
workItemId: WI-1031-cross-wi-acceptance-corrections
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-1031-cross-wi-acceptance-corrections
---

# WI-1031 — Cross-WI acceptance corrections

## Intent

Close the remaining behavioral gaps found in the independent review of the cross-Work-Item collaboration changes. This follow-up is in progress; it does not claim the requested behavior has passed or is ready for release.

## Scope

- Derive composition reuse identity and required check coverage from observed repository, executor, Contract, and dependency facts.
- Require current, successful, identity-bound verification evidence and actual target-merge facts.
- Make impact registration recoverable and admission outcome-specific, including transitive dependencies.
- Keep historical composition receipts while reporting current staleness, temporary composition, actual merge, and safely paused state separately.
- Preserve ordinary single-Work-Item serial execution.

## Out of scope

Release, tag mutation, publishing, Runtime upgrade, changes to WI-1030's immutable history, global Agent/MCP configuration, and unrelated collaboration redesign.

## Acceptance

1. Stale caller JSON cannot authorize reuse after an observed toolchain or environment change; unknown execution inputs disable reuse.
2. Required scenarios, participants, composition order, and dependency closure are covered before any command starts; a consumer worktree can target the resolved `main` ref.
3. Only supported successful receipts bound to the current provider identity satisfy verification dependencies; `MergedTarget` reflects actual target integration.
4. Registration/event interruption is recoverable; same-Work-Item actions are filtered by outcome and invalidation propagates through three levels.
5. Outcome retains history but marks stale attempts, separates composition from target merge, and agrees with pause admission.
6. An ordinary single-Work-Item workflow remains serial and does not require cross-WI coordination state.

## Evidence

Focused evidence currently passes: composition/reuse (17 tests), collaboration admission/projection (25 tests), coordination storage/recovery (14 tests), the repository gate-manifest regression, and real multi-process acceptance using two linked worktrees. The process acceptance recorded two registrations, one deduplicated impact event, and verification-process counts of one on the initial composition and zero on the exact repeat. The cases also cover changed executable/environment negatives, selective and transitive invalidation, invalid dependency-order rejection before launch, and ordinary serial single-WI verification. Whole-workspace tests, Clippy, the full canonical Runtime/Cargo/CI gate, independent PR review, merge, and cleanup remain pending. This Work Item still stops before release. See the [specification](WI-1031-cross-wi-acceptance-corrections/spec.md) and [implementation plan](WI-1031-cross-wi-acceptance-corrections/implementation-plan.md).
