---
author: AI Cockpit maintainers
title: "WI-585 — WI-584 terminal documentation promotion"
description: "Promote the verified WI-584 release projections after its close."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-585-wi584-doc-promotion
lastVerifiedBy: WI-585-wi584-doc-promotion
---

[简体中文](WI-585-wi584-doc-promotion.zh-CN.md) · [日本語](WI-585-wi584-doc-promotion.ja.md)

# WI-585 — WI-584 terminal documentation promotion

## Objective

Promote the tri-language Work Item and reference-parity projections for
WI-584 only after its immutable archive, verification, finalization, and close
receipts are valid. This Work Item changes documentation projections, not
governance facts or Runtime behavior.

## Boundary

The object repository, Runtime implementation, global Agent/MCP configuration,
and generated evidence/decision bytes are outside this Work Item. Contract
acceptance remains authoritative in its authoring language.

## Acceptance

1. The three WI-584 Work Item pages contain terminal paths derived from the
   immutable receipts.
2. The three reference-parity rows report WI-584 as implemented with matching
   evidence paths.
3. No governance facts, source implementation, object repository, or
   generated receipt bytes are changed.

## Verification

Run `tests/docs/documentation_acceptance.sh` and the Runtime verification
command declared by the active Contract with an explicit repository context.
