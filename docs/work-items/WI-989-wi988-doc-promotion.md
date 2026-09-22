---
author: AI Cockpit maintainers
title: "WI-989 — WI-988 terminal documentation projection"
description: "Promote the bounded WI-988 terminal documentation projection after its verified close."
audience: [maintainer, reviewer, contributor]
status: recovered
authority: authorized
workItemId: WI-989-wi988-doc-promotion
lastVerifiedBy: WI-989-wi988-doc-promotion
---

[简体中文](WI-989-wi988-doc-promotion.zh-CN.md) · [日本語](WI-989-wi988-doc-promotion.ja.md)

# WI-989 — WI-988 terminal documentation projection

WI-989 is preserved as an immutable failed predecessor. Its first verification
was rejected because the self-projection pages were missing, and its second
preflight was rejected because the initial scope declaration mixed prose with
path entries. The failed-attempt evidence is retained under
`.ai/evidence/`, the recovery binding is
`.ai/decisions/WI-989-wi988-doc-promotion.recovery.json`, and the retirement
receipt is `.ai/decisions/WI-989-wi988-doc-promotion.retirement.json`. WI-990
owns the exact path-scoped bounded WI-988 projection. No verification or close
is claimed for WI-989.

## Historical boundary

- The failed precondition evidence remains at
  `.ai/evidence/WI-989-wi988-doc-promotion.verification-attempt.e38e7412919a37f5f803d679cca33e2578dc62fe5928139be10cea9ad83dbe87.json` and
  `.ai/evidence/WI-989-wi988-doc-promotion.verification-attempt.ac465ae744c4da983183e60a1833fa42909a2178292422830f8e65aee15c5a31.json`.
- The recovery record explicitly binds WI-990 as the successor.
- WI-989 does not assert verification, completion, or close.

## Acceptance

- WI-989's failed attempt and recovery binding remain inspectable.
- WI-990 owns the corrected terminal projection from immutable WI-988 evidence.
