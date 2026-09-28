---
author: AI Cockpit maintainers
workItemId: WI-1037-task8-pr-lifecycle-gate
title: Task 8 PR lifecycle gate correction
description: Correct only the PR lifecycle gate preventing Task 8 closeout; merge and clean Task 8, proceed to Task 9, and stop before release.
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1037-task8-pr-lifecycle-gate
terminalArchive: .ai/work-items/archive/WI-1037-task8-pr-lifecycle-gate.contract.json
terminalVerification: .ai/evidence/WI-1037-task8-pr-lifecycle-gate.verification.json
terminalDecision: .ai/decisions/WI-1037-task8-pr-lifecycle-gate.close.json
---

[简体中文](WI-1037-task8-pr-lifecycle-gate.zh-CN.md) · [日本語](WI-1037-task8-pr-lifecycle-gate.ja.md)

# WI-1037 — Task 8 PR lifecycle gate correction

This bounded successor addresses the pre-merge CI lifecycle failure recorded on [PR #997](https://github.com/xinglun/ai-cockpit/pull/997), [run 36377627838](https://github.com/xinglun/ai-cockpit/actions/runs/36377627838): an ordinary archived Work Item cannot be formally closed until its PR is merged, so CI must recognize only the exact PR merge checkout that introduced the archive.

## Boundaries

- Admit only an exact `pull_request` merge ref whose event base/head match the two Git parents and whose diff introduces the archived Contract.
- Keep older or mismatched archives, malformed events, direct pushes, and later unclosed commits blocked.
- Preserve the existing post-merge close requirement. Do not create a close receipt before integration.
- No broad governance redesign, Task 9 migration, release, version change, tag, or publication is in scope.

## Acceptance

1. The exact PR-introduced archive may be `awaiting_merge_close`; event and Git identities must match.
2. Older unclosed archives and mismatched or malformed PR events remain blocking.
3. Only the immediate exact default-branch merge transition is temporary; a later non-merge commit without close is blocking.
4. The three WI-1036 parity rows identify its archived Contract and state archived-but-awaiting-merge without claiming completion.
5. Focused checks and exact-head hosted CI pass before integration. Merge and clean Task 8, then start Task 9 only when Runtime reports `readyOnBase`; stop before release for human review.

Current state is in progress. Required scenarios, local checks, hosted checks, and integration remain pending; this page does not claim they passed.
