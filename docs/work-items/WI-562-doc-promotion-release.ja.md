---
author: AI Cockpit maintainers
title: "WI-562 — WI-561 の終端ドキュメント昇格"
description: "クローズ済み WI-561 のリリース文書投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-562-doc-promotion-release
lastVerifiedBy: WI-562-doc-promotion-release
---

[English](WI-562-doc-promotion-release.md) · [简体中文](WI-562-doc-promotion-release.zh-CN.md)

# WI-562 — WI-561 の終端ドキュメント昇格

## 目的

不変の終端記録だけを使い、検証済みでクローズされた WI-561 の三言語
Work Item ページと reference-parity 投影を昇格する。

## 範囲と境界

対象は WI-561 の三言語ページ、対応する三つの reference-parity ページ、
および本 Work Item の三言語ページに限定する。終端状態を書き込むのは
closed Work Item promotion helper のみとする。Runtime 動作、対象リポジトリ、
グローバル Agent/MCP 設定、ソース在庫の意味、無関係な文書は対象外である。

## 受け入れ条件

- WI-561 の全投影が不変の archive、verification、finalization、close 証拠を参照し、統治事実を変更しない。
- 本 Work Item 自身の検証済みクローズ前の状態を三言語 parity ページへ登録する。
- closed Work Item チェック、ドキュメント受け入れ、parity gate、宣言済み検証コマンドがすべて成功する。
- 不変の receipt や無関係な投影を変更しない。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
