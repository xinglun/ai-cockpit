---
author: AI Cockpit maintainers
title: "WI-566 — WI-565 ドキュメント projection の昇格"
description: "検証・close 済み WI-565 の三言語 page を昇格し、この限定的な Work Item を登録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-566-documentation-promotion
lastVerifiedBy: WI-566-documentation-promotion
---

[English](WI-566-documentation-promotion.md) · [简体中文](WI-566-documentation-promotion.zh-CN.md)

# WI-566 — WI-565 ドキュメント projection の昇格

## 目的

検証・close 済みの WI-565 release に対応する三言語 documentation projection
を昇格し、この documentation Work Item 自身も同じ projection に登録する。
immutable な Runtime evidence は参照するだけで、書き換えない。

## 範囲と境界

範囲は WI-565 の三言語 page、WI-566 の三言語 page、三つの
reference-parity page、および closed Work Item promotion helper に限定する。
Runtime behavior、release artifact、object repository、global Agent/MCP 設定、
immutable な Contract/evidence/decision/archive bytes は対象外とする。

## 受入条件

- 三言語の WI-565 page が `implemented` となり、archive、verification、
  finalization、close evidence を参照する。
- 三つの parity page が WI-565 を Implemented とし、WI-566 の限定的な
  pre-archive projection を登録する。
- documentation、parity、promotion、diff の check が通り、historical
  governance record を書き換えない。
- WI-566 自身に三言語の readable page と一致する pre-archive parity 登録がある。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

