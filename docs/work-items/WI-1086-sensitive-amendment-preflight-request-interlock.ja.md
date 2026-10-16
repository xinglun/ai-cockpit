---
author: AI Cockpit maintainers
title: "WI-1086 — 機微な修訂後の preflight レビュー要求の相互待ち"
description: "現在の要求を生成し、レシートの整合性と既存の阻害要因を保持する。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1086-sensitive-amendment-preflight-request-interlock
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1086-sensitive-amendment-preflight-request-interlock
---

[English](WI-1086-sensitive-amendment-preflight-request-interlock.md) · [简体中文](WI-1086-sensitive-amendment-preflight-request-interlock.zh-CN.md)

# WI-1086 — 機微な修訂後の preflight レビュー要求の相互待ち

本 Work Item は進行中です。Runtime は 2026-10-16 に通常の入口から before-edit checkpoint を作成しました。テスト、実装、正式検証、受け入れは未完了です。lastVerifiedBy は現在の文書投影を示し、検証済みの終了を意味しません。

## Benefit

開発者は機微な修訂後、確認を待つ前に現在のレビュー要求を取得できます。元の阻害証拠と、identity、Contract、decision、snapshot の厳密なチェックを保持します。

## Design and acceptance

コミット f49a001ece01209ed9e4cf743f879b7a5931143c の v2 計画は Raydot の技術レビュー済みです。完全な native journal と現在の無効化 binding、要求生成の admission、state/outcome/blockers の保持に限定します。CLI/MCP で documentation の先行エラーと governance_red を再現します。追加のレシート欠陥には具体的な RED と最小範囲の再レビューが必要です。

## Execution boundary

直列のみ：worker は一つ、target/wi1086-sensitive-amendment-preflight-request-interlock を分離し、各コマンドは最大 900 秒です。元の WI-1085 を保持し、将来この修復を利用する側として扱います。共有ソース、台帳、ビルド、lifecycle 操作は並行実行しません。lease や人の確認を偽造せず、Runtime のインストール、Sentinel 操作、merge、close、cleanup は行いません。
