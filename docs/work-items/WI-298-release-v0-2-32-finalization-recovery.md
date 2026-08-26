---
author: AI Cockpit maintainers
title: "WI-298 — v0.2.32 release finalization recovery"
workItemId: WI-298-release-v0-2-32-finalization-recovery
description: "Complete the missing reviewed resource-finalization chain for WI-297 without rewriting its immutable archive."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-298-release-v0-2-32-finalization-recovery
authority: canonical
---

# WI-298 — v0.2.32 release finalization recovery

## Intent

Recover the missing `finalize-plan` boundary discovered after WI-297 was
archived. The predecessor archive, verification, recovery receipts, and merged
PR remain immutable; this Work Item records only the narrow closure recovery.

## Scope

- Bind the exact merged PR #258, branch, worktree, and default branch context.
- Run the installed Runtime verification and hosted quality checks for this
  recovery record.
- Record provider finalization, verify exact cleanup, and create a structured
  human close receipt.
- Keep the predecessor/successor relationship and all evidence identity-bound.

## Out of scope

Release implementation, Runtime behavior, package metadata, adopter acceptance,
Homebrew publication, and historical archive rewriting are outside this
recovery.

## Acceptance

- WI-297 archive bytes remain unchanged and are referenced by the recovery
  decision.
- `finalize-plan` is recorded before successor verification and archive.
- Hosted checks pass for the reviewed successor PR.
- `finalize-verify` proves the exact feature branch/worktree cleanup before
  structured close.
- A visible human Outcome contains status, unknowns, evidence, decision, and
  next action.

## Verification

Use the installed Runtime with an explicit `--repo`, repository governance and
documentation gates, hosted quality checks, and the complete
`finalize-plan → finalize → finalize-verify → close` chain.

The hosted quality result for the reviewed PR is part of the terminal evidence;
an earlier pre-archive run that lacked verification evidence remains a failed
historical attempt and is not reused.
