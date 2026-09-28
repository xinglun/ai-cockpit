---
author: AI Cockpit maintainers
workItemId: WI-1036-task8-ci-recovery
title: Task 8 CI closeout
description: Resolve only the three observed PR #997 CI failure classes, preserve required gates, merge and clean up Task 8, and stop before release.
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1036-task8-ci-recovery
---

[简体中文](WI-1036-task8-ci-recovery.zh-CN.md) · [日本語](WI-1036-task8-ci-recovery.ja.md)

# WI-1036 — Task 8 CI closeout plan

This is the final, bounded CI closeout for Task 8. It addresses only failures recorded on [PR #997](https://github.com/xinglun/ai-cockpit/pull/997), [CI run 36363705256](https://github.com/xinglun/ai-cockpit/actions/runs/36363705256): seven `cockpit-verification` composition tests could not inspect process file descriptors; `docs_closed_work_item_promotion` rejected the still-conditional status of the closed predecessor before post-merge projection; and `ci_manifest_regression` did not reach its expected missing-receipt diagnostic.

The process-observer cause is not yet proven. The existing implementation intentionally fails closed when a process that could own the temporary worktree is not inspectable. Determine whether the failure is a real ownership ambiguity or an unrelated process/scheduling interaction before changing that behavior.

## Bounded execution plan

1. Preserve the exact failed CI run, logs, and current PR heads. Reuse prior receipts only when Runtime confirms exact identity and freshness; do not rerun the archived 14-node hosted verification unless the current Runtime requires it.
2. Add focused failing regressions for the reported composition failures. Diagnose the process-ownership observation, then make the smallest safe correction. Unknown ownership must not become permission to clean up or reuse a live worktree.
3. Add stage-specific documentation-promotion coverage. Pre-merge CI must not require a post-merge projection; the synchronized-main promotion requirement remains mandatory and test-covered.
4. Isolate the missing-receipt negative case so unrelated live-repository documentation state cannot mask the intended diagnostic. Preserve checks for malformed and wrong receipts.
5. Run fresh Runtime preflight, then only its admitted targeted checks, serially in this Work Item. Let canonical PR CI exercise the full required gate set; do not skip or weaken gates.
6. Integrate the corrective PR into the Task 8 branch, confirm PR #997 required checks on its exact head, then merge PR #997 and complete exact local Work Item cleanup.
7. Close Task 8 and proceed directly to Task 9. Stop before release, tag, or publication.

## Reviewable change slices

- Composition ownership/scheduling fix with focused Rust regression coverage.
- Lifecycle-stage promotion eligibility with a regression proving post-merge promotion remains required.
- Deterministic missing-receipt manifest regression.
- This three-language Work Item plan/status projection.

No new collaboration capability, broad guide redesign, unrelated Task 8 remediation, or Task 9 migration belongs in this Work Item. Current state is in progress; this plan does not claim any fix or verification has passed.
