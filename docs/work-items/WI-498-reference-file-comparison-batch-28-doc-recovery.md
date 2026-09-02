---
author: AI Cockpit maintainers
title: "WI-498 — batch 28 documentation recovery"
description: "Correct the recovered predecessor documentation projection after hosted CI identified a stale status."
audience: [maintainer, reviewer, adopter]
workItemId: WI-498-reference-file-comparison-batch-28-doc-recovery
predecessorWorkItemId: WI-497-reference-file-comparison-batch-28-retry
status: recovered
authority: human-authorized
lastVerifiedBy: WI-498-reference-file-comparison-batch-28-doc-recovery
successorWorkItemId: WI-499-reference-file-comparison-batch-28-parity-order-recovery
recoveryDecision: .ai/decisions/WI-498-reference-file-comparison-batch-28-doc-recovery.recovery.json
---

# WI-498 — batch 28 documentation recovery

[简体中文](WI-498-reference-file-comparison-batch-28-doc-recovery.zh-CN.md) · [日本語](WI-498-reference-file-comparison-batch-28-doc-recovery.ja.md)

## Boundary

WI-497 is preserved as immutable hosted-CI failure history. This successor
corrects only the tri-language documentation projection required by the
authoritative recovery and parity records. It does not rewrite predecessor
archive/evidence bytes, change Runtime policy, copy source implementation, or
operate an object repository.

## Acceptance

- WI-496 and WI-497 predecessor pages use the `recovered` status and link their
  Runtime recovery receipts.
- The ten batch-28 classifications and source-only boundaries remain unchanged.
- Documentation, parity, inventory, governance, and declared Runtime checks
  pass before reviewed merge and exact cleanup.

## Recovery boundary

WI-498 is retained as immutable recovery history. Its hosted checks passed before
archive, but its parity row and verification evidence were introduced in the same
commit, so the strict prearchive ordering proof was not available. WI-499 is the
explicit successor; it preserves these bytes and redelivers the projection with
the parity registration commit preceding verification evidence.
