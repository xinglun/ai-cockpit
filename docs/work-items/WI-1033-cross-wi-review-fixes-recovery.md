---
author: AI Cockpit maintainers
title: WI-1033 — Cross-WI review corrections recovery
description: Complete Runtime-bound acceptance and closure for the selected cross-WI review-fix recovery lineage.
workItemId: WI-1033-cross-wi-review-fixes-recovery
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-1033-cross-wi-review-fixes-recovery
---

# WI-1033 — Cross-WI review corrections recovery

## Intent

Continue the same bounded cross-Work-Item review-fix objective through Runtime verification, independent PR review, merge, and exact cleanup. Stop before release.

## Lineage and current state

WI-1033 is the Runtime-bound recovery successor to WI-1032. Runtime archived WI-1032 as replaced with verification claim not_verified and preserved its source records and failed precondition evidence. The implementation commits are present, but WI-1033 still requires its own Contract-bound verification; existing local test reports are not a Runtime verification receipt.

The earlier WI-1031 archive has valid historical verification but a pending human close. Its bytes remain immutable; the selected successor lineage must be resolved only when Runtime has the complete terminal evidence.

The latest full Runtime verification passed 32 nodes on commit `6577f3c0`, but that receipt is now historical: the Contract has since been amended to 19 acceptance criteria and 15 scenarios, and the three newly reviewed defects changed source. Focused local suites currently pass (36 collaboration-admission and 20 coordination-store tests); final canonical gates and Runtime-bound verification remain pending. The added coverage includes invalidation after provider-output removal, path-safe request identity, and enforcing Requested as the initial state. Query Runtime for current freshness; do not treat the earlier 31-node/eight-scenario receipt or the 32-node run as current evidence.

## Scope

- Revalidate trusted registration/composition identity (including matching the execution repository's Git common directory to the CoordinationStore), required-check completeness, crash and live-lock safety, evidence containment, provider-scoped dependencies, actual CLI reuse, MCP identity parity, and the read-only query/write boundary.
- Prove real multi-process linked-worktree behavior and preserve ordinary single-WI serial execution.
- Preserve invalidation for removed provider outputs through dependency chains, and reject unsafe or pre-acknowledged coordination requests.
- Inspect Sentinel with the candidate CLI without writing its source, Contract/evidence, lifecycle Runtime, or coordination store.
- Maintain English, Simplified Chinese, and Japanese projections and reference parity.

## Out of scope

Task 8, the Runtime snapshot-binding behavior across commits, is a separate serial Work Item after this Work Item is integrated and cleaned up. Runtime upgrade, release preparation, tag changes, public publication, Sentinel writes, and history rewriting are excluded.

## Acceptance and verification

See the [specification](WI-1033-cross-wi-review-fixes-recovery/spec.md) and [implementation plan](WI-1033-cross-wi-review-fixes-recovery/implementation-plan.md). All fifteen required scenarios carry explicit expected results and verification plans. The canonical documentation gate is `bash tests/docs/documentation_acceptance.sh`.

## Delivery boundary

The Work Item remains in progress. Current verification freshness is authoritative only in Runtime; the prior receipt is retained as historical evidence. Completion requires final Runtime verification, independent review, hosted CI, merge, exact cleanup, and whatever historical lineage close Runtime admits. Release remains out of scope.
