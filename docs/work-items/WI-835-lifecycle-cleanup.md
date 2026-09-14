---
author: AI Cockpit maintainers
title: "WI-835 — lifecycle cleanup disposition"
description: "Record an evidence-bound disposition for remaining release branches and worktrees."
audience: [maintainer, reviewer]
status: implemented
authority: authorized
workItemId: WI-835-lifecycle-cleanup
lastVerifiedBy: WI-835-lifecycle-cleanup
terminalArchive: .ai/work-items/archive/WI-835-lifecycle-cleanup.contract.json
terminalVerification: .ai/evidence/WI-835-lifecycle-cleanup.verification.json
terminalDecision: .ai/decisions/WI-835-lifecycle-cleanup.close.json
---

[简体中文](WI-835-lifecycle-cleanup.zh-CN.md) · [日本語](WI-835-lifecycle-cleanup.ja.md)

# WI-835 — lifecycle cleanup disposition

## Intent and boundary

WI-835 records an evidence-bound disposition for every current non-main
remote release branch and its local worktree. It preserves dirty, divergent,
unmerged, ambiguous, or evidence-insufficient resources and removes nothing
unless the exact reviewed merge, Runtime lifecycle, and clean resource facts
are all proven.

Runtime-generated lifecycle records and historical evidence remain immutable;
release tags, Releases, product behavior, and unrelated source changes are
outside this Work Item.

## Acceptance

- Every remaining remote branch has a recorded disposition with PR state,
  merge fact, worktree state, and blocker or cleanup evidence.
- Runtime-generated archive, finalization, and close records are used;
  historical evidence bytes are not rewritten or deleted.
- Only exact clean merged branch and worktree resources are removed after
  finalization verification.
- The cleanup Work Item passes reviewed PR integration, archive, close, and
  documentation promotion checks.

## Verification

- Runtime `verify` with the repository-bound disposition verifier.
- `git ls-remote --heads origin 'codex/*'` and local `git worktree list`.
- GitHub PR state and merge facts for every listed branch.
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`.
