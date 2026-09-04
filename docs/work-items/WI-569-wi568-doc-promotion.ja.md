---
author: AI Cockpit maintainers
title: "WI-569 — WI-568 の終端ドキュメント昇格"
description: "不変のガバナンス記録を書き換えず、close 済み WI-568 の文書投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-569-wi568-doc-promotion
lastVerifiedBy: WI-569-wi568-doc-promotion
---

[English](WI-569-wi568-doc-promotion.md) · [简体中文](WI-569-wi568-doc-promotion.zh-CN.md)

# WI-569 — WI-568 の終端ドキュメント昇格

## 目的

検証済み close の WI-568 文書を昇格し、archive、evidence、finalization、close
の参照を三言語 parity matrix に保持する。不変の記録は変更しない。

## 範囲と境界

WI-568 三言語ページ、WI-569 三言語ページ、三つの reference-parity ページだけを
対象とする。Runtime、release、adopter repository、global Agent/MCP 設定、過去の
ガバナンス bytes は対象外。

## 受入れ

- WI-568 の三言語ページが `implemented` となり、終端 evidence を参照する。
- parity ページが WI-568 を実装済みとして WI-569 の pre-archive 投影を登録する。
- 文書、parity、promotion、diff の検査が不変記録を書き換えずに通る。
- WI-569 に可読な三言語ページと一致する pre-archive parity 登録がある。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
