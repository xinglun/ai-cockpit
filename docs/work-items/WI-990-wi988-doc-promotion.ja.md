---
author: AI Cockpit maintainers
title: "WI-990 — WI-988 terminal documentation projection"
description: "WI-989 の replacement 後に、範囲を限定した WI-988 terminal documentation projection を完了する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-990-wi988-doc-promotion
lastVerifiedBy: WI-990-wi988-doc-promotion
---

[English](WI-990-wi988-doc-promotion.md) · [简体中文](WI-990-wi988-doc-promotion.zh-CN.md)

# WI-990 — WI-988 terminal documentation projection

WI-990 は WI-989 に明示的に bind された successor であり、WI-988 の immutable archive、
verification、close の事実を exact path-scoped に projection する。replacement された
WI-989 の attempt は保持され、verification や close は主張しない。

## Boundary

対象は WI-988 の terminal page、WI-989 recovered predecessor page、三つの WI-990 page、
三つの reference-parity ledger に限定する。source code、release behavior、既存 lifecycle
receipt は対象外である。

## Acceptance

- WI-988 の三言語 page と parity row が immutable evidence に基づく terminal fact を示す。
- WI-989 の三言語 page と parity row が failed attempt、recovery、retirement、WI-990 binding を保持する。
- WI-990 の三言語 page と parity row が自身の bounded projection を保持する。
- WI-988 の単体 projection と repository-wide `--check-all` が通る。
