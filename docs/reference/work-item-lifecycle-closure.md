---
author: AI Cockpit maintainers
title: Work Item lifecycle closure
description: Safely close a reviewed Work Item after archive, merge, and exact cleanup.
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Work Item lifecycle closure

[简体中文](work-item-lifecycle-closure.zh-CN.md) · [日本語](work-item-lifecycle-closure.ja.md)

Closure is the final handoff after `start → preflight → checkpoint → verify →
finish → archive`. It is not a branch-deletion shortcut. The reviewed PR, exact
Work Item head, archived Contract/Summary/evidence, synchronized base, clean
worktrees, and remote branch absence must all be proven by the Runtime.

## Normal route

```text
verify → finish/archive → push → reviewed PR and hosted checks → merge
→ finalize → finalize-verify → close → synchronize and clean
```

Run the repository-bound close command from the Work Item checkout or the
explicitly registered recovery checkout. It verifies PR state, branch and head
identity, base fast-forward synchronization, archive/decision receipts, clean
worktrees, and remote branch absence before deleting the exact local Work Item
branch. A provider must not auto-delete the branch to bypass this proof.

`ready_on_base` means the invoking checkout is clean and on the synchronized
default branch. `closed_but_current_worktree_detached` means closure succeeded
but another verified worktree owns the base; continue from that printed base
worktree instead of treating the detached checkout as ready.

## Recovery and historical boundaries

Any missing, stale, foreign, or contradictory fact fails closed and preserves
the retry identity. A provider anomaly or stacked-PR recovery uses a separate,
explicit receipt; it never rewrites an immutable archive or turns an open PR
into a merged one. Historical source `make` commands and Python orchestration
are not Runtime commands. The Rust route preserves the same intent—review,
archive, exact cleanup—with explicit `--repo` and repository-local evidence.
