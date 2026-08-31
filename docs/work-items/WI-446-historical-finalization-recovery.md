---
author: AI Cockpit maintainers
title: "WI-446 — Historical finalization recovery"
workItemId: WI-446-historical-finalization-recovery
description: "Honest, append-only recovery for legacy finalization records."
audience: [maintainer, adopter, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-446-historical-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-446-historical-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-446-historical-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-446-historical-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-446-historical-finalization-recovery.close.json
---

# WI-446: Historical finalization recovery

## Intent

Provide an honest, append-only compatibility path for legacy finalization
records produced before the dedicated linked-worktree and reviewed-PR workflow.
The Runtime must help an adopter finish historical work without rewriting a
predecessor receipt or inventing a pull request.

## Scope

- classify an older shared-primary-worktree `retained` receipt with a
  Runtime-bound `historical_finalization_recovery` record;
- validate repository, Work Item, Contract base, predecessor digest, Runtime,
  and human authority bindings;
- accept a complete no-PR direct-merge receipt only when its real merge commit,
  parents, base, and repository facts match Git;
- permit explicit low-assurance historical close while keeping new Work Items
  on the deleted-resource gate;
- expose the path through `work-item finalize-recovery` and document it in all
  supported languages.

## Non-goals

This does not rewrite historical bytes, fabricate PR numbers, weaken current
Runtime identity checks, or migrate the object repository automatically.
Historical assurance remains `historical_low`; it is not provider assurance.

## Acceptance

The repository tests cover shared-worktree recovery, foreign/tampered/symlink
rejection, direct merge verification against real Git parents, and close
without rewriting the predecessor. The command and workflow references state
the compatibility boundary and the exact human-authorized recovery command.
