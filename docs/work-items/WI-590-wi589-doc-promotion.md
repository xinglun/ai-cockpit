---
author: AI Cockpit maintainers
title: "WI-590 — WI-589 terminal documentation promotion"
description: "Promote the WI-589 documentation projections after its verified close."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-590-wi589-doc-promotion
lastVerifiedBy: WI-590-wi589-doc-promotion
---

[简体中文](WI-590-wi589-doc-promotion.zh-CN.md) · [日本語](WI-590-wi589-doc-promotion.ja.md)

# WI-590 — WI-589 terminal documentation promotion

## Objective

Promote the three-language WI-589 Work Item and reference-parity projections
after its immutable archive, verification, finalization, and close evidence
were validated. This Work Item is a documentation-only terminal projection.

## Boundary

Runtime behavior, object repositories, global Agent/MCP configuration, and
generated evidence or decision bytes are outside this Work Item. Governance
facts remain derived from immutable Runtime records.

## Acceptance

1. WI-589 English, Chinese, and Japanese pages report terminal evidence paths
   and implemented status without changing their semantic content.
2. The three parity rows for WI-589 report matching terminal evidence paths.
3. WI-590's own pages and parity rows are registered for deterministic
   self-terminal documentation promotion; no additional documentation debt is
   introduced by closing this Work Item.

## Verification

Run `python3 tests/docs/promote_closed_work_item.py --repo <repository>
--check-all`, `tests/docs/documentation_acceptance.sh`, and
`tests/docs/parity_status_check.sh` with the explicit repository context.
