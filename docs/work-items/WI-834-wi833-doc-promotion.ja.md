---
author: AI Cockpit maintainers
title: "WI-834 — WI-833 ドキュメント promotion"
description: "完了した WI-833 lifecycle の証拠バインドされた閲覧用ドキュメントを登録します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-834-wi833-doc-promotion
lastVerifiedBy: WI-834-wi833-doc-promotion
---

[English](WI-834-wi833-doc-promotion.md) · [简体中文](WI-834-wi833-doc-promotion.zh-CN.md)

# WI-834 — WI-833 ドキュメント promotion

## 意図と境界

WI-834 は、完了した WI-833 release-script provenance Work Item の閲覧用ドキュメントと
reference-parity 行を登録します。この projection は Runtime が管理する正確な archive、
verification、finalization、close record を参照し、過去のバイト列を変更しません。

Runtime の動作、release artifact、過去の Work Item、無関係な source verification は対象外です。

## Acceptance

- English、簡体中文、日本語の WI-834 ページが存在し、同じ Work Item identity を参照する。
- 各 parity ledger に verification 前の WI-834 pre-archive 行が一つだけ存在する。
- WI-833 promotion check と repository-wide documentation check が pass する。
- close 後の promotion は同じ行に terminal evidence を追加し、WI-833 の履歴を変更しない。

## 検証

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`

