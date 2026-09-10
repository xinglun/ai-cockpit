---
author: AI Cockpit maintainers
title: "WI-774 — WI-773 terminal documentation promotion"
description: "Promote the closed WI-773 experiment into the required three-language documentation and parity boundary."
audience: [maintainer, reviewer, adopter]
status: recovered
authority: authorized
workItemId: WI-774-wi773-doc-promotion
lastVerifiedBy: WI-774-wi773-doc-promotion
---

[简体中文](WI-774-wi773-doc-promotion.zh-CN.md) · [日本語](WI-774-wi773-doc-promotion.ja.md)

# WI-774 — WI-773 terminal documentation promotion

## Intent and boundary

WI-774 is preserved as an immutable failed documentation delivery. Its hosted
PR #757 failed `docs_governance_integrity` because parity registration and
verification evidence were introduced in the same commit. WI-775 is the
bounded successor that redelivers this projection from the latest default
branch. WI-773's immutable Contract, benchmark evidence, Outcome, finalization,
close, and declined-candidate decision remain unchanged.

## Scope

- Preserve the WI-774 archive, verification evidence, recovery decision, and
  hosted failure binding without rewriting any historical bytes.
- Hand the ordered redelivery to WI-775, whose parity registration precedes
  fresh verification evidence.
- Keep Runtime-generated records, historical evidence, governance rules, and
  other Work Items unchanged.

## Immutable recovery binding

- Archive: `.ai/work-items/archive/WI-774-wi773-doc-promotion.archive.json`
- Verification: `.ai/evidence/WI-774-wi773-doc-promotion.verification.json`
- Recovery: `.ai/decisions/WI-774-wi773-doc-promotion.recovery.json`
- Failed delivery: [PR #757](https://github.com/xinglun/ai-cockpit/pull/757)
- Successor: WI-775 parity-order recovery

## Acceptance and verification boundary

- `promote_closed_work_item.py --check` reports WI-773 current.
- The three-language parity, documentation acceptance, and Work Item status
  consistency checks pass.
- Runtime verification binds the current Contract and declared documentation
  checks before finish, archive, review, merge, finalization, and close.
