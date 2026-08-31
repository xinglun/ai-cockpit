---
author: AI Cockpit maintainers
title: "Task Outcome レポート"
description: "Work Item が実施・発見・停止し、人間の確認に残した内容を evidence-bound に示すレポート。"
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-457
capabilityClaims:
  - task_outcome_report
---

# Task Outcome レポート

Rust Runtime は安定した machine object として `OutcomeV2` を保持し、新しく
生成する Outcome に strict な `taskOutcomeReport` projection を追加します。
これは additive であり、projection を持たない過去の OutcomeV2 bytes は読み取り
可能なまま書き換えません。

レポートには、結果概要、タスク概要、変更、発見、リスク、警告、制限、介入、
強制停止、解決、再発防止、回避した影響、残存リスク、人間の判断、実装方法、
証拠の各 section が明示されます。空の section は `None` であり、チェック済みや
効果ありを意味しません。

空でない claim には repository-local `evidenceRefs` が必要で、根拠が推論なら
`inference` を明示します。Contract の intent、scope、受入れ基準、authority は
人間が書く source text です。Runtime は利用者の効果、merge、release、provider
承認、enterprise assurance、security の事実を推測しません。

## Lifecycle artifact

`finish` 後、active Work Item には `<id>.outcome.json` と append-only の
`<id>.events.jsonl` が残ります。イベントは生成された完了、finding、risk、warning、stop、resolution、
recurrence prevention、evidence-backed な check-pass-after-fix を記録します。`archive` は event stream を byte-for-byte で移動し、archive
manifest に digest を束縛します。`close` は検証済み report を `finalReport` と
`finalReportDigest` として repository-bound close receipt に保存します。

失敗または中断したライフサイクル試行では、`<id>.outcome.finish-blocked.json` や
`<id>.events.finish-recovery.jsonl` のような Runtime 管理の投影が残ることがあります。
これらは監査履歴であり active Contract ではありません。そのため `status` は
`activeArtifacts` と `orphanedActiveArtifacts` として別に報告し、
`activeWorkItems` は Contract に基づく数え方を維持します。通常の `archive` は
認識済みの変種を正規ファイルとともに移動し、各ダイジェストを
`historicalArtifacts` に記録します。古い Runtime によって既にアーカイブされた
Work Item には、
`ai-cockpit work-item reconcile-artifacts --repo <repository> --id <id>` を使用します。
このコマンドは既存 archive manifest の検証と ID の一致を必須とし、通常ファイルだけを
移動して追記型の reconciliation receipt を作成します。履歴バイトを削除・書き換えません。

`archive` と `close` は別の境界です。アーカイブ済み Work Item は明示的な人間の判断で
初めて閉じられます。孤立した投影は Work Item が active であることを意味しませんが、
アーカイブまたは reconciliation が完了するまでリポジトリの ready 判定を阻害します。

Malformed JSON、未知フィールド、foreign identity、安全でない evidence path、
secret らしい内容、重複 ID、未出現イベントへの参照は fail closed です。修正は
新しい event として記録し、過去の行を暗黙に書き換えません。Finding/risk event は deterministic な
`findingFingerprint` を持ち、重複は明示的な correction/supersession binding がある場合だけ許可されます。
Rust の projection は reference event policy の semantic parity であり source JSON の byte-level compatibility
ではなく、reference Python generator を取り込みません。

## 人間向け handoff

CLI `ai-cockpit work-item outcome --repo <repository> --id <id>` と MCP の
`work_item_outcome` は同じ validated report と renderer を使います。status marker、
完了、問題、停止、解決、リスク、不明点、判断、検証、影響、次の action を表示します。
Runtime の生成ラベルは localize しますが、Contract source text は原文を保持します。

この report は merge、publication、provider 承認、組織上の判断を許可しません。
paused/blocked/stale/cancelled/rollback を event から再構成する完全な recovery は
別 capability です。

[Human Benefit Report](human-benefit-report.ja.md) | [Outcome reference](../reference/outcome-report.ja.md) |
[Features](README.ja.md) | [English](task-outcome-report.md) | [中文](task-outcome-report.zh-CN.md)
