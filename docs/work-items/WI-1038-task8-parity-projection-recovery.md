---
author: AI Cockpit maintainers
workItemId: WI-1038-task8-parity-projection-recovery
title: Task 8 parity projection recovery
description: Repair the late immutable WI-1037 parity registration through its exact Runtime-bound successor, preserving the historical ordering warning and fail-closed behavior for unowned recovery.
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1038-task8-parity-projection-recovery
---

[简体中文](WI-1038-task8-parity-projection-recovery.zh-CN.md) · [日本語](WI-1038-task8-parity-projection-recovery.ja.md)

# WI-1038 — Task 8 parity projection recovery

This bounded successor repairs the late registration of archived WI-1037 in the three parity ledgers. WI-1037's archive and verification evidence are immutable; the registration must remain visibly historical rather than being rewritten as if it preceded verification.

## Boundaries

- Accept the historical projection only when the valid Runtime successor decision binds WI-1038 to WI-1037 and both WI-1038 Contract scope and Summary changed paths own all three parity documents.
- Keep the predecessor's original ordering visible as a historical warning. A missing, malformed, foreign, mismatched, or partially scoped recovery remains blocking.
- Preserve the existing exact PR lifecycle gates and the immutable WI-1037 archive, Summary, Outcome, events, and verification evidence.
- No generic waiver, broad governance redesign, Task 9 migration, release, version change, tag, or publication is in scope.

## Acceptance

1. A real-Git positive fixture accepts an identity-valid successor recovery covering every parity ledger and reports three `recovered_postarchive_parity_registration` historical warnings.
2. A negative fixture with even one parity document outside successor Contract scope remains blocking with `stale_prearchive_parity_registration`.
3. English, Chinese, and Japanese rows accurately project WI-1037's immutable archive and WI-1038's archived-but-awaiting-merge-and-close lifecycle without claiming either is closed.
4. WI-1038's registered parity row remains valid through its archive transition in the focused lifecycle fixture.
5. Declared focused checks and exact-head hosted CI pass before PR #997 is merged. Merge, Task 8 cleanup, and Task 9 start remain subject to Runtime admission; stop before release for human review.

The focused local verification passed and Runtime archived WI-1038. The post-archive parity check exposed that the current row is introduced in Git after its verification evidence; preserve the evidence and commit ordering so the registered row precedes the evidence path. Exact-head hosted CI, PR merge, formal close, and cleanup remain pending.
