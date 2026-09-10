---
author: AI Cockpit maintainers
title: "WI-784 — WI-783 terminal documentation promotion"
description: "Promote the closed WI-783 evidence into a bounded, tri-language documentation projection."
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-784-wi783-doc-promotion
lastVerifiedBy: WI-784-wi783-doc-promotion
---

[简体中文](WI-784-wi783-doc-promotion.zh-CN.md) · [日本語](WI-784-wi783-doc-promotion.ja.md)

# WI-784 — WI-783 terminal documentation promotion

## Intent and boundary

WI-784 is the bounded post-close documentation Work Item for WI-783. It
projects the immutable WI-783 Contract, verification, finalization, and close
records into the English, Simplified Chinese, and Japanese Work Item pages and
reference-parity tables.

It does not change Runtime source, product behavior, release state, governance
rules, or immutable WI-781, WI-782, or WI-783 records.

## Scope

- Promote the WI-783 terminal documentation projection in three languages.
- Keep this Work Item's own three planning pages and parity rows registered
  before close so the projection remains bounded after close.
- Keep the promotion helper and documentation acceptance checks reproducible.

## Verification

The declared documentation checks are:

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-783-parity-finalization-recovery --check`

`bash tests/docs/documentation_acceptance.sh --repo <repo>`

The attempt stopped truthfully at `finish.preflight` because its initial
Contract used Runtime-invalid authority and evidence-class values. The
attempt remains immutable and is continued by WI-785; WI-784 is not claimed as
completed. The recovery boundary is
`.ai/decisions/WI-784-wi783-doc-promotion.recovery.json`.
