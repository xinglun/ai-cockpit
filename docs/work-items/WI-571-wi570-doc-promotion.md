---
author: AI Cockpit maintainers
title: "WI-571 — terminal documentation promotion for WI-570"
description: "Promote the closed WI-570 documentation projection without rewriting immutable governance records."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-571-wi570-doc-promotion
lastVerifiedBy: WI-571-wi570-doc-promotion
---

[简体中文](WI-571-wi570-doc-promotion.zh-CN.md) · [日本語](WI-571-wi570-doc-promotion.ja.md)

# WI-571 — terminal documentation promotion for WI-570

## Objective

Promote the verified-close documentation pages for WI-570 and register this
documentation projection in the three-language parity matrix. Immutable
governance records remain unchanged.

## Scope and boundary

The scope is the three WI-570 pages, the three WI-571 pages, and the three
reference-parity pages. Runtime behavior, release artifacts, object
repositories, global Agent/MCP settings, and historical governance bytes are
out of scope.

## Acceptance

- WI-570 pages are `implemented` and link to archive, verification,
  finalization, and close evidence in all three languages.
- The three parity pages identify WI-570 as implemented and register WI-571's
  bounded terminal projection with its evidence paths.
- Documentation, parity, promotion, and diff checks pass without rewriting
  immutable governance records.
- WI-571 has readable matching English, Simplified Chinese, and Japanese
  pages.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
