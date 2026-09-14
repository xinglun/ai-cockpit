---
author: AI Cockpit maintainers
title: "WI-838 — lifecycle projection guard integration"
description: "Integrate the evidence-bound lifecycle projection guard and complete the reader documentation contract."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-838-lifecycle-projection
lastVerifiedBy: WI-838-lifecycle-projection
terminalArchive: .ai/work-items/archive/WI-838-lifecycle-projection.contract.json
terminalVerification: .ai/evidence/WI-838-lifecycle-projection.verification.json
terminalDecision: .ai/decisions/WI-838-lifecycle-projection.close.json
---

[简体中文](WI-838-lifecycle-projection.zh-CN.md) · [日本語](WI-838-lifecycle-projection.ja.md)

# WI-838 — lifecycle projection guard integration

## Intent and boundary

WI-838 integrates the exact Runtime-owned lifecycle records required to make
the WI-837 projection current and records this Work Item's own documentation
projection before its terminal lifecycle.

Runtime source behavior, release artifacts, workspace verification, remote
branch deletion, and unrelated Work Items are outside this Work Item.

## Acceptance

- The transferred WI-836 and WI-837 records remain byte-preserved and bound to
  their original evidence.
- The three WI-837 pages and parity rows are current under the official helper.
- The three WI-838 language pages and parity rows exist before verification and
  are promoted from its terminal Runtime records after close.
- Repository-wide documentation promotion passes without rerunning product
  builds or workspace verification.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-837-lifecycle-close --check`.
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`.
- `git diff --check` and regular non-symlink checks for the projected pages.
