---
author: AI Cockpit maintainers
title: "WI-567 — WI-566 終端ドキュメント昇格"
description: "close 済み WI-566 の documentation projection を昇格し、immutable な governance record を変更しない。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-567-wi566-doc-promotion
lastVerifiedBy: WI-567-wi566-doc-promotion
---

[English](WI-567-wi566-doc-promotion.md) · [简体中文](WI-567-wi566-doc-promotion.zh-CN.md)

# WI-567 — WI-566 終端ドキュメント昇格

## 目的

検証・close 済み WI-566 の三言語 page を昇格し、三言語の reference matrix に
この限定的な昇格を登録する。immutable な Contract、evidence、decision、archive
record は変更しない。

## 範囲と境界

範囲は WI-566 の三言語 page、WI-567 の三言語 page、および三つの
reference-parity page に限定する。Runtime behavior、release artifact、object
repository、global Agent/MCP 設定、historical governance bytes は対象外とする。

## 受入条件

- 三言語の WI-566 page が `implemented` となり、archive、verification、
  finalization、close evidence を参照する。
- 三つの parity page が WI-566 を Implemented とし、WI-567 の限定的な
  pre-archive projection を登録する。
- documentation、parity、promotion、diff check が通り、immutable な
  governance record を書き換えない。
- WI-567 自身に readable な三言語 page と一致する pre-archive parity 登録がある。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --work-item WI-566-documentation-promotion`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

