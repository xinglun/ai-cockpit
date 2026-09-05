---
author: AI Cockpit maintainers
title: "WI-599 — WI-598 terminal documentation promotion"
description: "Promote the verified WI-598 documentation projections with pre-registered tri-language parity evidence."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-599-wi598-doc-promotion
lastVerifiedBy: WI-599-wi598-doc-promotion
---

[简体中文](WI-599-wi598-doc-promotion.zh-CN.md) · [日本語](WI-599-wi598-doc-promotion.ja.md)

# WI-599 — WI-598 terminal documentation promotion

## Objective

Promote the tri-language Work Item and reference-parity projections for
WI-598 after its immutable archive, verification, finalization, and close
receipts are valid. Register this Work Item's own projection before creating
new verification evidence so the governance-integrity gate can audit the
complete lifecycle.

## Boundary

This Work Item changes documentation projections only. Runtime behavior,
object repositories, global Agent/MCP configuration, source implementation,
and generated evidence or decision bytes are outside the boundary. Contract
acceptance remains authoritative in its authoring language.

## Acceptance

1. The three WI-598 Work Item pages contain terminal paths derived from its
   immutable archive, verification, finalization, and close receipts.
2. The three reference-parity rows report WI-598 as implemented with matching
   terminal evidence paths.
3. This WI-599 record and its three parity rows are registered before any
   verification evidence is generated and are promoted only after close.
4. No governance facts, source implementation, object repository, or
   generated receipt bytes are changed.

## Verification

Run `tests/docs/promote_closed_work_item.py --check`,
`tests/docs/documentation_acceptance.sh`,
`tests/docs/parity_status_check.sh`, the reference inventory and metadata
regressions, and the locked workspace checks with the explicit repository
context.
