---
author: AI Cockpit maintainers
title: "WI-1084 — cross-checkout recovery receipt integrity"
description: "Plan for detecting loss of a required recovery receipt while preserving historical close bytes."
audience:
  - maintainer
  - reviewer
workItemId: WI-1084-cross-checkout-closeout-receipt-integrity
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1084-cross-checkout-closeout-receipt-integrity
terminalArchive: .ai/work-items/archive/WI-1084-cross-checkout-closeout-receipt-integrity.contract.json
terminalVerification: .ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity.verification.json
terminalDecision: .ai/decisions/WI-1084-cross-checkout-closeout-receipt-integrity.close.json
contractDigest: sha256:507bedef85c7d3cb7ff2b8fd3a885d5441384ea23b8d6e4b010fa9be8c150107
---

[简体中文](WI-1084-cross-checkout-closeout-receipt-integrity.zh-CN.md) · [日本語](WI-1084-cross-checkout-closeout-receipt-integrity.ja.md)

# WI-1084 — cross-checkout recovery receipt integrity

Ray directly confirmed the bound Contract/preflight/snapshot and Native recorded confirm_review. Both receipt-only deletion and recovered Retained-close→Deleted-transition REDs are preserved. The bounded correction passes18 shared unit and14 repository/CLI/MCP integration tests; strict workspace clippy and format checks pass. Current formal evidence and remaining acceptance are determined by Native. lastVerifiedBy identifies this in-progress WI projection, not a verified close.

## Intended benefit and observed problem

The independent-checkout RED reproduced the faulty closed projection; its GREEN and the subsequent P2 combination matrix are retained. The original Sentinel acceptance history remains unchanged. A normal next Deleted transition keeps the immutable historical close valid after full existing transition checks.

## Bounded design

The [final plan](../../.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/final-plan-20261015/implementation-plan.md) fixes the planned wire shape: an immutable same-WI origin file (schema1) binds repository/WI, canonical Contract, source snapshot, provider/Runtime facts, exact original close and the complete raw payload list. It declares the required recovery receipt's canonical path and version2. The version2 receipt points one way to the origin's actual digest. No content-hash cycle is introduced. Missing or mismatched required records cannot fall back to ordinary close or downgrade to v1.

Install historical payload, origin, receipt and then the exact original close marker under the existing lifecycle lock/immutable installer. Recheck all required bytes before the marker; preserve published paths on interruption and reject conflicting retries. A recovered checkout may become a later source only after shared validation. Origin has no changing timestamp; receipt creation metadata remains honest and does not create a TTL.

Ordinary current/legacy closes retain their semantics. Existing v1 evidence is not rewritten or silently migrated. If the sole old receipt disappeared before independent provenance existed, that history is not inferable; removing every provenance record is not a general detection guarantee.

## RED and acceptance plan

1. Recover in independent Git common directories; establish the valid closed baseline, then remove only the receipt. Require `archived`, `blocking=true` and an integrity diagnostic while allowing query exit0. Observe the assertion fail before production changes.
2. Prove overlapping work is not falsely admitted. Cover malformed/missing/non-regular origin or receipt, wrong version/path/digest/identity, close/payload mismatch and archive completeness.
3. Use private hooks immediately after origin, after receipt, before close and after close. Preserve source and published paths, prove supported restoration/idempotence and use a recovered checkout as another source.
4. Preserve ordinary/v1 compatibility and historical `not_ready` / `historical_evidence_not_revalidated`; add narrow CLI/MCP parity coverage.
5. After real policyReview and fresh admission, run focused RED/GREEN recovery regressions, `cargo test --locked --workspace` and required repository gates. Keep the900-second bound, an isolated target, one build task and actual raw evidence. Current formal verification remains governed by the Native receipt; no terminal acceptance is claimed.

## Execution boundary

Single serial WI-1084 worker; final parallel compatibility is false because WI-1070–1074 lack boundaries/declarations. Their records remain unchanged. Build output is isolated at `target/wi1084-closeout-receipt-integrity/`; checkpointCount remains1. Keep PR#1040 Draft and integration serialized. No Sentinel business change, WI-1078 resource change, Task9, TTL, mechanism refactor, release, new-WI human close, merge or deletion is authorized.

Continue current Native verification and Raydot review using the [preserved RED/GREEN evidence](../../.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/p2-transition-20261015/red-summary.md). The unchanged Contract needs no repeated human review. Keep PR#1040 Draft; merge, close and resource deletion remain outside this approval.
