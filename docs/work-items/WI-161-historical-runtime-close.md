---
author: AI Cockpit maintainers
title: "WI-161 — Historical Runtime evidence close compatibility"
description: "Keep archived evidence immutable while allowing close after a Runtime upgrade."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-161-historical-runtime-close
workItemId: WI-161-historical-runtime-close
---

# WI-161 — Historical Runtime evidence close compatibility

## Intent

Runtime upgrades must not make an already archived Work Item impossible to
close. Active Work Items remain strictly bound to the Runtime that executes
verification; archived evidence is immutable historical truth and may be
projected as historical after an upgrade.

## Boundary

When closing an archived Work Item, the Runtime first validates the archived
verification evidence without applying the current Runtime identity. If those
bytes are otherwise valid, a different Runtime is an explicit historical
compatibility case, not a current verification failure. Resource finalization
is still request-scoped and must be bound to the Runtime executing `close`.

This does not rewrite evidence, turn historical evidence green, or weaken the
active `finish`/`archive` gates.

The Runtime command and receipt boundary introduced by WI-159 remains in
force; this Work Item only defines the historical compatibility lane.

## Acceptance

1. Active lifecycle rejects foreign Runtime verification evidence.
2. Archived foreign-Runtime evidence is projected as historical and remains
   digest/identity/archive-manifest bound.
3. Close after a Runtime upgrade succeeds only when current resource
   finalization requirements are satisfied.
4. English, Simplified Chinese, and Japanese workflow/parity docs describe
   the same Runtime-versus-historical evidence boundary.

## Verification

Evidence: `.ai/evidence/WI-161-historical-runtime-close.verification.json`.
Archive: `.ai/work-items/archive/WI-161-historical-runtime-close.archive.json`.
