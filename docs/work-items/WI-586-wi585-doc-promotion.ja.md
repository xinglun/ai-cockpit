---
author: AI Cockpit maintainers
title: "WI-586 — WI-585 終端ドキュメント投影"
description: "WI-585 の close 後に三言語ドキュメント投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-586-wi585-doc-promotion
lastVerifiedBy: WI-586-wi585-doc-promotion
---

[English](WI-586-wi585-doc-promotion.md) · [简体中文](WI-586-wi585-doc-promotion.zh-CN.md)

# WI-586 — WI-585 終端ドキュメント投影

## 目的

WI-585 の immutable archive、verification、finalization、close receipt が有効に
なった後だけ、三言語の Work Item と reference-parity 投影を昇格する。本 Work
Item はドキュメント投影だけを変更し、治理事実や Runtime の挙動は変更しない。

## 境界

Object repository、Runtime 実装、グローバル Agent/MCP 設定、生成済み
evidence/decision bytes は対象外とする。Contract の acceptance は原文言語を
権威として保持する。

## 受入れ

1. 三つの WI-585 Work Item ページに immutable receipt から導出した終端パスがある。
2. 三言語の reference-parity 行が WI-585 を Implemented とし、evidence パスを
   バインドする。
3. Governance の事実、source 実装、object repository、生成 receipt bytes を変更しない。

## 検証

明示的な repository context で `tests/docs/documentation_acceptance.sh` と、現在の
Contract が宣言する Runtime 検証コマンドを実行する。
