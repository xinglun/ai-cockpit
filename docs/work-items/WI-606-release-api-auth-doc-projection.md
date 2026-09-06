---
title: "WI-606 — WI-605 terminal documentation projection"
description: "Promote the tri-language parity records required by the hosted documentation gate."
author: AI Cockpit maintainers
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-606-release-api-auth-doc-projection
workItemId: WI-606-release-api-auth-doc-projection
predecessorWorkItemId: WI-605-release-api-auth
---

[简体中文](WI-606-release-api-auth-doc-projection.zh-CN.md) · [日本語](WI-606-release-api-auth-doc-projection.ja.md)

# WI-606 — WI-605 terminal documentation projection

## Objective

Keep the reference-parity index and Work Item documentation aligned with the
published release-acceptance fix. This is a bounded documentation successor;
it does not change Runtime behavior or object repositories.

## Boundary

The scope is the three language parity indexes and the three language WI-605
documentation records. The predecessor archive, evidence, and recovery bytes
remain immutable. Runtime, release harness, and installer behavior are out of
scope.

## Verification

Run `bash tests/docs/parity_status_check.sh .` and the declared workspace
verification. The reviewed PR and its hosted checks are required before close.
