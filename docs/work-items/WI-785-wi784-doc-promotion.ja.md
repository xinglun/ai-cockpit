---
author: AI Cockpit maintainers
title: "WI-785 — WI-784 documentation-promotion recovery"
description: "Runtime-valid な WI-784 successor により WI-783 の terminal documentation projection を完了する。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorized-successor-recovery
workItemId: WI-785-wi784-doc-promotion
lastVerifiedBy: WI-785-wi784-doc-promotion
---

[English](WI-785-wi784-doc-promotion.md) · [简体中文](WI-785-wi784-doc-promotion.zh-CN.md)

# WI-785 — WI-784 documentation-promotion recovery

## Intent と boundary

WI-785 は `finish.preflight` で停止した WI-784 の明示的な successor である。
`authority: authorized` と supported な `verification` evidence class を使用し、
close 済み WI-783 evidence の bounded な三言語 documentation projection を完了する。

WI-784 の Contract、Summary、Outcome、verification、recovery record は不変の
predecessor evidence である。本 Work Item はそれらを書き換えず、Runtime source、
product behavior、release state、無関係な docs も変更しない。

## Verification

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-783-parity-finalization-recovery --check`

`bash tests/docs/documentation_acceptance.sh --repo <repo>`

Successor binding は `.ai/decisions/WI-784-wi783-doc-promotion.recovery.json` である。
