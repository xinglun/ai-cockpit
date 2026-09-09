---
author: AI Cockpit maintainers
title: "WI-745 — WI-743 terminal documentation promotion"
description: "Promote the verified WI-743 terminal documentation and repair the hosted documentation-governance projections."
audience: [maintainer, reviewer, adopter]
workItemId: WI-745-wi743-doc-promotion
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-745-wi743-doc-promotion
recoveryDecision: .ai/decisions/WI-745-wi743-doc-promotion.recovery.6ef6b1b0e804eaf246518a1839851b47b0de88cb9a18c73282f21fb0816d1e41.json
closeDecision: .ai/decisions/WI-745-wi743-doc-promotion.close.json
---

[简体中文](WI-745-wi743-doc-promotion.zh-CN.md) · [日本語](WI-745-wi743-doc-promotion.ja.md)

# WI-745 — WI-743 terminal documentation promotion

## Intent

Promote the verified terminal documentation for WI-743 and register this
documentation-only successor in the three reference-parity ledgers. The
successor repairs the hosted documentation-governance projection; it does not
change the already closed WI-743 records or its merged performance delivery.

## Boundary

This Work Item is limited to the six WI-743 documentation projections and the
tri-language WI-745 Work Item/parity projections required before archive. It
does not change production code, tests, governance rules, WI-743 archive or
decision records, PR #716, WI-744, or the WI-742 predecessor history.

## Evidence and lifecycle

- WI-743 terminal archive, verification, finalization, and close records are
  the immutable sources for the promoted content.
- PR #718 carries this documentation-only successor from the synchronized
  `origin/main` base.
- The merged predecessor is retained as immutable recovered history through
  the Runtime `supersede` decision. WI-745 is closed as that historical
  predecessor; follow-up ownership is WI-746. This page does not claim a fresh
  performance or user-visible benefit.
- The current supersede receipt is
  `.ai/decisions/WI-745-wi743-doc-promotion.recovery.6ef6b1b0e804eaf246518a1839851b47b0de88cb9a18c73282f21fb0816d1e41.json`;
  the corresponding close decision is
  `.ai/decisions/WI-745-wi743-doc-promotion.close.json`.

## Acceptance

The promotion helper, documentation acceptance, parity status check, and Work
Item status consistency check must pass. No archived evidence is rewritten,
and no production behavior or governance decision is introduced.
