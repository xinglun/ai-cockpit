---
author: AI Cockpit maintainers
title: "WI-357 — Release adopter finalization recovery"
workItemId: WI-357-release-adopter-finalization-recovery
description: "Rebind the reviewed WI-356 delivery to its actual provider context without rewriting immutable predecessor evidence."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-357-release-adopter-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-357-release-adopter-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-357-release-adopter-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-357-release-adopter-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-357-release-adopter-finalization-recovery.close.json
predecessor: WI-356-release-adopter-script-order
capabilityClaims:
  - adopter_finalization_recovery
---

# WI-357 — Release adopter finalization recovery

[简体中文](WI-357-release-adopter-finalization-recovery.zh-CN.md) · [日本語](WI-357-release-adopter-finalization-recovery.ja.md)

## Intent and boundary

WI-357 is an explicit recovery successor for WI-356. WI-356 was merged as PR
#321, but its immutable archive was created before that PR existed and therefore
retains a provisional `pending` resource context. This Work Item records the
actual provider binding and exact cleanup through a new, auditable lifecycle;
it never rewrites WI-356 archive, evidence, or outcome bytes.

The scope is limited to the recovery decision, tri-lingual governance records,
the reviewed PR resource binding, and finalization/close evidence. Runtime
feature changes, adopter harness behavior, release version changes, and
provider automation are outside this boundary.

## Delivery and verification

- The recovery receipt binds the predecessor Contract, Summary, Outcome, and
  Events digests and explicitly links WI-356 and PR #321.
- A dedicated branch/worktree and reviewed PR bind the actual GitHub context
  before verification evidence is recorded.
- `finalize-verify` must prove the merged PR, synchronized default branch, and
  exact local/remote branch and worktree cleanup before structured close.
- All generated receipts remain repository-bound and are produced by the
  installed Runtime; historical predecessor bytes remain immutable.

## Recovery boundary

This successor exists because a truthful provider receipt cannot replace the
provisional resource context embedded in an immutable predecessor Contract.
The recovery path is therefore explicit, fail-closed, and separately
reviewable rather than using a fabricated URL or editing the predecessor.
