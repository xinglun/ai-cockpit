---
author: AI Cockpit maintainers
title: "WI-602 — WI-601 terminal documentation promotion"
description: "Promote the missing WI-601 terminal parity projection without changing governance facts."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-602-doc-promotion-wi601
lastVerifiedBy: WI-602-doc-promotion-wi601
---

[简体中文](WI-602-doc-promotion-wi601.zh-CN.md) · [日本語](WI-602-doc-promotion-wi601.ja.md)

# WI-602 — WI-601 terminal documentation promotion

## Objective

Promote the missing tri-language parity row for closed WI-601 so the closed
Work Item documentation check has one auditable terminal projection.

## Boundary

This Work Item changes only the three reference-parity pages and these three
documentation records. Runtime behavior, reference-source scaffolding or wire
formats, object repositories, global Agent/MCP configuration, and generated
evidence or decision bytes are outside the boundary.

## Acceptance

1. The English, Chinese, and Japanese parity tables each contain exactly one
   WI-601 terminal row with its immutable archive, verification, finalization,
   and close paths.
2. The three WI-602 pages are registered before evidence generation and retain
   the Contract's original language and human-owned boundaries.
3. Documentation, metadata, and closed-work-item promotion checks pass without
   changing generated governance facts.
4. No reference-source scaffold is copied and no object/adopter repository is
   modified.

## Verification

Run the documentation acceptance, reference metadata, and closed-work-item
promotion checks with the explicit repository context, plus the Contract's
locked workspace verification command.
