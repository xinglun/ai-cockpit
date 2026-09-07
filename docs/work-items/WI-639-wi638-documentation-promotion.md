---
author: AI Cockpit maintainers
title: WI-639 - WI-638 documentation promotion
description: Promote the verified terminal documentation projection for WI-638 without changing governance facts.
audience: [maintainer, reviewer, adopter]
workItemId: WI-639-wi638-documentation-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-639-wi638-documentation-promotion
---

# WI-639 - WI-638 documentation promotion

This Work Item applies the deterministic documentation projection produced by
`tests/docs/promote_closed_work_item.py` to the three language pages and the
three reference-parity rows for WI-638. It changes reader-facing projections
only; immutable Contract, verification, archive, finalization, and close
records remain the authority and are not rewritten.

The projection is repository-local and language-linked. It does not copy the
reference implementation or alter object/adopter repositories.

## Acceptance

- The helper promotes all three WI-638 Work Item pages.
- The helper promotes all three WI-638 parity rows.
- Immutable lifecycle records remain byte-for-byte unchanged.
- Documentation, parity, and post-close promotion checks pass.

参见：[中文](WI-639-wi638-documentation-promotion.zh-CN.md) · [日本語](WI-639-wi638-documentation-promotion.ja.md)。
