---
author: AI Cockpit maintainers
title: "WI-315 — post-close reconciliation promotion recovery"
workItemId: WI-315-post-close-reconciliation-promotion-recovery
description: "Correct recovered-predecessor promotion semantics without rewriting immutable W314 history."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-315-post-close-reconciliation-promotion-recovery
---

# WI-315 — post-close reconciliation promotion recovery

## Intent and boundary

W314 is an immutable failed hosted delivery. Its documentation gate exposed a
projection defect: a valid successor recovery was ignored when the predecessor
also contained a confirmed close. This successor corrects that narrow gate
condition from the latest default branch and does not rewrite W314 history.

## Scope and acceptance

- A valid repository-bound `successor` or `supersede` recovery makes its
  predecessor historical, regardless of the predecessor close projection.
- Retry, malformed, foreign, and non-canonical recovery receipts continue
  through normal promotion validation and fail closed when required evidence is
  invalid.
- Regression coverage includes a confirmed approved close plus a valid
  successor recovery, as well as invalid recovery variants.
- Tri-language documentation and parity register W315 before verification and
  preserve the W314 failure and recovery boundary.

## Verification

Run the focused documentation regression, documentation acceptance, `cargo fmt`,
warning-denied clippy, and the locked single-process workspace test suite.
Hosted CI must pass on the exact reviewed branch before merge. The installed
Runtime remains the governance interface.

## Related history

- W314: immutable predecessor whose hosted quality gate identified this defect.
- W315: bounded successor that corrects only the promotion projection.

[简体中文](WI-315-post-close-reconciliation-promotion-recovery.zh-CN.md) ·
[日本語](WI-315-post-close-reconciliation-promotion-recovery.ja.md)
