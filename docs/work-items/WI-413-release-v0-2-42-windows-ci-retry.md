---
author: AI Cockpit maintainers
title: "WI-413 — v0.2.42 release recovery after Windows CI"
workItemId: WI-413-release-v0-2-42-windows-ci-retry
description: "Redeliver the v0.2.42 candidate after the immutable WI-412 delivery was rejected by a Windows reuse-timing check."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-413-release-v0-2-42-windows-ci-retry
terminalArchive: .ai/work-items/archive/WI-413-release-v0-2-42-windows-ci-retry.contract.json
terminalVerification: .ai/evidence/WI-413-release-v0-2-42-windows-ci-retry.verification.json
terminalFinalization: .ai/decisions/WI-413-release-v0-2-42-windows-ci-retry.finalize.json
terminalDecision: .ai/decisions/WI-413-release-v0-2-42-windows-ci-retry.close.json
---

# WI-413 — v0.2.42 release recovery after Windows CI

This is a bounded recovery successor to WI-412. The predecessor archive,
verification, recovery decision, and failed PR remain immutable. The successor
fixes only the platform-dependent `execution_elapsed_ms` projection for a plan
satisfied entirely by reusable receipts, then repeats the complete release
verification and reviewed-delivery lifecycle.

## Scope

- Make zero-node execution report exactly zero elapsed execution time without
  changing receipt identity, reuse authorization, or fail-closed behavior.
- Retain the inherited v0.2.42 version/release/documentation candidate and
  keep all three language projections synchronized.
- Verify the full workspace and hosted quality, Windows-runtime, and
  reference-oracle checks before merge; adopter acceptance remains post-release.

## Recovery boundary

WI-412 and PR #377 were rejected by hosted Windows CI and are preserved as
historical recovery evidence. This successor is the only active delivery path;
it does not rewrite predecessor bytes or broaden the reference-parity batch.

## Verification

The installed Runtime is used with an explicit repository path. Required checks
include the locked workspace tests, formatting, warning-denied Clippy, release
static gates, governance integrity, documentation consistency, and hosted
quality/Windows/reference-oracle checks.

[简体中文](WI-413-release-v0-2-42-windows-ci-retry.zh-CN.md) · [日本語](WI-413-release-v0-2-42-windows-ci-retry.ja.md)
