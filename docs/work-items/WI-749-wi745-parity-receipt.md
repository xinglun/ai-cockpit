---
author: AI Cockpit maintainers
title: "WI-749 — WI-745 recovery parity receipt"
description: "Register the current immutable recovery decision in the tri-language parity projections."
audience: [maintainer, reviewer, adopter]
workItemId: WI-749-wi745-parity-receipt
status: in-progress
authority: human:repository-owner
lastVerifiedBy: WI-749-wi745-parity-receipt
---

# WI-749 — WI-745 recovery parity receipt

This docs-only Work Item updates the three reference-parity projections so
the recovered WI-745 row points to the current Runtime-selected recovery
receipt. Historical WI-745 and WI-746 records remain unchanged.

[简体中文](WI-749-wi745-parity-receipt.zh-CN.md) · [日本語](WI-749-wi745-parity-receipt.ja.md)

## Scope

- Update only the WI-745 row in the three reference-parity documents.
- Preserve the `Recovered` status and the existing verification binding.
- Do not change Runtime behavior, tests, historical receipts, or the separate
  sections IV and V track.
