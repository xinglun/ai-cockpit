---
author: AI Cockpit maintainers
title: "WI-313 — post-close finalization reconciliation"
workItemId: WI-313-post-close-finalization-reconciliation
description: "Enforce cleanup-before-close and provide a narrowly bound recovery path for immutable legacy close records."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-313-post-close-finalization-reconciliation
---

# WI-313 — post-close finalization reconciliation (recovered history)

## Intent and boundary

W312 exposed a real ordering defect: an older Runtime could close a Work Item
while its provider finalization remained `retained`, then the closed-document
promotion gate correctly refused to claim terminal cleanup. WI-313 was the
first bounded correction, but its PR #277 failed hosted quality before merge.
This document therefore describes immutable failed-delivery history, not a
merged implementation. WI-321 records the explicit successor-owned recovery;
no WI-313 bytes are rewritten. New Work Items must clean provider resources
before close; only an immutable legacy close may receive one bound deleted
transition afterward.

## Scope and acceptance

The original Rust protocol/repository lifecycle correction and its hosted
delivery attempt remain unchanged as historical evidence. The current gate
projects this Work Item as `Recovered` only through the Runtime-generated
successor receipt bound to WI-321. The original Contract, Summary, Outcome,
Events, archive, verification, retry receipt, branch, and PR bytes remain
unchanged. The documentation promotion gate and tri-language workflow reject
an orphaned retry and require an explicit successor or a valid terminal path.

## Verification

The original targeted Rust finalization tests and hosted PR evidence are
historical. WI-321 adds the orphaned-retry static regression and verifies the
three-language recovery projection, documentation gates, and installed
Runtime-generated successor receipt. No source-build fallback is a release
acceptance substitute.

[WI-321 successor recovery](WI-321-explicit-failed-delivery.md)

[简体中文](WI-313-post-close-finalization-reconciliation.zh-CN.md) ·
[日本語](WI-313-post-close-finalization-reconciliation.ja.md)
