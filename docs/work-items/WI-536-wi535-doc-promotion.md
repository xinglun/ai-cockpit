---
author: AI Cockpit maintainers
title: "WI-536 — WI-535 terminal documentation promotion"
description: "Promote WI-535 reader documentation and register this documentation Work Item before archive."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-536-wi535-doc-promotion
lastVerifiedBy: WI-536-wi535-doc-promotion
---

[简体中文](WI-536-wi535-doc-promotion.zh-CN.md) · [日本語](WI-536-wi535-doc-promotion.ja.md)

## Goal

Synchronize the three-language WI-535 pages with immutable terminal evidence,
and register this Work Item in every parity ledger before verification and
archive.

## Scope and boundary

- WI-535's three-language reader pages.
- WI-536's three-language reader pages.
- The English, Japanese, and Simplified Chinese parity ledgers.
- Runtime behavior, generated `.ai` records, release artifacts, and object
  repositories are outside this Work Item.

## Acceptance

- WI-535 reader pages and parity rows bind exact terminal evidence.
- WI-536 is registered in all three parity ledgers before verification and
  archive.
- Documentation, parity, and governance integrity checks pass.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-535-mcp-fixture-cleanup
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
tests/ci/governance_integrity_gate.py --repo <repo>
```
