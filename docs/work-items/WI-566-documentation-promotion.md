---
author: AI Cockpit maintainers
title: "WI-566 — documentation promotion for WI-565"
description: "Promote the verified-close documentation projections for WI-565 and register this bounded promotion Work Item."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-566-documentation-promotion
lastVerifiedBy: WI-566-documentation-promotion
---

[简体中文](WI-566-documentation-promotion.zh-CN.md) · [日本語](WI-566-documentation-promotion.ja.md)

# WI-566 — documentation promotion for WI-565

## Objective

Promote the three-language documentation projections for the verified and
closed WI-565 release, and keep this documentation Work Item auditable in the
same projections. Immutable Runtime evidence is referenced, not rewritten.

## Scope and boundary

The scope is the three WI-565 pages, the three WI-566 pages, the three
reference-parity pages, and the closed-Work-Item promotion helper. Runtime
behavior, release artifacts, object repositories, global Agent/MCP settings,
and immutable Contract, evidence, decision, or archive bytes are out of scope.

## Acceptance

- WI-565 pages are `implemented` and link to archive, verification,
  finalization, and close evidence in all three languages.
- The three parity pages identify WI-565 as implemented and register WI-566's
  bounded pre-archive projection.
- Documentation, parity, promotion, and diff checks pass without rewriting
  historical governance records.
- WI-566 itself has readable three-language documentation and one matching
  pre-archive parity entry.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

