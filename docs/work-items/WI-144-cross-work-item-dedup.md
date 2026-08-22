---
author: AI Cockpit maintainers
workItemId: WI-144-cross-work-item-dedup
title: Cross-Work-Item physical execution reuse
description: Separate shared physical execution from per-Work-Item authorization evidence.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-144-cross-work-item-dedup
---

# WI-144 — Cross-Work-Item physical execution reuse

This Work Item adds explicit `PhysicalExecution`, `ExecutionResult`, and
per-Work-Item `WorkItemEvidenceReceipt` boundaries. Physical execution may be
shared only when repository, snapshot, command, environment, Runtime, and
toolchain identities match. Authorization evidence remains Work Item-local.

Implementation evidence: `.ai/evidence/WI-144-cross-work-item-dedup.verification.json`.
Closure decision: `.ai/decisions/WI-144-cross-work-item-dedup.close.json`.
