---
author: AI Cockpit maintainers
title: "WI-567 — terminal documentation promotion for WI-566"
description: "Promote the closed WI-566 documentation projection without rewriting immutable governance records."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-567-wi566-doc-promotion
lastVerifiedBy: WI-567-wi566-doc-promotion
---

[简体中文](WI-567-wi566-doc-promotion.zh-CN.md) · [日本語](WI-567-wi566-doc-promotion.ja.md)

# WI-567 — terminal documentation promotion for WI-566

## Objective

Promote the verified-close documentation pages for WI-566 and record this
bounded promotion in the three-language reference matrix. Immutable Contracts,
evidence, decisions, and archive records remain unchanged.

## Scope and boundary

The scope is the three WI-566 pages, the three WI-567 pages, and the three
reference-parity pages. Runtime behavior, release artifacts, object
repositories, global Agent/MCP settings, and historical governance bytes are
out of scope.

## Acceptance

- WI-566 pages are `implemented` and link to archive, verification,
  finalization, and close evidence in all three languages.
- The three parity pages identify WI-566 as implemented and register WI-567's
  bounded pre-archive projection.
- Documentation, parity, promotion, and diff checks pass without rewriting
  immutable governance records.
- WI-567 itself has readable three-language documentation and one matching
  pre-archive parity entry.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --work-item WI-566-documentation-promotion`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

