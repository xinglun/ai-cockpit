---
workItemId: WI-527-direct-merge-context
title: "WI-527 — direct-merge recovery context compatibility"
status: implemented
mode: code
author: AI Cockpit maintainers
description: "Bounded compatibility for historical direct-merge receipts that preserve an archived local resource context."
audience:
  - maintainer
  - adopter
authority: canonical
lastVerifiedBy: WI-527-direct-merge-context
terminalArchive: .ai/work-items/archive/WI-527-direct-merge-context.contract.json
terminalVerification: .ai/evidence/WI-527-direct-merge-context.verification.json
terminalFinalization: .ai/decisions/WI-527-direct-merge-context.finalize.f7bc389eb8064f2451fb5cbd0bb28785546030040c999d25e65f6e0adb5a7c85.json
terminalDecision: .ai/decisions/WI-527-direct-merge-context.close.json
---

# WI-527 — direct-merge recovery context compatibility

## Intent and boundary

Make the historical no-PR recovery path usable for repositories whose
archived Contract still contains its original local `resourceContext`.
The Runtime may accept that context only for an explicitly typed
`direct_merge_no_pr` / `historical_low` receipt and must continue to bind the
repository, Work Item, branch, worktree, base, real merge commit and parents.
No PR number is invented and no object repository is modified.

## Implementation

- The protocol accepts the unchanged archived local context in this narrow
  historical case and rejects foreign branch/worktree/base identities.
- `finalize-recovery-plan` emits an identity-consistent historical context so
  an Agent can use the suggestion without guessing provider or URL fields.
- Rust protocol and repository regressions cover both the unchanged-context
  and transformed-historical-context forms, plus the real Git parent binding.

## Acceptance

The public Runtime must accept a truthful first direct-merge record, remain
fail-closed for malformed, foreign, stale, symlinked, or non-ancestor inputs,
and leave immutable historical bytes untouched. Targeted and workspace tests,
documentation checks, and the normal lifecycle evidence are required.

## Object-repository handoff

`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` is read-only
for this Work Item. Its team reruns `finalize-recovery-plan` and applies only
the published suggested receipt after release.
