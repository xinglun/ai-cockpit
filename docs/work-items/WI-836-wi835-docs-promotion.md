---
author: AI Cockpit maintainers
title: "WI-836 — WI-835 documentation promotion"
description: "Promote the evidence-bound reader documentation for closed WI-835."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-836-wi835-docs-promotion
lastVerifiedBy: WI-836-wi835-docs-promotion
terminalArchive: .ai/work-items/archive/WI-836-wi835-docs-promotion.contract.json
terminalVerification: .ai/evidence/WI-836-wi835-docs-promotion.verification.json
terminalDecision: .ai/decisions/WI-836-wi835-docs-promotion.close.json
---

[简体中文](WI-836-wi835-docs-promotion.zh-CN.md) · [日本語](WI-836-wi835-docs-promotion.ja.md)

# WI-836 — WI-835 documentation promotion

## Intent and boundary

WI-836 promotes the reader-facing English, Simplified Chinese, and Japanese
pages and reference-parity rows for the closed WI-835 lifecycle cleanup
disposition. The projection must link the exact Runtime-owned archive,
verification, and close records without rewriting those records.

Runtime behavior, release artifacts, branch deletion, historical evidence, and
unrelated Work Items are outside this Work Item.

## Acceptance

- The three WI-835 language pages are regular non-symlink files and describe
  the same bounded intent, scope, evidence, and terminal state.
- Each parity ledger contains exactly one WI-835 row linking the matching page
  and immutable archive, verification, and close evidence.
- The three WI-836 language pages and parity rows exist in pre-archive form
  before verification.
- The targeted WI-835 promotion check passes without rewriting Runtime evidence.
- The repository-wide promotion check passes, or reports only independently
  pre-existing items with explicit evidence.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-835-lifecycle-cleanup --check`.
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`.
- `git diff --check` and exact path/type checks for the six projected pages and
  three parity ledgers.
