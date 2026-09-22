---
author: AI Cockpit maintainers
title: "WI-989 — WI-988 terminal documentation projection"
description: "WI-988 の verification 済み close 後に、範囲を限定した terminal documentation projection を完了する。"
audience: [maintainer, reviewer, contributor]
status: recovered
authority: authorized
workItemId: WI-989-wi988-doc-promotion
lastVerifiedBy: WI-989-wi988-doc-promotion
---

[English](WI-989-wi988-doc-promotion.md) · [简体中文](WI-989-wi988-doc-promotion.zh-CN.md)

# WI-989 — WI-988 terminal documentation projection

WI-989 は immutable な failed predecessor として保持される。最初の verification は
self-projection page 不足で拒否され、二度目の preflight は初回 scope 宣言に説明文と
path entry が混在していたため拒否された。failed-attempt evidence は `.ai/evidence/`
に残り、recovery binding は `.ai/decisions/WI-989-wi988-doc-promotion.recovery.json`、
retirement receipt は `.ai/decisions/WI-989-wi988-doc-promotion.retirement.json` である。
WI-990 が exact path-scoped な WI-988 projection を担当する。WI-989 は verification
または close を主張しない。

## Historical boundary

- failed precondition evidence は二つの `verification-attempt` file に保持される。
- recovery record は WI-990 を successor として明示的に bind する。
- WI-989 は verification、completion、close を主張しない。

## Acceptance

- WI-989 の failed attempt と recovery binding が監査可能な形で保持される。
- WI-990 が WI-988 の immutable evidence に基づく corrected terminal projection を担当する。
