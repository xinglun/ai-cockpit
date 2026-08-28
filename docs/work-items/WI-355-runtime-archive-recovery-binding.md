---
author: Ray
title: "WI-355 — Runtime archive recovery binding"
workItemId: WI-355-runtime-archive-recovery-binding
description: "Consume legitimate stale retry receipts as historical evidence for archived Work Items while preserving fail-closed validation."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-355-runtime-archive-recovery-binding
terminalArchive: .ai/work-items/archive/WI-355-runtime-archive-recovery-binding.contract.json
terminalVerification: .ai/evidence/WI-355-runtime-archive-recovery-binding.verification.json
terminalFinalization: .ai/decisions/WI-355-runtime-archive-recovery-binding.finalize.json
terminalDecision: .ai/decisions/WI-355-runtime-archive-recovery-binding.close.json
predecessor: WI-353-runtime-recovery-delivery-binding
capabilityClaims:
  - archived_retry_recovery_binding
---

# WI-355 — Runtime archive recovery binding

[简体中文](WI-355-runtime-archive-recovery-binding.zh-CN.md) · [日本語](WI-355-runtime-archive-recovery-binding.ja.md)

## Intent and boundary

This successor Work Item repairs the archived read path for a legitimate stale
retry recovery receipt. Once a retry has completed and fresh archived
projections exist, the old retry receipt is consumed as historical evidence;
it must not block Outcome or close evaluation as if it were current recovery.

Malformed, foreign, misnamed, ambiguous, and still-pending retry evidence must
remain fail-closed. WI-353 archive bytes are immutable and remain outside the
implementation edit boundary.

## Verification and delivery boundary

- Add an archived stale-retry regression and preserve the existing negative
  recovery tests.
- Run formatting, locked workspace tests, clippy, governance integrity, and
  documentation acceptance.
- The reviewed PR is merged, provider finalization is verified, and structured
  close is recorded.
