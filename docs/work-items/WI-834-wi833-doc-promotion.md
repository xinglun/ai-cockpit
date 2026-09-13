---
author: AI Cockpit maintainers
title: "WI-834 — WI-833 documentation promotion"
description: "Register the evidence-bound reader documentation for the closed WI-833 lifecycle."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-834-wi833-doc-promotion
lastVerifiedBy: WI-834-wi833-doc-promotion
---

[简体中文](WI-834-wi833-doc-promotion.zh-CN.md) · [日本語](WI-834-wi833-doc-promotion.ja.md)

# WI-834 — WI-833 documentation promotion

## Intent and boundary

WI-834 registers the reader-facing documentation and reference-parity rows
for the closed WI-833 release-script provenance Work Item. The projection
must point to the exact Runtime-owned archive, verification, finalization, and
close records without modifying those historical bytes.

Runtime behavior, release artifacts, historical Work Items, and unrelated
source verification are outside this Work Item.

## Acceptance

- The English, Simplified Chinese, and Japanese WI-834 pages are present and
  bind the same Work Item identity.
- Each parity ledger has exactly one pre-archive WI-834 row before verification.
- The WI-833 promotion check and repository-wide documentation checks pass.
- After close, promotion updates the same rows with terminal evidence without
  changing WI-833 history.

## Verification

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`

