---
author: AI Cockpit maintainers
title: "WI-790 — WI-785 と WI-787 のドキュメント投影"
description: "Runtime の recovery と close の検証後に bounded な三言語ドキュメント投影を修復する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-790-doc-promotion
lastVerifiedBy: WI-790-doc-promotion
---

[English](WI-790-doc-promotion.md) · [简体中文](WI-790-doc-promotion.zh-CN.md)

# WI-790 — WI-785 と WI-787 のドキュメント投影

## Intent と境界

WI-790 は bounded な documentation Work Item です。不変の Runtime 記録から
WI-787 の terminal projection を昇格し、WI-785 の recovered parity projection を
修復します。archive、evidence、finalization、close、recovery の bytes は書き換えず、
製品動作、Runtime 動作、Release 状態も変更しません。

## Acceptance

- 三言語の WI-787 ページが実際の terminal archive、verification、finalization、close path を参照する。
- 三言語の WI-785 ページと parity row が不変の試行を recovered と示し、実際の hashed supersede decision と verification を参照する。
- WI-790 自身の三言語ページと parity row が archive 前に登録される。
- documentation、parity、governance-integrity、status-consistency checks が不変記録を書き換えずに pass する。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-787-parity-finalization-recovery`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test --locked --workspace`

