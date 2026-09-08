---
author: AI Cockpit maintainers
title: "WI-661 — WI-660 terminal documentation promotion"
description: "Promote the closed WI-660 documentation projections with immutable terminal evidence."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-661-wi660-doc-promotion
lastVerifiedBy: WI-661-wi660-doc-promotion
---

[简体中文](WI-661-wi660-doc-promotion.zh-CN.md) · [日本語](WI-661-wi660-doc-promotion.ja.md)

# WI-661 — WI-660 terminal documentation promotion

## Intent

Synchronize the three-language WI-660 Work Item pages and reference-parity rows
with the immutable archive, verification, finalization, and close records.

## Boundary

This is a documentation-only projection. The six scoped Markdown files are the
only intended changes. Immutable `.ai` archive, evidence, finalization, and
close records are read-only inputs; no Runtime, repository, or historical
Work Item behavior is changed.

## Acceptance

- All three WI-660 language pages expose terminal `Implemented` status and the
  exact terminal evidence paths after verified close.
- All three WI-660 parity rows expose the matching terminal status and evidence
  paths.
- Promotion, documentation, parity, status-consistency, governance, and
  Hosted quality checks pass on the exact reviewed head.
