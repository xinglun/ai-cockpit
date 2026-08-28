---
author: AI Cockpit maintainers
title: "WI-348 — Verification, operation-time policy, and provider-bound reference batch"
workItemId: WI-348-reference-verification-operation-policy
description: "Compare the next ten pinned reference paths and close bounded Rust-native verification/policy gaps."
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
capabilityClaims: [reference_parity, operation_time_policy_evaluation]
---

# WI-348 — Verification, operation-time policy, and provider-bound reference batch

[简体中文](WI-348-reference-verification-operation-policy.zh-CN.md) · [日本語](WI-348-reference-verification-operation-policy.ja.md)

## Intent and boundary

Compare the next ten paths at pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` one by one. Preserve useful
verification, multilingual, performance, and operation-time governance
semantics in Rust without copying source Python/Make runtime files, generated
assessment bytes, provider-global configuration, or historical provider truth.

The shared external Runtime remains request-scoped. Every adopter and object
project uses explicit `--repo`; Contracts, evidence, performance facts, and
decisions stay repository-local.

## File-level decisions

| Reference path | Classification | Decision |
| --- | --- | --- |
| `docs/reference/japanese-capability-assessment.md` | implemented-different-by-design | Map the source matrix to bounded tri-language reader/Outcome/adversarial/installation/documentation checks; do not claim general fluency. |
| `docs/reference/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | Document proportional routes, content-bound reuse, partial dependency handling, monotonic escalation, and visible advisory boundaries. |
| `docs/reference/multilingual-semantic-parity.md` | implemented-different-by-design | Keep Runtime-owned presentation facts equivalent across three languages; preserve Contract values in the authoring language. |
| `docs/reference/open-pr-issue-reconciliation-662.json` | reference-only | Historical source/provider inventory; never current GitHub or release truth. |
| `docs/reference/open-pr-issue-reconciliation-662.md` | reference-only | Historical reconciliation narrative; never current authorization. |
| `docs/reference/operation-time-policy-reevaluation.{ja,md,zh-CN}.md` | implemented-different-by-design | Add the Rust Core `OperationTimeRequest` evaluator with strict fail-closed bindings; it evaluates only and never executes or grants provider permission. |
| `docs/reference/performance-diagnosis.md` | implemented-different-by-design | Map source diagnosis to request-scoped Rust `diagnose` and advisory cost observations; do not invent provider wait/P95/assurance. |
| `docs/reference/pre-release-documentation-alignment.json` | reference-only | Historical generated assessment receipt; target documentation uses its own checks and evidence. |

## Verification

- Inventory contains exactly ten WI-348 records: seven
  `implemented-different-by-design` and three `reference-only`, with no
  deferred or migrate-gap rows.
- `OperationTimeRequest` rejects unsupported schema, unknown operation,
  operation/target/scope mismatch, missing scope/authority, stale evidence,
  untrusted input, and unclassified impact; it never performs the operation.
- English, Simplified Chinese, and Japanese documentation links are complete;
  fixed presentation labels localize without changing Contract bytes.
- The comparison ledger records the pinned target baseline and current counts;
  historical provider/pre-release records are not copied into `.ai/` or status.
- Rust tests, documentation/inventory checks, formatting, lint, and locked
  workspace verification pass; installed Runtime evidence reaches a visible
  human Outcome before reviewed merge.
