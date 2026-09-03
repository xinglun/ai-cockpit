---
author: AI Cockpit maintainers
title: "WI-525 — WI-524 terminal documentation promotion"
description: "正確な終端証跡バインディングで WI-524 の完了済みドキュメント投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-525-wi524-doc-promotion
lastVerifiedBy: WI-525-wi524-doc-promotion
---

[English](WI-525-wi524-doc-promotion.md) · [简体中文](WI-525-wi524-doc-promotion.zh-CN.md)

## 目的

三言語の WI-524 Work Item ページと parity 行を、不変の archive、verification、
resource-finalization、close 証跡に同期する。

## 範囲

- WI-524 ページと三つの reference-parity 投影を昇格する。
- 履歴証跡バイト列、Runtime 動作、対象リポジトリ、グローバル Agent/MCP 設定を変更しない。
- 終端 close 後もこの投影を監査可能に保つ。

## 受入れ

- すべての WI-524 ページと parity 行が正確な終端証跡パスを参照する。
- 完了 Work Item の昇格、ドキュメント、ガバナンスゲートが成功する。
- 対象リポジトリの状態と履歴証跡を変更しない。

## 検証

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
