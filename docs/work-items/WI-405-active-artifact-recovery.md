---
author: AI Cockpit maintainers
title: WI-405 — Active artifact recovery
description: Reconcile failed Work Item artifacts without hiding residue or rewriting immutable history.
workItemId: WI-405-active-artifact-recovery
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-405-active-artifact-recovery
---

# WI-405 — Active artifact recovery

## Intent

Keep failed or interrupted Work Item artifacts auditable while preventing
stale files in `active/` from being mistaken for active governance state.

## Scope

- Detect and reconcile recognized failed-attempt outcome and event variants.
- Preserve their bytes and digests in the archive manifest.
- Report orphaned active artifacts separately from valid active Contracts.
- Keep repositories and Runtime evidence isolated.

## Evidence

- Archive Contract: `.ai/work-items/archive/WI-405-active-artifact-recovery.contract.json`
- Verification: `.ai/evidence/WI-405-active-artifact-recovery.verification.json`
- Installed Runtime: v0.2.40

## Boundary

This Work Item does not rewrite or delete historical evidence, change release
automation, or alter the meaning of existing Work Item decisions.
