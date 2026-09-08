---
author: AI Cockpit maintainers
title: WI-681 — WI-677 lifecycle concurrency recovery redelivery
description: Redeliver the P2-C lifecycle concurrency and recovery boundary from the latest default base.
workItemId: WI-681-wi677-lifecycle-concurrency-recovery
audience:
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-681-wi677-lifecycle-concurrency-recovery
---

# WI-681 — WI-677 lifecycle concurrency recovery redelivery

WI-681 is the explicit successor for WI-677 after the archived PR became
conflicting when the remote default branch advanced. It preserves the
predecessor archive, evidence, Outcome, finalization, and recovery decision,
and redelivers the same P2-C boundary from the latest `origin/main`.

## Boundary

Serialize `finish`, `archive`, `close`, recovery-decision recording, and active
artifact reconciliation per Work Item with a stable OS advisory lock. Make
atomic temporary names unique within a process, and keep failure projections
from downgrading a committed terminal state. Preserve public JSON, lifecycle
semantics, archive layout, and Runtime compatibility.

## Verification

The focused concurrency tests cover same-process and cross-process finish
races, archive/close races, and fail-closed missing or corrupt projections.
The full locked workspace suite, formatting, Clippy, documentation acceptance,
governance integrity, and hosted checks remain required before merge.
