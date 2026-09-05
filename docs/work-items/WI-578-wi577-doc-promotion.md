---
author: AI Cockpit maintainers
title: "WI-578 — WI-577 terminal documentation promotion"
description: "Promote the closed WI-577 documentation projection without rewriting immutable records."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-578-wi577-doc-promotion
lastVerifiedBy: WI-578-wi577-doc-promotion
---

[简体中文](WI-578-wi577-doc-promotion.zh-CN.md) · [日本語](WI-578-wi577-doc-promotion.ja.md)

# WI-578 — WI-577 terminal documentation promotion

## Objective

Promote the closed WI-577 Work Item pages and parity registration so the
documentation projection is truthful and the repository can pass its
post-close documentation gate.

## Boundary

Only the three WI-577 pages, three parity rows, and this tri-language
promotion record are in scope. WI-577 archive/evidence/decision bytes, Runtime
behavior, object repositories, global configuration, and historical prose are
immutable or out of scope.

## Acceptance

- The three WI-577 pages are `implemented` and link to terminal archive,
  verification, finalization, and close evidence.
- Each parity page records WI-577 as implemented with its bounded metadata
  guard; no semantic comparison claim is added.
- Documentation acceptance, status consistency, and promotion `--check-all`
  pass in all three languages.
- No immutable governance record is rewritten.

## Verification

See the active Contract and `tests/docs/promote_closed_work_item.py`.
