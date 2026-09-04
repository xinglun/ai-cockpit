---
author: AI Cockpit maintainers
title: "WI-575 — WI-574 の終端ドキュメント昇格"
description: "不変のガバナンス記録を書き換えずに、終了済み WI-574 のリリース文書を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-575-wi574-doc-promotion
lastVerifiedBy: WI-575-wi574-doc-promotion
---

[English](WI-575-wi574-doc-promotion.md) · [简体中文](WI-575-wi574-doc-promotion.zh-CN.md)

# WI-575 — WI-574 の終端ドキュメント昇格

## 目的

WI-574 の verified-close リリース文書を終端状態へ昇格し、このドキュメント
投影を三言語の parity マトリクスに登録する。不変のガバナンス記録は変更しない。

## 範囲と境界

範囲は WI-574 の三つのページ、WI-575 の三つのページ、三つの
reference-parity ページである。Runtime の動作、リリース成果物、対象リポジトリ、
グローバル Agent/MCP 設定、過去のガバナンスバイトは範囲外とする。

## 受入れ

- 三つの WI-574 ページが `implemented` となり、archive、verification、
  finalization、close の証拠へリンクする。
- 三つの parity ページが WI-574 を実装済みと示し、WI-575 の限定された終端投影を
  証拠パス付きで登録する。
- 不変のガバナンス記録を書き換えずに、ドキュメント、parity、promotion、
  状態整合性、ガバナンス完全性、diff のチェックが通る。
- WI-575 に英語、簡体字中国語、日本語の対応する読みやすいページがある。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh .`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report /tmp/wi575-governance-report.json`
- `git diff --check`
