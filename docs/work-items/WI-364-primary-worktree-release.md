---
author: AI Cockpit maintainers
title: "WI-364 — primary worktree release recovery"
workItemId: WI-364-primary-worktree-release
description: "Prevent ordinary Work Items from binding the repository primary worktree and redeliver v0.2.37 from a dedicated checkout."
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-364-primary-worktree-release
capabilityClaims: [lifecycle_entry, release_distribution]
---

# WI-364 — primary worktree release recovery

[简体中文](WI-364-primary-worktree-release.zh-CN.md) · [日本語](WI-364-primary-worktree-release.ja.md)

## Intent

Root-fix the release delivery boundary exposed by WI-363: an ordinary Work
Item must not bind the repository primary worktree or default branch. Redeliver
the v0.2.37 release from a dedicated Work Item worktree and preserve the
predecessor's immutable recovery evidence.

## Scope and boundary

- Reject ordinary `start` and `work-item new` before writing a Contract when the
  current checkout is the Git primary worktree or known default branch.
- Reject a linked worktree when the remote default base is missing or ambiguous;
  retain `status: unknown` for unbound local calibration repositories.
- Add focused CLI regressions for primary, default, dedicated, and ambiguous
  topology cases.
- Document the topology requirement and the WI-363 recovery boundary in the
  canonical tri-language workflow, command, and parity references.
- Complete the immutable public v0.2.37 artifact, adopter, N-1, finalization,
  close, and exact cleanup acceptance from this dedicated worktree.

Changing WI-363 archive/evidence/decision bytes, release artifact semantics,
global Agent/MCP configuration, or unrelated Runtime behavior is outside this
Work Item.

## Acceptance

1. Ordinary `start` and `work-item new` fail closed on the primary worktree and
   default branch, while a dedicated linked worktree is accepted.
2. Missing or ambiguous remote default metadata cannot authorize a linked
   worktree; no false-green Contract is written.
3. Focused regressions cover all topology cases and leave no Work Item files on
   rejected entry.
4. Tri-language workflow, command, and parity documentation states the rule and
   links the predecessor recovery boundary.
5. The public v0.2.37 artifact is downloaded and checksum-bound without source
   or workspace fallback; adopter and N-1 receipts prove isolation and cleanup.
6. Reviewed merge, finalization, close, and exact branch/worktree cleanup leave
   synchronized `main` ready on base.

## Verification boundary

The installed Runtime records Contract amendment, preflight, checkpoint,
verification, finish, archive, finalization, and close evidence. Hosted CI and
public-artifact acceptance are authoritative for release claims. WI-363's
archive and recovery bytes remain historical and are never rewritten.
