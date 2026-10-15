---
author: AI Cockpit maintainers
title: "WI-1084 — cross-checkout recovery receipt integrity"
description: "Plan for detecting loss of a required recovery receipt while preserving historical close bytes."
audience:
  - maintainer
  - reviewer
workItemId: WI-1084-cross-checkout-closeout-receipt-integrity
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1084-cross-checkout-closeout-receipt-integrity
contractDigest: sha256:507bedef85c7d3cb7ff2b8fd3a885d5441384ea23b8d6e4b010fa9be8c150107
---

[简体中文](WI-1084-cross-checkout-closeout-receipt-integrity.zh-CN.md) · [日本語](WI-1084-cross-checkout-closeout-receipt-integrity.ja.md)

# WI-1084 — cross-checkout recovery receipt integrity

Ray directly confirmed the bound Contract/preflight/snapshot and Native recorded confirm_review. The receipt-only deletion RED failed on unchanged production source (actual closed, expected archived). The bounded implementation is in progress; focused repository regressions passed18/18. CLI/MCP, complete verification and scenario recording remain pending. lastVerifiedBy identifies this WI projection and does not claim formal verification.

## Intended benefit and observed problem

Developers should receive an archived, blocking state when the evidence required for a recovered close is lost. [Issue #1004's Mac report](https://github.com/xinglun/ai-cockpit/issues/1004#issuecomment-6186749658) used exact source `bcd76ae76f61650a890267f511099c96dc6a47fd` and reports that removing only the recovery receipt leaves `closed` and `blocking=false`. This environment read that report and traced the matching missing-file branch; it has not rerun the Sentinel acceptance or demonstrated the RED yet. `status` exit0 itself is valid behavior.

## Bounded design

The [final plan](../../.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/final-plan-20261015/implementation-plan.md) fixes the planned wire shape: an immutable same-WI origin file (schema1) binds repository/WI, canonical Contract, source snapshot, provider/Runtime facts, exact original close and the complete raw payload list. It declares the required recovery receipt's canonical path and version2. The version2 receipt points one way to the origin's actual digest. No content-hash cycle is introduced. Missing or mismatched required records cannot fall back to ordinary close or downgrade to v1.

Install historical payload, origin, receipt and then the exact original close marker under the existing lifecycle lock/immutable installer. Recheck all required bytes before the marker; preserve published paths on interruption and reject conflicting retries. A recovered checkout may become a later source only after shared validation. Origin has no changing timestamp; receipt creation metadata remains honest and does not create a TTL.

Ordinary current/legacy closes retain their semantics. Existing v1 evidence is not rewritten or silently migrated. If the sole old receipt disappeared before independent provenance existed, that history is not inferable; removing every provenance record is not a general detection guarantee.

## RED and acceptance plan

1. Recover in independent Git common directories; establish the valid closed baseline, then remove only the receipt. Require `archived`, `blocking=true` and an integrity diagnostic while allowing query exit0. Observe the assertion fail before production changes.
2. Prove overlapping work is not falsely admitted. Cover malformed/missing/non-regular origin or receipt, wrong version/path/digest/identity, close/payload mismatch and archive completeness.
3. Use private hooks immediately after origin, after receipt, before close and after close. Preserve source and published paths, prove supported restoration/idempotence and use a recovered checkout as another source.
4. Preserve ordinary/v1 compatibility and historical `not_ready` / `historical_evidence_not_revalidated`; add narrow CLI/MCP parity coverage.
5. After real policyReview and fresh admission, run focused RED/GREEN recovery regressions, `cargo test --locked --workspace` and required repository gates. Keep the900-second bound, an isolated target, one build task and actual raw evidence. No result is currently claimed.

## Execution boundary

Single serial WI-1084 worker; final parallel compatibility is false because WI-1070–1074 lack boundaries/declarations. Their records remain unchanged. Build output is isolated at `target/wi1084-closeout-receipt-integrity/`; checkpointCount remains1. Keep PR#1040 Draft and integration serialized. No Sentinel business change, WI-1078 resource change, Task9, TTL, mechanism refactor, release, new-WI human close, merge or deletion is authorized.

Next: CLI/MCP and preservation regressions, then current Native workspace verification. [Direct approval and raw RED](../../.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/implementation-20261015/progress.md) are retained. The unchanged Contract needs no repeated human review. Merge, close and deletion remain unauthorized.
