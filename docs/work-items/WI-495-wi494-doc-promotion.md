---
author: AI Cockpit maintainers
title: "WI-495 — WI-494 terminal documentation promotion"
description: "Promote the closed WI-494 comparison evidence and terminate the documentation gate loop."
audience: [maintainer, reviewer, adopter]
workItemId: WI-495-wi494-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-495-wi494-doc-promotion
---

# WI-495 — WI-494 terminal documentation promotion

This bounded documentation Work Item promotes WI-494's terminal comparison
evidence and parity registration. Its own three language pages are included so
the post-close documentation check remains self-terminal.

[简体中文](WI-495-wi494-doc-promotion.zh-CN.md) · [日本語](WI-495-wi494-doc-promotion.ja.md)

## Scope

- Keep the three WI-494 pages and parity rows bound to immutable terminal receipts.
- Provide the three WI-495 pages and parity rows required by the documentation gate.
- Change no Runtime behavior, reference inventory, or global Agent/MCP configuration.

## Acceptance

- WI-494 documentation links archive, verification, finalization, and close evidence.
- Documentation promotion, governance-integrity, status-consistency, and parity checks pass.
- The projection remains readable in English, Simplified Chinese, and Japanese.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
