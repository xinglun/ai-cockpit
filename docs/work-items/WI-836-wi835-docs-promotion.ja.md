---
author: AI Cockpit maintainers
title: "WI-836 — WI-835 documentation promotion"
description: "closed WI-835 の証拠付き reader documentation を promotion する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-836-wi835-docs-promotion
lastVerifiedBy: WI-836-wi835-docs-promotion
terminalArchive: .ai/work-items/archive/WI-836-wi835-docs-promotion.contract.json
terminalVerification: .ai/evidence/WI-836-wi835-docs-promotion.verification.json
terminalDecision: .ai/decisions/WI-836-wi835-docs-promotion.close.json
---

[English](WI-836-wi835-docs-promotion.md) · [简体中文](WI-836-wi835-docs-promotion.zh-CN.md)

# WI-836 — WI-835 documentation promotion

## Intent と boundary

WI-836 は closed WI-835 lifecycle cleanup disposition の English、Simplified Chinese、
Japanese reader page と reference-parity row を promotion します。projection は Runtime
が所有する正確な archive、verification、close record を参照し、それらの record を書き換えません。

Runtime behavior、release artifact、branch deletion、historical evidence、無関係な Work Item は範囲外です。

## Acceptance

- WI-835 の三言語 page は regular non-symlink file で、同じ intent、scope、evidence、terminal state を記述する。
- 各 parity ledger に WI-835 row が一つだけあり、対応 page と immutable archive、verification、close evidence を参照する。
- verification 前に WI-836 自身の三言語 page と parity row の pre-archive 形が存在する。
- WI-835 targeted promotion check が Runtime evidence を書き換えずに成功する。
- repository-wide promotion check が成功するか、独立した evidence のある既存項目だけを報告する。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-835-lifecycle-cleanup --check`。
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`。
- `git diff --check` と六つの projected page、三つの parity ledger の path/type check。
