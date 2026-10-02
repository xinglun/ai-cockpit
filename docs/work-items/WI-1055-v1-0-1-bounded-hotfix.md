---
author: AI Cockpit maintainers
workItemId: WI-1055-v1-0-1-bounded-hotfix
title: Bounded repair-release exception
description: Record the narrow WI-1052 and issue 1004 repair-release scope, the Runtime bootstrap rejection, actual tests, and unresolved publication evidence.
audience: [maintainer, reviewer]
status: in-progress
authority: user-authorized-project-bootstrap-exception
lastVerifiedBy: pending-exact-head-acceptance
---

[简体中文](WI-1055-v1-0-1-bounded-hotfix.zh-CN.md) · [日本語](WI-1055-v1-0-1-bounded-hotfix.ja.md)

# WI-1055 bounded repair-release exception

Status: implementation under a user-authorized, project-local bootstrap exception; **not** Runtime-admitted, verified, closed, merged, or released.

## Exact boundary

- Base: `main` at `78ae7240aac016f005fcf4ced61fcb251ea20eb1`.
- Candidate reference only: `befdbd11d60af5c01d40ec62497aa9c8b46ba0ce`. Port selected product code, tests, and documentation; do not copy its WI-1053 active Contract, Summary, amendment receipts, controls input, or decisions.
- Allowlist: WI-1052's identity-bound pending amendment-review request generation without automatic approval; issue #1004's strict cross-checkout historical closeout recovery, raw finalization binding, atomic rollback, and adopter continuation regression.
- Excluded: Task9 script migration, historical WI-1041/1042/1043/1051/1053 closeout, generic scope relaxation, deletion or editing of old receipts, provider resources in Sentinel, and claims of cross-platform/stable acceptance based only on Mac evidence.

## Why this is an exception

The installed Runtime `1.0.1-rc.1` generated the WI-1055 scaffold but rejected activation with:

> `start work item: repository protocol state error at /Users/sei-rinn/.codex/worktrees/hotfix-v1-0-1/ai-cockpit: lifecycle entry rejected before start: archived Work Item scope conflict: archived_work_item_scope_conflict:WI-1042-contract-amendment-environment-drift, archived_work_item_scope_conflict:WI-1043-amendment-review-admission-fix`

The scaffold remains `not_ready`. The user directed a narrow repair release ahead of ordinary governance-debt resolution, and delegated ordinary technical decisions to Raydot. This note is a human-readable scope and debt record, **not** a Runtime receipt, an identity-bound human review, or a claim that the rejected action was admitted. Do not retry the same start/amend admission or alter old WI evidence to make it pass.

## Required proof and publication limit

1. Record exact source-to-new-branch mapping and test each behavior against the new integrated source snapshot.
2. Verify pending review never starts a verification process. Verify #1004 identity mismatch, tamper, races, idempotency, rollback, and raw historical digest.
3. For Sentinel continuation, first correct its formatter Contract scope to an exact file path through its authorized process, then observe whether six recovered historical receipt files still independently trigger `scope_exceeded`. Never globally ignore `.ai/` or delete those receipts.
4. Obtain an independent code review and exact-head PR CI. Build, download, checksum-check, install, and run `doctor` on Mac arm64. Preserve raw test outputs separately; do not manufacture a Runtime verification receipt.
5. `v1.0.1-rc.1` is immutable. `v1.0.1-rc.2` was not reserved when this note was written; recheck tag and Release before use. Prefer a fresh prerelease while cross-platform and governance closure remain unproven. Release notes must disclose the exception and deferred work.

Rollback: before publication, do not tag or release a failing candidate. After an immutable publication, do not overwrite its tag or assets; document the defect and publish a new corrective version. Keep WI-1055 and all predecessor evidence available for later governance reconciliation.

## Current evidence, not a completion claim

- New-branch focused tests passed: WI-1052 CLI amendment process 4/4; #1004 repository recovery 3/3, CLI recovery 5/5, MCP recovery 1/1; repository library 54/54 including race, interruption, and rollback.
- The Sentinel formatter Contract was amended to the exact source path. Its next real preflight changed from red `scope_exceeded` to yellow with no blockers and an identity-bound `contract-preflight-review` request. The six historical recovery files remain in `Summary.changedPaths`, but the latest preflight did **not** independently classify them as a scope blocker. Verification remains paused for authentic human review. Do not add a product-wide path ignore based on the earlier confounded red result.
- Full workspace, hosted CI, independent review, Mac artifact acceptance, and public Release are pending.

Source mapping: `9025949c` supplied the WI-1052 ordering change and CLI negative tests; `6b9adcf1` supplied #1004 planner/recovery plus CLI/MCP adapters and tests; `6815002f` supplied raw historical finalization binding and corresponding tests. The port was applied as selected file hunks, not as an old WI commit or record import.
