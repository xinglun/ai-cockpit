---
author: AI Cockpit メンテナー
title: WI-1032 — Work Item 横断レビューの修正
description: identity、admission、recovery、reuse、MCP contract に残る不足を解消する。
workItemId: WI-1032-cross-wi-review-fixes
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-1032-cross-wi-review-fixes
---

# WI-1032 — Work Item 横断レビューの修正

## 意図

Work Item 横断の協調機能に対する独立レビューの限定的な後続修正を完了します。この projection は実施中であり、完全な検証、受け入れ、リリース準備完了を示しません。

## 範囲

- registration、composition admission、evidence、target merge、resource generation、Outcome state を観測済み repository facts に結び付ける。
- invalidation 登録を復旧可能にし、append-only event history を保持し、選択された provider/outcome pair に基づいて action admission を絞る。
- executable、限定された完全な read set、environment、upstream receipt がすべて観測上不変の場合だけ composition 結果を再利用し、それ以外は再実行する。
- MCP の各 coordination action に provider、Work Item、event、consumer identity の一意な意味を定義する。
- query は read-only、write は明示的にし、通常の単一 WI による逐次 verification を維持する。
- Sentinel では候補 Runtime の read-only compatibility check のみ行う。Sentinel 側の実装変更は後続の独立 WI に分ける。

## 対象外

Release の準備・公開、tag 変更、Runtime の install/upgrade、この WI での Sentinel source/lifecycle への write、global Agent/MCP 設定、協調アーキテクチャ全体の再設計。

## 受け入れ条件

1. registration と dependency admission は検証済み repository、Contract、branch/head、evidence、generation facts に基づき、安全 pause 中は composition を開始しない。
2. 必須 participant/check と実際の target merge state を検証し、attempt/cleanup を復旧可能にし、完全に観測した identity だけで node reuse を決める。
3. interruption、generation、provider/outcome pair、推移的 consumer をまたいで impact report、recovery、action admission が一貫する。
4. MCP schema と handler は各 action の identity を一意に定義し、CLI/MCP parity と canonical gate で保護する。
5. read-only inspection は協調状態を永続化せず、impact/publication/coordination/recovery は明示的な write entry point を使う。
6. 実プロセス・linked-worktree 受け入れ、通常の逐次経路、canonical verification、独立 PR review、merge、許可済み cleanup を完了してから WI を close する。Release 前で停止する。

## 証拠と現在の状態

Task 1–7 は逐次の個別 commit として実装済みです。composition、coordination store、CLI/MCP の query-write 境界テストに加え、gate-manifest 回帰、release CLI build、documentation acceptance、実プロセス受け入れがローカルで通過しました。実受け入れでは 2 つの linked worktree を使用し、同一 composition の process 数は 1 → 0、JSON bytes を維持したまま実際に観測した environment input を変更すると 1 に戻りました。query test は store 不在/既存の両方で coordination directory の path と byte を比較します。明示的な registration と impact report だけが記録を永続化し、新しい MCP process がその記録を読み取ります。Runtime-bound の全体 verification、Sentinel read-only compatibility check、独立 PR review、merge、正確な cleanup は未完了です。これらはローカル開発 evidence であり、Runtime-bound verification や Release 承認ではありません。[仕様](WI-1032-cross-wi-review-fixes/spec.md)、[実施計画](WI-1032-cross-wi-review-fixes/implementation-plan.md)、[独立レビュー](WI-1032-cross-wi-review-fixes/independent-review.md)を参照してください。
