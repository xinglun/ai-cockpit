---
author: Ray
title: "WI-351 — Runtime retry recovery receipt binding"
workItemId: WI-351-runtime-recovery-binding
description: "Keep retry recovery receipts valid across Runtime-owned state projections while preserving fail-closed validation."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-351-runtime-recovery-binding
terminalArchive: .ai/work-items/archive/WI-351-runtime-recovery-binding.contract.json
terminalVerification: .ai/evidence/WI-351-runtime-recovery-binding.verification.json
capabilityClaims:
  - recovery_receipt_binding
---

# WI-351 — Runtime retry recovery receipt binding

[简体中文](WI-351-runtime-recovery-binding.zh-CN.md) · [日本語](WI-351-runtime-recovery-binding.ja.md)

## Intent and boundary

This Work Item repairs the retry recovery lifecycle in the shared Rust Runtime.
After a retry, the Runtime may update the current Summary, Outcome, and Events
projection. Those Runtime-owned bytes must not make the exact retry receipt look
foreign or stale, while foreign, stale, malformed, and misnamed evidence must
remain fail-closed.

The implementation is limited to the recovery binding logic and its regression
test. Sentinel business code, Provider discovery, trading decisions, gates,
execution, position sizing, global configuration, and historical archives are
outside the boundary.

## Verification

- `retry → verify → preflight → finish` is covered by a regression test that
  mutates the Runtime-owned post-retry projections.
- Existing recovery negative paths continue to reject invalid evidence.
- `cargo fmt --all -- --check`, locked workspace tests, and clippy pass locally;
  PR [#318](https://github.com/xinglun/ai-cockpit/pull/318) carries the hosted
  verification.

The Work Item is currently awaiting reviewed merge and provider finalization;
the immutable archive remains the source of lifecycle evidence.
