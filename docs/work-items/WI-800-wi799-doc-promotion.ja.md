---
author: AI Cockpit maintainers
title: "WI-800 — WI-799 のドキュメント投影"
description: "現在の Runtime evidence により、close 済み WI-799 の三言語 Work Item と reference-parity 投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-root-cause-repair-and-release
workItemId: WI-800-wi799-doc-promotion
lastVerifiedBy: WI-800-wi799-doc-promotion
---

[English](WI-800-wi799-doc-promotion.md) · [简体中文](WI-800-wi799-doc-promotion.zh-CN.md)

# WI-800 — WI-799 のドキュメント投影

## Intent と境界

WI-800 は bounded な documentation Work Item です。不変の archive、verification、
finalization、close evidence から WI-799 の terminal projection を昇格します。
WI-799 の履歴 bytes、製品、Runtime、Release、provider の動作は変更しません。

## Acceptance

- WI-799 の英語、簡体字中国語、日本語ページが同じ terminal evidence を参照する。
- 三つの reference-parity row が同じ predecessor、evidence、finalization、close facts を参照する。
- WI-800 自身の三言語ページと parity row が archive 前に登録される。
- documentation、parity、governance-integrity、status-consistency checks が不変記録を書き換えずに pass する。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-799-wi799-doc-promotion --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
