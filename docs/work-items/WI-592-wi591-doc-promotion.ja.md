---
author: AI Cockpit maintainers
title: "WI-592 — WI-591 terminal documentation promotion"
description: "検証済み WI-591 release projection を昇格し、後続 parity recovery を不変履歴として保持します。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: canonical
workItemId: WI-592-wi591-doc-promotion
lastVerifiedBy: WI-592-wi591-doc-promotion
---

[English](WI-592-wi591-doc-promotion.md) · [简体中文](WI-592-wi591-doc-promotion.zh-CN.md)

# WI-592 — WI-591 terminal documentation promotion

## Objective

WI-591 の immutable archive と verification evidence を記録した後、英語・中国語・
日本語の release と reference-parity projection を昇格します。後続 CI が WI-592
自身の parity 登録漏れを検出した事実は不変履歴として保持し、successor WI-593
で再配信します。

## Boundary

本記録は documentation projection のみを扱います。Runtime behavior、release
artifact、object repository、global Agent/MCP configuration、生成済みの
archive/evidence/decision bytes は範囲外で、変更しません。

## Acceptance

1. WI-591 の release documentation が三言語で terminal Runtime evidence から
   一貫して昇格されること。
2. WI-592 の archive bytes を書き換えず、recovery boundary と successor WI-593
   を記録すること。
3. Documentation gate が再現可能で、parity 登録漏れを黙って履歴変更せず、有界の
   successor task として報告すること。

## Verification

明示的な repository context で `python3 tests/docs/promote_closed_work_item.py
--repo <repository> --check-all`、`tests/docs/documentation_acceptance.sh`、
`tests/docs/parity_status_check.sh` を実行します。
