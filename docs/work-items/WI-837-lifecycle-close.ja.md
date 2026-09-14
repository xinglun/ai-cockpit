---
author: AI Cockpit maintainers
title: "WI-837 — lifecycle close projection"
description: "closed WI-837 lifecycle close integration の証拠付き reader documentation を promotion する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-837-lifecycle-close
lastVerifiedBy: WI-837-lifecycle-close
terminalArchive: .ai/work-items/archive/WI-837-lifecycle-close.contract.json
terminalVerification: .ai/evidence/WI-837-lifecycle-close.verification.json
terminalDecision: .ai/decisions/WI-837-lifecycle-close.close.json
---

[English](WI-837-lifecycle-close.md) · [简体中文](WI-837-lifecycle-close.zh-CN.md)

# WI-837 — lifecycle close projection

## Intent と boundary

WI-837 は post-merge WI-836 close integration の Runtime terminal record を統合し、reader-facing
documentation projection を完了する。projection は正確な archive、verification、close evidence を保持する。

Source behavior、product artifact、release、workspace verification、remote branch deletion、無関係な Work Item は範囲外です。

## Acceptance

- WI-837 の三言語 page は regular non-symlink file で、同じ bounded intent、scope、evidence、terminal state を記述する。
- 各 parity ledger に WI-837 row が一つだけあり、対応 page と immutable terminal evidence を参照する。
- targeted と repository-wide promotion check が Runtime evidence を書き換えずに成功する。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-837-lifecycle-close --check`。
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`。
- `git diff --check` と三つの projected page、三つの parity ledger の path/type check。
