---
author: AI Cockpit maintainers
title: "WI-575 — terminal documentation promotion for WI-574"
description: "Promote the closed WI-574 release documentation without rewriting immutable governance records."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-575-wi574-doc-promotion
lastVerifiedBy: WI-575-wi574-doc-promotion
---

[简体中文](WI-575-wi574-doc-promotion.zh-CN.md) · [日本語](WI-575-wi574-doc-promotion.ja.md)

# WI-575 — terminal documentation promotion for WI-574

## Objective

Promote the verified-close documentation pages for WI-574 and register this
documentation projection in the three-language parity matrix. Immutable
governance records remain unchanged.

## Scope and boundary

The scope is the three WI-574 pages, the three WI-575 pages, and the three
reference-parity pages. Runtime behavior, release artifacts, object
repositories, global Agent/MCP settings, and historical governance bytes are
out of scope.

## Acceptance

- WI-574 pages are `implemented` and link to archive, verification,
  finalization, and close evidence in all three languages.
- The three parity pages identify WI-574 as implemented and register WI-575's
  bounded terminal projection with its evidence paths.
- Documentation, parity, promotion, status-consistency, governance-integrity,
  and diff checks pass without rewriting immutable governance records.
- WI-575 has readable matching English, Simplified Chinese, and Japanese
  pages.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh .`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report /tmp/wi575-governance-report.json`
- `git diff --check`
