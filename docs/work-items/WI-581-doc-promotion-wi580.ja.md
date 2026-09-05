---
author: AI Cockpit maintainers
title: "WI-581 — WI-580 終端ドキュメント投影"
description: "不変のガバナンス証跡からクローズ済み WI-580 のドキュメント投影を生成する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-581-doc-promotion-wi580
lastVerifiedBy: WI-581-doc-promotion-wi580
terminalArchive: .ai/work-items/archive/WI-581-doc-promotion-wi580.contract.json
terminalVerification: .ai/evidence/WI-581-doc-promotion-wi580.verification.json
terminalFinalization: .ai/decisions/WI-581-doc-promotion-wi580.finalize.2c045b801ad0a39909547eeed34d24da608fa121228b58ffa932079a6461b235.json
terminalDecision: .ai/decisions/WI-581-doc-promotion-wi580.close.json
---

[English](WI-581-doc-promotion-wi580.md) · [简体中文](WI-581-doc-promotion-wi580.zh-CN.md)

# WI-581 — WI-580 終端ドキュメント投影

## 目的

クローズ済み WI-580 の不変アーカイブ、検証証跡、リソース終端チェーン、
クローズ決定から、三言語の人間向けドキュメントを投影する。本ページは
投影であり、元の記録を置き換えない。

## 証跡の境界

Runtime が生成した記録を権威とする。投影は三言語の表示を同期するだけで、
Contract、検証結果、リソース終端履歴、または人間の決定を変更しない。

## 終端証跡

閉じた証跡を `tests/docs/promote_closed_work_item.py` が検証した後、終端
リンクが決定的に追加される。
