---
author: AI Cockpit maintainers
title: "WI-1085 — 旧 v1 closeout の明示的な現在の出典登録"
description: "Present-day explicit attestation with preserved legacy history and bounded recovery."
audience:
  - maintainer
  - reviewer
workItemId: WI-1085-legacy-v1-closeout-source-registration
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1085-legacy-v1-closeout-source-registration
---

[English](WI-1085-legacy-v1-closeout-source-registration.md) · [简体中文](WI-1085-legacy-v1-closeout-source-registration.zh-CN.md)

# WI-1085 — 旧 v1 closeout の明示的な現在の出典登録

本 Work Item は進行中です。個別回帰と独立レビューを保存していますが、正式 Runtime 検証、受け入れ、終端 receipt は未完了です。lastVerifiedBy は現在の作業投影であり、検証済み close を示しません。

## 期待する効果と境界

保守者は独立 checkout と固定した完全 commit を使い、完全な旧 v1 closeout の現在の観測を明示登録できます。現在の登録者が確認した bytes を attestation として記録し、旧 producer、時刻、Contract、archive、verification、provider の原文を保持します。過去の証拠を現在の verified 実行に昇格しません。

## 実装

不変 requirement は最初の正確な registration template を保存し、registration は完全で有界な raw/canonical inventory を束縛します。同一 WI lifecycle lock が no-replace 公開を直列化します。root/extension resume は独立保存した元の bytes と digest を必要とします。root の正確な復元後も extension が不完全なら pending のままで、未束縛 transition を受け入れません。正当な Retained→Deleted append は明示 record と通常 finalize から同じ持 lock core を使います。

壊れた canonical 名や ASCII 大文字小文字 alias も義務として残し、依存 query の循環・深さ・仕事量を制限します。必須証拠の破損は closed と scope 解放を撤回します。完全登録後に元の出典 checkout の常時オンラインを要求せず、明示的な外部依存の失敗条件は保持します。全登録義務と独立に利用できる登録 anchor が一緒に消失・巻き戻された場合、過去の登録は推定できません。遠隔の単調 registry は導入しません。

## 証拠と残作業

[実装 ledger](../../.ai/evidence/WI-1085-legacy-v1-closeout-source-registration/implementation-20261015/ledger.md) に実際の RED/GREEN、compile failure、公開 subprocess test、file binding、資源観測、レビュー結果を保存しています。crash matrix は公開 primitive を、完全履歴 API の partial-state 回復は別の integration fixture を検証します。どちらも正式 Runtime receipt ではありません。

レビュー修正と三言語 commands/workflow を完了し、必須 parity 行の scope admission、material review、controls を準備してから、単一 build job、隔離 target、外側 900 秒上限で宣言済み workspace 検証を実行します。本 Work Item は PR、Ready、merge、人間 close、release、cleanup を許可しません。
