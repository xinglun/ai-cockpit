---
author: AI Cockpit maintainers
title: "WI-588 — WI-587 terminal documentation promotion"
description: "WI-587 close 後に三言語の documentation projection を terminal に昇格します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-588-wi587-doc-promotion
lastVerifiedBy: WI-588-wi587-doc-promotion
---

[English](WI-588-wi587-doc-promotion.md) · [简体中文](WI-588-wi587-doc-promotion.zh-CN.md)

# WI-588 — WI-587 terminal documentation promotion

## Objective

WI-587 の immutable archive、verification、finalization、close receipt が有効に
なった後、三言語の Work Item と reference-parity projection を terminal にします。
本 WI は documentation projection だけを変更します。

## Boundary

Runtime behavior、object repository、global Agent/MCP configuration、生成済み
evidence/decision bytes は対象外です。Contract acceptance は authoring language
を authoritative とします。

## Acceptance

1. 三つの WI-587 Work Item page が immutable receipt から導出した terminal path を含む。
2. 三つの reference-parity row が WI-587 を matching terminal evidence path 付きで Implemented と報告する。
3. Governance facts、source implementation、object repository、generated receipt bytes を変更しない。

## Verification

明示的な repository context で `tests/docs/promote_closed_work_item.py --check`、
`tests/docs/documentation_acceptance.sh`、`tests/docs/parity_status_check.sh` を実行します。
