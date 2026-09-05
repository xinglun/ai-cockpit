---
author: AI Cockpit maintainers
title: "WI-588 — WI-587 terminal documentation promotion"
description: "Promote the verified WI-587 documentation projections after its close."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-588-wi587-doc-promotion
lastVerifiedBy: WI-588-wi587-doc-promotion
---

[简体中文](WI-588-wi587-doc-promotion.zh-CN.md) · [日本語](WI-588-wi587-doc-promotion.ja.md)

# WI-588 — WI-587 terminal documentation promotion

## Objective

Promote the tri-language Work Item and reference-parity projections for
WI-587 after its immutable archive, verification, finalization, and close
receipts are valid. This Work Item changes documentation projections only.

## Boundary

Runtime behavior, object repositories, global Agent/MCP configuration, and
generated evidence or decision bytes are outside this Work Item. Contract
acceptance remains authoritative in its authoring language.

## Acceptance

1. The three WI-587 Work Item pages contain terminal paths derived from the
   immutable archive, verification, finalization, and close receipts.
2. The three reference-parity rows report WI-587 as implemented with matching
   terminal evidence paths.
3. No governance facts, source implementation, object repository, or generated
   receipt bytes are changed.

## Verification

Run `tests/docs/promote_closed_work_item.py --check`,
`tests/docs/documentation_acceptance.sh`, and
`tests/docs/parity_status_check.sh` with the explicit repository context.
