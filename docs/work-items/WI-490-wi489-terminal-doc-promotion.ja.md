---
author: AI Cockpit maintainers
title: "WI-490 — WI-489 終端ドキュメント投影"
description: "有界な WI-489 ドキュメント投影を昇格し、post-close ドキュメントゲートの再帰を終端させる。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-490-wi489-terminal-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-490-wi489-terminal-doc-promotion
---

# WI-490 — WI-489 終端ドキュメント投影

この限定的なドキュメント Work Item は、WI-489 の不変な終端証拠を使って
三言語のページと parity 登録を昇格します。Runtime の動作、履歴証拠、
グローバル Agent/MCP 設定は変更せず、post-close ドキュメント投影を終端させます。

[English](WI-490-wi489-terminal-doc-promotion.md) · [简体中文](WI-490-wi489-terminal-doc-promotion.zh-CN.md)

## スコープ

- 三つの WI-489 Work Item ページを終端証拠付きメタデータへ昇格する。
- 三つの WI-489 parity 行を昇格し、archive、verification、finalization、close を参照する。
- 本 Work Item 自身のページと parity 登録を同じ有界投影に含め、終端チェッカーの再帰を防ぐ。

## 受入れ

- 六つの WI-489 投影ページ/行を、本文と不変なガバナンス記録を変更せずに昇格する。
- post-close 昇格チェッカーと status-consistency チェッカーが、この正確なドキュメント範囲を自己終端として認識する。
- 英語・中国語・日本語のドキュメント検査が成功し、グローバル Agent/MCP 設定を変更しない。

## 検証

- `bash tests/docs/promote_closed_work_item_test.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `python3 tests/conformance/reference_file_inventory.py --check`
