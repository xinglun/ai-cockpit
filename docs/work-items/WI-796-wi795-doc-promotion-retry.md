---
author: AI Cockpit maintainers
title: "WI-796 — WI-795 ordered documentation redelivery successor"
description: "Complete the documentation promotion after preserving WI-795's immutable ordering failure."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized-successor-recovery
workItemId: WI-796-wi795-doc-promotion-retry
lastVerifiedBy: WI-796-wi795-doc-promotion-retry
terminalArchive: .ai/work-items/archive/WI-796-wi795-doc-promotion-retry.contract.json
terminalVerification: .ai/evidence/WI-796-wi795-doc-promotion-retry.verification.json
terminalFinalization: .ai/decisions/WI-796-wi795-doc-promotion-retry.finalize.e98e9a07ee6e979dade4b7884f937e1ceb9e298d41c937e59a544ef5d3a86c43.json
terminalDecision: .ai/decisions/WI-796-wi795-doc-promotion-retry.close.json
predecessorWorkItemId: WI-795-wi794-doc-promotion
recoveryDecision: .ai/decisions/WI-795-wi794-doc-promotion.recovery.json
---

[简体中文](WI-796-wi795-doc-promotion-retry.zh-CN.md) · [日本語](WI-796-wi795-doc-promotion-retry.ja.md)

# WI-796 — WI-795 ordered documentation redelivery successor

## Recovery boundary

WI-796 is the bounded successor of WI-795. WI-795 remains immutable evidence:
its parity registration and verification evidence were introduced in the same
commit, so the required registration-before-evidence order cannot be repaired
without rewriting history. The Runtime recovery decision preserves that fact;
this successor supplies a fresh ordered documentation boundary.

WI-796 changes only documentation projections and their Runtime governance
records. It does not change Rust behavior, Runtime behavior, release assets,
authorization semantics, or any predecessor archive, evidence, finalization, or
recovery bytes.

## Acceptance

- The WI-794 terminal projection remains bound to its immutable archive,
  verification, finalization, and close records.
- WI-796's three language pages and parity row are registered before fresh
  verification evidence is introduced.
- English, Simplified Chinese, and Japanese preserve the same governance facts,
  unknowns, recovery boundary, and operational consequences.
- Documentation, parity, status-consistency, and governance-integrity checks
  pass without rewriting predecessor records.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-794-release-v0-2-90-closure --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
