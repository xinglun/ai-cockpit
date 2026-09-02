---
author: AI Cockpit maintainers
title: "WI-492 — WI-491 terminal documentation promotion"
description: "Promote the closed WI-491 release evidence into reader-facing documentation before publishing v0.2.58."
audience: [maintainer, reviewer, adopter]
workItemId: WI-492-wi491-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-492-wi491-doc-promotion
---

# WI-492 — WI-491 terminal documentation promotion

This bounded documentation Work Item promotes the closed WI-491 release
evidence into the three language pages and the parity ledger. It preserves
immutable governance records and does not change Runtime behavior.

[简体中文](WI-492-wi491-doc-promotion.zh-CN.md) · [日本語](WI-492-wi491-doc-promotion.ja.md)

## Scope

- Promote the three WI-491 Work Item pages and their three parity rows.
- Bind each projection to WI-491 archive, verification, finalization, and close evidence.
- Keep this Work Item's own pages and parity row in the same bounded lifecycle.

## Acceptance

- All six WI-491 projections are evidence-backed without rewriting immutable records.
- The closed Work Item promotion check and status-consistency check pass.
- No Runtime source, reference inventory classification, or global Agent/MCP configuration changes.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `git diff --check`
