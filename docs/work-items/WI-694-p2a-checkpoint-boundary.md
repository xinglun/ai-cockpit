---
author: AI Cockpit maintainers
title: "WI-694 — P2-A checkpoint responsibility boundary"
description: "Separate checkpoint observation, governance validation, and persistence within one complete lifecycle use case."
audience: [maintainer, reviewer]
workItemId: WI-694-p2a-checkpoint-boundary
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-694-p2a-checkpoint-boundary
terminalArchive: .ai/work-items/archive/WI-694-p2a-checkpoint-boundary.contract.json
terminalVerification: .ai/evidence/WI-694-p2a-checkpoint-boundary.verification.json
terminalFinalization: .ai/decisions/WI-694-p2a-checkpoint-boundary.finalize.json
terminalDecision: .ai/decisions/WI-694-p2a-checkpoint-boundary.close.json
---

[简体中文](WI-694-p2a-checkpoint-boundary.zh-CN.md) · [日本語](WI-694-p2a-checkpoint-boundary.ja.md)

# WI-694 — P2-A checkpoint responsibility boundary

## Boundary

This successor starts from the latest remote `main` after the earlier WI-655
branch became stale. It changes only `checkpoint_work_item` in the repository
lifecycle module and its focused lifecycle verification. The public function,
serialized Summary/checkpoint evidence, error precedence, authorization
semantics, and file layout remain unchanged.

## Responsibility split

- `checkpoint_observe` reads the active Contract, captures one Git snapshot,
  and derives the Contract and repository snapshot digests.
- `checkpoint_governance_checks` validates the current preflight bindings,
  reuses the captured snapshot for the existing governance authority, and
  performs no persistence.
- `checkpoint_work_item` retains lifecycle ordering and performs checkpoint
  evidence and Summary writes only after the preceding checks pass.

This is a narrow use-case extraction. It does not introduce a Port or trait,
duplicate governance rules, extend snapshot validity across an edit boundary,
or change finish/archive/close semantics.

## Verification

Focused lifecycle entry, order, and concurrency tests cover the unchanged
fail-closed behavior. Full workspace format, Clippy, tests, Runtime evidence,
hosted checks, and terminal lifecycle records are required before closure.

## Remaining risk

The governance decision remains the existing authority and may still read its
repository-bound declarations; this Work Item only prevents the checkpoint
use-case body from recapturing its own Contract/snapshot/digests. Broader
observation-context threading is a separate P1-B boundary.
