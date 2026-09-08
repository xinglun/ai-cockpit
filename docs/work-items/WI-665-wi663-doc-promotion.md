---
author: AI Cockpit maintainers
title: "WI-665 — WI-664 documentation promotion"
description: "Promote the closed WI-663 and WI-664 documentation projections and bind the WI-659 supersede decision with immutable terminal evidence."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-665-wi663-doc-promotion
lastVerifiedBy: WI-665-wi663-doc-promotion
---

[简体中文](WI-665-wi663-doc-promotion.zh-CN.md) · [日本語](WI-665-wi663-doc-promotion.ja.md)

# WI-665 — WI-664 documentation promotion

## Intent

Synchronize the three-language WI-663 and WI-664 Work Item pages and their
shared reference-parity rows with immutable archive, verification, finalization,
and close records; bind the versioned WI-659 supersede decision without editing
historical evidence.

## Boundary

This is a documentation-only projection. The twelve scoped Markdown files are
the only intended changes. Immutable `.ai` archive, evidence, recovery,
finalization, and close records are read-only inputs; no Runtime, repository,
historical Work Item, or other agent behavior is changed.

## Acceptance

- WI-663 and WI-664 pages and parity rows expose their terminal `Implemented`
  status and exact terminal evidence paths after verified close.
- The WI-659 parity row cites the canonical recovery, versioned supersede
  decision, and superseded close records without rewriting predecessor history.
- WI-665 remains auditable as the explicit prearchive self-registration for
  this bounded documentation projection.
- Promotion, documentation, parity, status-consistency, governance, and
  Hosted quality checks pass on the exact reviewed head.
