---
author: AI Cockpit maintainers
title: "WI-479 — WI-478 terminal documentation promotion"
description: "Promote the closed WI-478 release record and register this documentation Work Item before its own close."
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-479-wi478-doc-promotion
workItemId: WI-479-wi478-doc-promotion
---

# WI-479 — WI-478 terminal documentation promotion

This documentation-only Work Item promotes the closed WI-478 release into
reader-facing projections and registers its own lifecycle before close. It
does not rewrite immutable Runtime records or modify any adopter repository.

[简体中文](WI-479-wi478-doc-promotion.zh-CN.md) · [日本語](WI-479-wi478-doc-promotion.ja.md)

## Scope

- Keep the three WI-478 Work Item pages and three reference-parity ledgers
  bound to immutable WI-478 lifecycle records.
- Register this Work Item in all three parity ledgers while its own close is
  pending, then promote that registration after verified close.
- Keep the post-close documentation promotion check deterministic.

## Out of scope

Runtime or protocol behavior, release packaging, CI policy, reference-source
implementation, adopter repositories, global Agent/MCP configuration, and
immutable Contract, evidence, archive, finalization, recovery, or close bytes.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-478-release-v0-2-57`
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`

The final status and terminal evidence links for this page are promoted only
after reviewed merge, finalization, and close.
