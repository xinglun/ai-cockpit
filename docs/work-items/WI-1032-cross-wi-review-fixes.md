---
author: AI Cockpit maintainers
title: WI-1032 — Cross-WI review corrections
description: Close the remaining identity, admission, recovery, reuse, and MCP-contract gaps from independent acceptance.
workItemId: WI-1032-cross-wi-review-fixes
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-1032-cross-wi-review-fixes
---

# WI-1032 — Cross-WI review corrections

## Intent

Complete the bounded follow-up to the independent review of cross-Work-Item collaboration. This projection is in progress; it does not claim full verification, acceptance, or release readiness.

## Scope

- Bind registration, composition admission, evidence, target merge, resource generations, and Outcome state to observed repository facts.
- Make invalidation registration recoverable, preserve append-only event history, and filter action admission by the selected provider/outcome pair.
- Reuse composition results only when executable, bounded read-set, environment, and upstream receipts are observed unchanged; otherwise execute again.
- Give each MCP coordination action an unambiguous provider, Work Item, event, and consumer identity contract.
- Keep query paths read-only, writes explicit, and ordinary single-Work-Item verification serial.
- Validate this Runtime candidate against Sentinel read-only; implement Sentinel-side behavior only in a later, separate Work Item.

## Out of scope

Release preparation or publication, tag mutation, Runtime installation/upgrade, Sentinel source or lifecycle writes in this Work Item, global Agent/MCP configuration, and broad coordination-architecture redesign.

## Acceptance

1. Registration and dependencies are admitted from verified repository, Contract, branch/head, evidence, and generation facts; a safely paused execution cannot start composition.
2. Required composition participants/checks and actual target merge state are verified, attempts and cleanup remain recoverable, and reusable nodes are selected from complete observed identities.
3. Impact reporting, recovery, and action admission remain consistent across interruptions, generations, provider/outcome pairs, and transitive consumers.
4. MCP schemas and handlers define one identity meaning per action, with CLI/MCP parity and canonical gate coverage.
5. Read-only inspection does not persist collaboration state; explicit report/publication/coordination/recovery operations do.
6. Real multiple-process and linked-worktree acceptance, the ordinary serial path, canonical verification, independent PR review, merge, and authorized cleanup complete before this Work Item is closed. Stop before release.

## Evidence and current state

Tasks 1–7 are implemented in serial commits. Focused composition, coordination-store, and CLI/MCP query-write tests pass, as do the gate-manifest regression, release CLI build, documentation acceptance, and real multi-process acceptance. The real acceptance uses two linked worktrees; composition process counts are 1 → 0 for an exact repeat → 1 after changing an observed environment input while keeping the JSON byte-identical. Query tests compare exact coordination paths and bytes before and after absent-store and populated-store reads; only explicit registration and impact-report writes persist records, which a fresh MCP process then observes. Runtime-bound full verification, the read-only Sentinel compatibility check, independent PR review, merge, and exact cleanup remain pending. These local results are development evidence, not Runtime-bound verification or release approval. See the [specification](WI-1032-cross-wi-review-fixes/spec.md), [implementation plan](WI-1032-cross-wi-review-fixes/implementation-plan.md), and [independent review](WI-1032-cross-wi-review-fixes/independent-review.md).
