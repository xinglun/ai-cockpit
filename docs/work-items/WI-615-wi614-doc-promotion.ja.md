---
title: "WI-615 — WI-614 終端ドキュメント昇格"
description: "WI-614 の不変なガバナンス記録を変更せず、検証済み終端ドキュメント投影を昇格する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-615-wi614-doc-promotion
workItemId: WI-615-wi614-doc-promotion
predecessorWorkItemId: WI-614-direct-merge-no-pr-apply
---

[English](WI-615-wi614-doc-promotion.md) · [简体中文](WI-615-wi614-doc-promotion.zh-CN.md)

# WI-615 — WI-614 終端ドキュメント昇格

## 目的

英語・簡体字中国語・日本語の WI-614 ページと reference-parity ledger を、一時的な In progress 投影から検証済み Implemented 状態へ昇格する。昇格元は WI-614 の終端 archive、verification、finalization、close receipt とし、Runtime が生成した記録は不変のまま保持する。

## 境界

この Work Item はドキュメント投影だけを変更する。Runtime、protocol、reference fixture、release artifact、adopter repository、グローバル Agent/MCP 設定、WI-614 のガバナンス記録は変更しない。

## 検証

close 前に documentation acceptance、parity status、reference metadata、governance integrity、conformance、closed Work Item promotion check を通過させる。
