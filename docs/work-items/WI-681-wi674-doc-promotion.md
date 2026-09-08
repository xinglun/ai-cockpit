---
author: AI Cockpit maintainers
title: "WI-681 — WI-674 terminal documentation promotion"
description: "Promote the closed WI-674 repository-split Work Item into the governed three-language documentation projection."
audience: [maintainer, reviewer, adopter]
workItemId: WI-681-wi674-doc-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-681-wi674-doc-promotion
---

[简体中文](WI-681-wi674-doc-promotion.zh-CN.md) · [日本語](WI-681-wi674-doc-promotion.ja.md)

# WI-681 — WI-674 terminal documentation promotion

## Intent

Synchronize the three-language WI-674 Work Item pages and reference-parity
rows with the immutable archive, verification, finalization, and close records.

## Boundary

This is a documentation-only projection. The six WI-674 projection files and
three WI-681 self-registration pages are the only intended changes. Immutable
`.ai` lifecycle records are read-only inputs; no runtime behavior, governance
rule, historical evidence, or other agent worktree is changed.

## Acceptance

- All three WI-674 language pages expose terminal `Implemented` status and exact
  terminal evidence paths after verified close.
- All three WI-674 parity rows expose matching terminal status and evidence.
- This WI is registered in its own three-language pages and parity rows so the
  closed-work-item promotion check remains bounded and repeatable.
- Documentation, parity, status-consistency, and closed-work-item promotion
  checks pass on the exact reviewed head.
- `user_visible_benefit_not_declared` remains explicit; this projection makes
  no user-visible, performance, or cognitive-benefit claim.

WI-674 terminal evidence:

- archive: `.ai/work-items/archive/WI-674-p1-repository-split.contract.json`
- verification: `.ai/evidence/WI-674-p1-repository-split.verification.json`
- finalization: `.ai/decisions/WI-674-p1-repository-split.finalize.json`
- close: `.ai/decisions/WI-674-p1-repository-split.close.json`
