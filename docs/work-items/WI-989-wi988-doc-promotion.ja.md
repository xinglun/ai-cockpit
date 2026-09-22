---
author: AI Cockpit maintainers
title: "WI-989 — WI-988 terminal documentation projection"
description: "WI-988 の verification 済み close 後に、範囲を限定した terminal documentation projection を完了する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-989-wi988-doc-promotion
lastVerifiedBy: WI-989-wi988-doc-promotion
---

[English](WI-989-wi988-doc-promotion.md) · [简体中文](WI-989-wi988-doc-promotion.zh-CN.md)

# WI-989 — WI-988 terminal documentation projection

WI-989 は verification 済みで close された WI-988 projection の bounded な
documentation successor である。WI-988 の immutable archive、verification、close
の事実を記録し、source behavior や既存 lifecycle receipt は再び開かない。

## Boundary

対象はこの Work Item に必要な三言語ページと三つの reference-parity ledger row に
限定する。文書の対象は WI-988 の terminal projection のみであり、source code、release
behavior、lifecycle receipt は対象外である。

## Acceptance

- WI-988 のページと parity row が immutable evidence に基づき terminal に昇格される。
- 三言語の WI-989 ページと parity row が自身の bounded projection を保持する。
- 単体 projection、repository-wide projection、parity、status の検査が通る。
