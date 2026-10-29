---
author: AI Cockpit maintainers
title: "WI-1111-token-outcome-ci — Usage intake and Outcome parity"
description: "実装・アーカイブ済み。ネイティブの人手による close と後続の納品は未完了です。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1111-token-outcome-ci
status: implemented
authority: native-contract
lastVerifiedBy: WI-1111-token-outcome-ci
---

# WI-1111-token-outcome-ci — 使用量の取込み、Outcome の整合と有限 CI 高速化

実装とローカル検証は記録済みです。ネイティブ Work Item は 2026-10-26 にアーカイブされ、現在は人手による個別 close 待ちの yellow 状態です。このページは close、公開納品、ホスト CI、測定済みの性能効果、または本作業の実使用量を完了したとは主張しません。

## 実装済み・ローカル検証済み

- ネイティブ使用量取込みは、ソース ID、不変レシート、nullable カウンター、呼出元の出典を保持します。現在のセッション使用量が取得できない場合や形式が未対応の場合は unknown のままにし、累積 Codex JSONL を呼出ごとの使用量として扱いません。
- JSON、Markdown、CLI、MCP の Outcome 表示は、日本語・英語・中国語で既知／unknown の使用量を一貫して保ちます。finish/archive のバイト列は凍結され、close には別の cutoff があります。
- レポート投影は型付きポリシー、producer、描画成果物に束縛されます。固定された証拠で裏付けられる有限ルールで旧形式を読み、履歴バイト列は書き換えません。
- ローカルテストでは、安価な事前チェック、有限な Windows コマンド分割、安定した集約ゲート、パッケージ網羅性、警告ゼロ検査を扱います。
- ネイティブ検証レシートは 54/54 件成功、54 件を新規実行、再利用 0、worker 1、各検査の上限 900 秒、timeout なし、経過時間 24 分 15 秒を記録しています。アーカイブ Summary は 13/13 シナリオを verified としています。固定 Runtime は `sha256:134567563cc90a9a20744f075952eae6812b21c7f606ae3722d49ae249b0bd5b` で、source、Contract、snapshot の束縛はアーカイブ記録にあります。

## 証拠の限界と close 前の確認事項

- アーカイブ Outcome は 16 件の acceptance result を持ちますが、Summary の `acceptanceEvidence` は 14 件です。read-only archive validator は `acceptanceEvidence: verified` を返しますが、実際の行は未検証と記された plan 宣言で、移動済みの旧 active Contract パスを参照しています。`/acceptanceCriteria/13` と `/acceptanceCriteria/14` の項目はありません。シナリオのテキストは attempt `8c1471fc…` と receipt digest `6dd8ccc4…` を参照する一方、現在の verification record は attempt `9ecb62e3…` と receipt digest `c32b0b13…` を示します。54 件のレシートやシナリオ状態を 16 項目すべての一対一の証明とみなす前、また人手による close 前に、ネイティブ参照を照合してください。
- 成功した repository テストノードの stdout は Runtime の取得上限で切り詰められました。警告ゼロのゲート自体は成功しています。以前の ENOSPC 失敗試行と、その中の build 警告行は保持されています。
- 3 件のソース材料の構文分類は Unknown のままです。agent の自己申告レビューはそれらを Clean に変更していません。現在のセッション使用量レシートはなく、使用量カウンターはすべて null です。ユーザーに見える効果は unknown です。
- 文書同期後、Runtime status は close 待ちに加えて `material_review_receipt_stale` と `repository_material_inspection_unavailable` を報告します。新しい材料レビューや正式検証は記録しておらず、既存レシートは以前の snapshot に結び付いています。read-only archive validator は凍結アーカイブを verified としますが、現在の status の unknown は解消されません。
- 最終 HEAD の Linux／Windows hosted CI、実際の draft PR 実行、比較可能な性能測定は未検証です。アーカイブ Contract に `resourceContext` はなく、現在の模擬 PR ルートは `ordinary_repository_route` を選択しました。そのため、この WI に束縛された PR や hosted run があるとは主張しません。branch と PR の完全一致を持つアーカイブ Contract の一般ルート回帰テストは成功しています。
- 状態は `archived` であり、closed ではありません。`close_decision_pending` と `user_visible_benefit_not_declared` が残り、close 決定は記録されていません。

## ネイティブ証拠

- Source HEAD：`e9e6526008f73459064026852d79bbaee6400c23`。
- アーカイブ Contract：`.ai/work-items/archive/WI-1111-token-outcome-ci.contract.json`（`sha256:a82110bf724bca97055a265b3792f95942f1b4c98f394dad52c100d350baa2f4`）。
- アーカイブマニフェストと凍結成果物：`.ai/work-items/archive/WI-1111-token-outcome-ci.archive.json`、`summary.json`、`outcome.json`、`task-report.json`、`task-report.md`。
- 検証レシート：`.ai/evidence/WI-1111-token-outcome-ci.verification.json`。
- 材料レビューのレシート：`.ai/evidence/material-inspection-review/WI-1111-token-outcome-ci/99092156923ce9d1bd906919ab9cf163a5acfa7effea682805c7dc152dd5f1e9.json`。

公開ソース／ネイティブ／helper 資料と draft PR には、別途範囲を限定した許可が必要です。hosted CI は正確な納品 HEAD で実行し、既存ゲートを維持する必要があります。このページは merge、release、本番有効化、共有 Runtime の置換、人手による close を許可しません。
