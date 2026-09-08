---
author: AI Cockpit maintainers
title: "WI-688 — WI-683 terminal documentation promotion"
description: "Promote the verified WI-683 documentation projections with immutable terminal evidence."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human:repository-owner
workItemId: WI-688-wi683-doc-promotion
lastVerifiedBy: WI-688-wi683-doc-promotion
terminalArchive: .ai/work-items/archive/WI-688-wi683-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-688-wi683-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-688-wi683-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-688-wi683-doc-promotion.close.json
---

[简体中文](WI-688-wi683-doc-promotion.zh-CN.md) · [日本語](WI-688-wi683-doc-promotion.ja.md)

# WI-688 — WI-683 terminal documentation promotion

## Intent

Synchronize the three-language WI-683 Work Item pages and reference-parity rows
with the immutable archive, verification, finalization, and close records.

## Boundary

This is a documentation-only projection. The six WI-683 Markdown projections
are the intended changes; immutable governance records are read-only inputs. No
Runtime behavior, repository implementation, or historical Work Item bytes are
changed.

## Acceptance

- The three WI-683 language pages expose terminal status and exact evidence paths after verified close.
- The three WI-683 parity rows expose the matching terminal status and evidence paths.
- Promotion, documentation, parity, status-consistency, governance, and hosted quality checks pass on the reviewed head.
