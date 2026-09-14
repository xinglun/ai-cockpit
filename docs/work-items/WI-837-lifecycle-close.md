---
author: AI Cockpit maintainers
title: "WI-837 — lifecycle close projection"
description: "Promote the evidence-bound reader documentation for the closed WI-837 lifecycle close integration."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-837-lifecycle-close
lastVerifiedBy: WI-837-lifecycle-close
terminalArchive: .ai/work-items/archive/WI-837-lifecycle-close.contract.json
terminalVerification: .ai/evidence/WI-837-lifecycle-close.verification.json
terminalDecision: .ai/decisions/WI-837-lifecycle-close.close.json
---

[简体中文](WI-837-lifecycle-close.zh-CN.md) · [日本語](WI-837-lifecycle-close.ja.md)

# WI-837 — lifecycle close projection

## Intent and boundary

WI-837 integrates the Runtime-owned terminal records for the post-merge WI-836
close and completes its reader-facing documentation projection. The projection
must preserve the exact archive, verification, and close evidence.

Source behavior, product artifacts, releases, workspace verification, remote
branch deletion, and unrelated Work Items are outside this Work Item.

## Acceptance

- The three WI-837 language pages are regular non-symlink files and describe the same bounded intent, scope, evidence, and terminal state.
- Each parity ledger contains exactly one WI-837 row linking the matching page and immutable terminal evidence.
- The targeted and repository-wide promotion checks pass without rewriting Runtime evidence.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-837-lifecycle-close --check`.
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`.
- `git diff --check` and exact path/type checks for the three projected pages and three parity ledgers.
