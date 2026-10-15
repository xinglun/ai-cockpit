---
author: AI Cockpit maintainers
title: "WI-1084 — checkout 間 recovery receipt の完全性"
description: "履歴 close の元バイトを保持し、必須 recovery receipt の消失を検出する実装計画。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1084-cross-checkout-closeout-receipt-integrity
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1084-cross-checkout-closeout-receipt-integrity
terminalArchive: .ai/work-items/archive/WI-1084-cross-checkout-closeout-receipt-integrity.contract.json
terminalVerification: .ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity.verification.json
terminalDecision: .ai/decisions/WI-1084-cross-checkout-closeout-receipt-integrity.close.json
contractDigest: sha256:507bedef85c7d3cb7ff2b8fd3a885d5441384ea23b8d6e4b010fa9be8c150107
---

[English](WI-1084-cross-checkout-closeout-receipt-integrity.md) · [简体中文](WI-1084-cross-checkout-closeout-receipt-integrity.zh-CN.md)

# WI-1084 — checkout 間 recovery receipt の完全性

Ray が結合 Contract/preflight/snapshot を確認し、Native が confirm_review を記録しました。receipt 単独削除と回復後の Retained close→Deleted transition の両 RED を保持。有界修正は18共有 unit と14 repository/CLI/MCP integration、厳格 workspace clippy と format check に合格。現在の正式証拠と残る受入は Native に従い、lastVerifiedBy は進行中 WI の投影先であり検証済み close を主張しません。

## 想定する効果と観測済みの問題

独立 checkout の RED は誤った closed 投影を再現し、GREEN と後続 P2 組合せ matrix を保持しています。元 Sentinel 受入履歴はそのままです。通常の次の Deleted transition は完全な既存規則の検証後に不変の履歴 close を有効なまま保持します。

## 最小設計

[最終計画](../../.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/final-plan-20261015/implementation-plan.md)が予定形式を定義します。同じ WI の不変 origin（schema1）は repository/WI、canonical Contract、source snapshot、provider/Runtime の事実、元 close と全 raw payload の一覧を束縛し、必須 recovery receipt の固定パスと version2 を指定します。version2 receipt は origin の実際の digest を一方向に参照し、内容ハッシュの循環を作りません。必須レコードの欠落・不一致・誤った版を ordinary close や v1 への fallback で許容しません。

既存の lifecycle lock と不変 installer を再利用し、履歴 payload、origin、receipt、元 close marker の順で配置し、marker 直前に全必須バイトを再確認します。中断時は公開済みパスを保持し、競合する再試行で上書きしません。復元済み checkout を次の source に使う場合も共有 validator を先に通します。origin に変動する時刻を含めず、receipt の作成メタデータを正確に記録し、新しい TTL を導入しません。

通常の現在/旧履歴 close は互換性を維持します。既存 v1 を書き換えたり暗黙に移行したりしません。独立した来歴束縛以前に旧唯一 receipt が失われた履歴は推定できず、全来歴の同時削除を一般的に検出するとも主張しません。

## RED と受け入れ計画

1. 独立した Git common-dir で正常に復元し、closed の有効な基準を確認後、receipt だけを削除します。照会の終了0を許容しつつ `archived`、`blocking=true` と完全性診断を要求し、本番修正前の失敗を観測します。
2. 重複 scope の作業が誤って許可されないこと、origin/receipt の欠落・不正・非通常ファイル、誤った版/パス/digest/identity、close/payload 不一致と archive 完全性を確認します。
3. 既存の private hook で origin 後、receipt 後、close 前後の中断を扱い、source と公開済みパスの保持、サポートされた復元/冪等性、復元済み checkout の再 source 化を証明します。
4. ordinary/v1 互換性と履歴 `not_ready`、`historical_evidence_not_revalidated` を保持し、狭い CLI/MCP 同等性の回帰を追加します。
5. 実際の policyReview と新しい admission 後に focused RED/GREEN、`cargo test --locked --workspace`、必須 repository gate を実行します。900秒上限、独立 target、単一 build task と元出力を保持します。現在の正式証拠は Native receipt に従い、終端受入の合格は主張しません。

## 実行境界

WI-1084 は単一 worker の直列実行です。WI-1070～1074 に boundary/declaration がなく、正式な並列互換性は false のままで、それらの記録は変更しません。出力は `target/wi1084-closeout-receipt-integrity/` に隔離し、checkpointCount は1のままです。PR#1040 は Draft、統合は直列とします。Sentinel の業務変更、WI-1078 resource の変更、Task9、TTL、機構再設計、release、新 WI の人による close、merge、削除は対象外です。

現在の Native admission に従い正式検証と Raydot review を進めます。[保持した RED/GREEN 証拠](../../.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/p2-transition-20261015/red-summary.md)を参照。同じ Contract の再確認は不要です。PR#1040 は Draft を維持し、この承認は merge・close・resource 削除を含みません。
