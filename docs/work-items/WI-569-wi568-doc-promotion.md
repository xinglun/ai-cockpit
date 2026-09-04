---
author: AI Cockpit maintainers
title: "WI-569 — terminal documentation promotion for WI-568"
description: "Promote the closed WI-568 documentation projection without rewriting immutable governance records."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-569-wi568-doc-promotion
lastVerifiedBy: WI-569-wi568-doc-promotion
---

[简体中文](WI-569-wi568-doc-promotion.zh-CN.md) · [日本語](WI-569-wi568-doc-promotion.ja.md)

# WI-569 — terminal documentation promotion for WI-568

## Objective

Promote the verified-close documentation pages for WI-568 and preserve their
archive, evidence, finalization, and close references in the three-language
parity matrix. Immutable governance records remain unchanged.

## Scope and boundary

The scope is the three WI-568 pages, the three WI-569 pages, and the three
reference-parity pages. Runtime behavior, release artifacts, object
repositories, global Agent/MCP settings, and historical governance bytes are
out of scope.

## Acceptance

- WI-568 pages are `implemented` and link to archive, verification,
  finalization, and close evidence in all three languages.
- The three parity pages identify WI-568 as implemented and register WI-569's
  bounded pre-archive projection.
- Documentation, parity, promotion, and diff checks pass without rewriting
  immutable governance records.
- WI-569 has readable three-language documentation and one matching
  pre-archive parity entry.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
