---
author: AI Cockpit maintainers
title: "WI-838 — lifecycle projection guard integration"
description: "証拠付き lifecycle projection guard を統合し、reader documentation contract を完成させる。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-838-lifecycle-projection
lastVerifiedBy: WI-838-lifecycle-projection
terminalArchive: .ai/work-items/archive/WI-838-lifecycle-projection.contract.json
terminalVerification: .ai/evidence/WI-838-lifecycle-projection.verification.json
terminalDecision: .ai/decisions/WI-838-lifecycle-projection.close.json
---

[English](WI-838-lifecycle-projection.md) · [简体中文](WI-838-lifecycle-projection.zh-CN.md)

# WI-838 — lifecycle projection guard integration

## Intent and boundary

WI-838 は WI-837 projection を current に保つための Runtime terminal record を統合し、自身の documentation projection を terminal lifecycle 前に登録する。

Runtime source behavior、release artifact、workspace verification、remote branch deletion、無関係な Work Item は対象外である。

## Acceptance

- WI-836 と WI-837 の移送 record は元の byte と evidence binding を保持する。
- WI-837 の三言語 page と parity row は公式 helper により current になる。
- WI-838 の三言語 page と parity row は verification 前に存在し、close 後に Runtime terminal record から promotion される。
- repository-wide documentation promotion は product build や workspace verification を再実行せず pass する。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-837-lifecycle-close --check`。
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`。
- `git diff --check` と projected page が regular non-symlink file であることを確認する。
