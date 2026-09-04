---
author: AI Cockpit maintainers
title: "WI-573 — WI-572 の終端ドキュメント昇格"
description: "不変のガバナンス記録を書き換えずに、終了済み WI-572 のドキュメント投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-573-wi572-doc-promotion
lastVerifiedBy: WI-573-wi572-doc-promotion
---

[English](WI-573-wi572-doc-promotion.md) · [简体中文](WI-573-wi572-doc-promotion.zh-CN.md)

# WI-573 — WI-572 の終端ドキュメント昇格

## 目的

WI-572 の verified-close ドキュメントを終端状態へ昇格し、このドキュメント
投影を三言語の parity マトリクスに登録する。不変のガバナンス記録は変更しない。

## 範囲と境界

範囲は WI-572 の三つのページ、WI-573 の三つのページ、三つの
reference-parity ページである。Runtime の動作、リリース成果物、対象リポジトリ、
グローバル Agent/MCP 設定、過去のガバナンスバイトは範囲外とする。

## 受入れ

- 三つの WI-572 ページが `implemented` となり、archive、verification、
  finalization、close の証拠へリンクする。
- 三つの parity ページが WI-572 を実装済みと示し、WI-573 の限定された終端投影を
  証拠パス付きで登録する。
- 不変のガバナンス記録を書き換えずに、ドキュメント、parity、promotion、diff の
  チェックが通る。
- WI-573 に英語、簡体字中国語、日本語の対応する読みやすいページがある。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `git diff --check`
