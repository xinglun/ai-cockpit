---
author: AI Cockpit maintainers
title: "WI-560 — WI-559 の終端ドキュメント投影"
description: "クローズ済み WI-559 のドキュメント投影を昇格し、この有界な自己投影を登録する。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-560-doc-promotion-wi559
lastVerifiedBy: WI-560-doc-promotion-wi559
terminalArchive: .ai/work-items/archive/WI-560-doc-promotion-wi559.contract.json
terminalVerification: .ai/evidence/WI-560-doc-promotion-wi559.verification.json
terminalFinalization: .ai/decisions/WI-560-doc-promotion-wi559.finalize.json
terminalDecision: .ai/decisions/WI-560-doc-promotion-wi559.close.json
---

[English](WI-560-doc-promotion-wi559.md) · [简体中文](WI-560-doc-promotion-wi559.zh-CN.md)

# WI-560 — WI-559 の終端ドキュメント投影

## 目的

不変の終端記録だけを用いて、WI-559 の三言語 Work Item ページと
reference-parity 投影を昇格する。

## 範囲と境界

範囲は WI-559 の三言語ページ、対応する三つの reference-parity ページ、
およびこの有界な自己投影の三言語ページに限定する。終端状態を書き込むのは
昇格ヘルパーだけである。Runtime 挙動、object repository、global Agent/MCP
設定、source inventory の意味、無関係なドキュメントは変更しない。

## 受入れ

- WI-559 の投影が governance facts を変更せず、終端 archive、verification、
  finalization、close を参照する。
- この Work Item 自身が verified close するまで、必要な pre-archive 状態で
  三言語の parity ページに登録される。
- closed Work Item promotion check、documentation acceptance、parity gate、
  宣言した検証コマンドがすべて成功する。
- 不変 receipt または無関係な投影を変更しない。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
