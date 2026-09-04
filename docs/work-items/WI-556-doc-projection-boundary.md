---
author: AI Cockpit maintainers
title: "WI-556 — bounded documentation projection"
description: "Record a finite, exact documentation projection boundary for closed release work."
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-556-doc-projection-boundary
lastVerifiedBy: WI-556-doc-projection-boundary
---

[简体中文](WI-556-doc-projection-boundary.zh-CN.md) · [日本語](WI-556-doc-projection-boundary.ja.md)

# WI-556 — bounded documentation projection

## Objective

Record a finite, exact scope for documentation-only terminal projections so
the closed-work-item promotion check does not create an unbounded successor chain.

## Boundary

Only the three WI page files and three reference-parity files named by the
Contract are in scope. Runtime, source, CI, evidence, and object repositories
are out of scope.

## Acceptance

- The exact six documentation paths are registered before archive and remain consistent with terminal evidence.
- The closed Work Item promotion check reports a bounded self-projection.
