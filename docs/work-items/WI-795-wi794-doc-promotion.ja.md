---
author: AI Cockpit maintainers
title: "WI-795 — WI-794 closed documentation promotion"
description: "WI-794 の Runtime close evidence を bounded な三言語 documentation projection に昇格する。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-795-wi794-doc-promotion
lastVerifiedBy: WI-796-wi795-doc-promotion-retry
recoveryDecision: .ai/decisions/WI-795-wi794-doc-promotion.recovery.json
---

[English](WI-795-wi794-doc-promotion.md) · [简体中文](WI-795-wi794-doc-promotion.zh-CN.md)

# WI-795 — WI-794 closed documentation promotion

## Intent and boundary

WI-795 は WI-794 の post-close documentation Work Item です。immutable な
WI-794 Contract、verification、finalization、close record を、英語、簡体字中国語、
日本語の Work Item page と reference-parity table に投影します。

Runtime source、product behavior、release state、governance rule、WI-794 の
immutable record は変更しません。

## Scope

- WI-794 の terminal documentation projection を三言語で昇格します。
- close 前にこの Work Item 自身の三言語 planning page と parity row を登録し、
  close 後の projection の境界を保ちます。
- promotion helper と documentation acceptance check を再現可能に保ちます。

## Acceptance

- WI-794 の三言語 page が実際の terminal archive、verification、finalization、
  close path を bind します。
- WI-795 自身の三言語 page と parity row が archive 前に登録され、verified close
  後にのみ昇格されます。
- documentation、parity、governance-integrity、status-consistency check が
  immutable record を書き換えずに pass します。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-794-release-v0-2-90-closure --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
