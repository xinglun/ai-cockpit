---
author: AI Cockpit maintainers
title: "WI-187 — resource finalization before archive"
description: "Require an explicit, non-provisional resource finalization plan before a current Work Item can be archived."
audience:
  - maintainer
  - reviewer
workItemId: WI-187-finalization-before-archive
status: implemented
authority: canonical
lastVerifiedBy: WI-187-finalization-before-archive
---

# WI-187 — resource finalization before archive

WI-187 closes a lifecycle ordering gap. `start` intentionally records a
provisional `resourceContext`: local branch and worktree observations are
available, while `baseBranch`, `baseRemote`, `provider`, and `pullRequest`
remain `unknown`. That context is not a resource finalization plan.

The standard `finish` boundary now rejects a missing or provisional context
before it can create a `finish_ready` state, and `archive` independently
rechecks the same condition before moving any active Contract, Summary,
Outcome, report, event, or approach bytes. The operator must run `work-item
finalize-plan` with a complete, validated, identity-bound context before
verification, finish, and archive. A valid non-provisional plan preserves the
existing successful lifecycle flow.

## Historical and recovery boundary

WI-186 is the observed predecessor: the public v0.2.23 Runtime archived its
records while its Contract still contained the provisional context written by
`start`. WI-187 does not edit, normalize, or retroactively promote those
historical archive bytes. Historical readers continue to accept the optional
context shape, and the explicit supersession recovery route continues to copy
the predecessor artifacts byte-for-byte. That recovery exception requires its
own identity-bound recovery decision; it does not let a current ordinary Work
Item bypass `finalize-plan`.

WI-187 is the bounded successor for that observed gap. The installed Runtime
records a `supersede` recovery receipt under `.ai/decisions/` that binds the
exact WI-186 Contract, Summary, Outcome, and events digests to WI-187. The
receipt is additive: it neither creates a new interpretation of the WI-186
result nor rewrites any file in the WI-186 archive bundle.

The first WI-187 execution itself reached `finish_ready` before this ordering
was enforced. Runtime therefore refused to replace its provisional plan after
verification. Those exact records are preserved through a digest-bound
supersession, while `WI-190-finalization-plan-order` repeats the lifecycle in
the required order and carries the verified implementation forward.

The regression suite covers protocol-level provisional detection, repository
archive refusal and byte preservation, CLI refusal and recovery state, success
after a valid plan, historical evidence readability, and immutable superseded
predecessor recovery. Shared reference parity files are intentionally outside
this Work Item.

[简体中文](WI-187-finalization-before-archive.zh-CN.md) ·
[日本語](WI-187-finalization-before-archive.ja.md)
