---
author: AI Cockpit maintainers
title: "WI-774 — WI-773 terminal documentation promotion"
description: "close 済み WI-773 experiment を必要な三言語 documentation と parity boundary に反映する。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: authorized
workItemId: WI-774-wi773-doc-promotion
lastVerifiedBy: WI-774-wi773-doc-promotion
---

[English](WI-774-wi773-doc-promotion.md) · [简体中文](WI-774-wi773-doc-promotion.zh-CN.md)

# WI-774 — WI-773 terminal documentation promotion

## Intent と boundary

WI-774 は immutable な documentation delivery failure として保持する。hosted PR #757 は
parity registration と verification evidence が同一 commit で導入されたため
`docs_governance_integrity` に失敗した。WI-775 が最新 default branch からこの projection を
ordered に再配信する bounded successor である。WI-773 の immutable Contract、benchmark
evidence、Outcome、finalization、close、candidate 却下判断は変更しない。

## Scope

- WI-774 の archive、verification evidence、recovery decision、hosted failure binding を
  保持し、historical bytes は書き換えない。
- ordered redelivery は WI-775 が担当し、parity registration を fresh verification evidence
  より先に commit する。
- Runtime-generated record、historical evidence、governance rule、他の Work Item は変更しない。

## Immutable recovery binding

- archive: `.ai/work-items/archive/WI-774-wi773-doc-promotion.archive.json`
- verification: `.ai/evidence/WI-774-wi773-doc-promotion.verification.json`
- recovery: `.ai/decisions/WI-774-wi773-doc-promotion.recovery.json`
- failed delivery: [PR #757](https://github.com/xinglun/ai-cockpit/pull/757)
- successor: WI-775 parity-order recovery

## Acceptance と verification boundary

- `promote_closed_work_item.py --check` が WI-773 を current と報告する。
- 三言語 parity、documentation acceptance、Work Item status consistency check が pass する。
- finish、archive、review、merge、finalization、close の前に、Runtime verification が current
  Contract と宣言済み documentation check に bind される。
