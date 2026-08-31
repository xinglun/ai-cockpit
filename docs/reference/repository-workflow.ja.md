---
author: AI Cockpit maintainers
title: リポジトリワークフロー
description: Work Item、レビュー、アーカイブ、クリーンアップの repository-scoped 手順。
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: translation
canonical: docs/reference/repository-workflow.md
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - repository_workflow
---

# リポジトリワークフロー

[English](repository-workflow.md) · [简体中文](repository-workflow.zh-CN.md) · [日本語](repository-workflow.ja.md)

AI Cockpit は、限定された変更ごとに 1 Work Item、1 専用 branch/worktree、1 件のレビュー済み Pull Request を使います。マシン上の Runtime は共有できますが、Contract、evidence、repository identity は request-scoped です。

## 開始からレビューまで

1. remote の既定 branch の最新 commit を取得し、remote、branch、revision を Contract に記録します。
2. その revision から専用 linked worktree と branch を作成します。
3. 明示的な scope、out-of-scope、authority、acceptance、required evidence を指定して `ai-cockpit start --repo <worktree> --id <id> --intent <text> --goal <text>` を実行します。
4. preflight と checkpoint を実行します。yellow/red は人のレビュー条件であり、編集・完了の許可ではありません。
5. 宣言した scope だけを変更し、同じ `--repo` と明示的な argv で verify を記録してから `finish`、`archive` を実行します。
6. 正確な branch を push し、1 件のレビュー済み PR を作成して必要な hosted check を待ちます。local `main` への merge でレビューを代替してはいけません。

## Repository 全体の serial 境界

新しい Contract を書く前に、Runtime は全 linked worktree を確認します。別の non-detached worktree に active Contract/Summary の組がある場合、または組が壊れている場合は新しい Work Item を停止します。replacement が predecessor を暗黙に終了させることはありません。明示的な recovery/supersede decision を記録し、predecessor の bytes を保全します。

## Merge、close、cleanup

手順は次の順序です。

```text
最新 remote 既定 base → 専用 branch/worktree → 実装
→ verify/finish/archive → レビュー済み PR → merge → finalize-verify → close
→ 既定 branch 同期 → 正確な branch/worktree の削除
```

PR merge 前に branch を削除せず、provider の自動削除で finalization を迂回しません。新しい
Work Item の `close` には structured human decision、archive evidence、merge 済み PR identity、
削除済み finalization receipt、fast-forward 同期済みの既定 branch、clean worktree が必要です。
検証済みの歴史 shared-worktree または direct-merge receipt は、`historical_low` assurance、
明示的な human authority、repository に束縛された Git facts がある場合だけ狭い `retained`
例外を使えます。新しい Work Item には適用されず、歴史 evidence を昇格させません。失敗した
postcondition は可視のまま fail closed になります。

close の直後に文書投影を同期します。

```sh
python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all
```

check が stale を示した場合は、狭い範囲の documentation-promotion Work Item を作成して helper を実行し、`ready_on_base` を主張する前に再確認します。helper は reader-facing status/parity だけを更新し、Contract、evidence、archive、decision の履歴を書き換えません。

## Recovery と adoption

Recovery は append-only で identity-bound です。snapshot の変更、stale receipt、provider conflict は retry、successor、supersede decision として記録します。後続を green にするために古い evidence を編集しません。Install、upgrade、adapter setup、歴史 finalization recovery は独立した repository operation であり、必要に応じて immutable public Release を使います。`work-item finalize-recovery --repo <path> --id <id> --input <receipt.json>` は immutable な旧 finalization の唯一の互換 path です。predecessor digest、repository/Work Item/Contract base、current Runtime、actor、authority、reason、timestamp を bind しますが predecessor は編集しません。process-wide な current project を選ぶ command や、provider-global Agent/MCP 設定を変更する command はありません。

新しい Runtime が作成する successor には、正確な predecessor Work Item、Contract digest、recovery path、repository binding が必要です。これらの Contract field が存在する前に作成された historical successor については、recovery receipt 自体が predecessor と successor を bind し、successor に検証済み archive、strict verification evidence、confirmed close decision が揃っている場合にだけ狭い互換経路を許可します。新しい append-only recovery receipt には `successorBindingMode: legacy_terminal_evidence` を記録します。欠落、foreign、stale、malformed、symlink、または不完全な evidence は `recovery_decision_invalid` のまま拒否され、transition を認可しません。この互換 projection は未完了 successor を green にせず、predecessor bytes も書き換えません。

1 つの predecessor に選択済み successor lineage は 1 つだけです。有効な
`successor` receipt が存在する状態で別の Work Item を指す `successor` decision
を追加すると、Runtime は安定した境界
`recovery_decision_invalid:competing_successor` で fail closed します。既存の
lineage を継続するか、明示的に `supersede` を記録してください。複数の successor
をファイル名から人が推測する状態を残さず、predecessor の bytes を書き換えずに
recovery graph と終端 decision の監査可能性を保ちます。

archived predecessor に、対象の binding が未完了だった古い successor 試行が残って
いる場合でも、より新しく有効な `supersede` receipt がその歴史的残留を解決できます。
Runtime は、その receipt が有効で記録時刻の順序で勝つ場合に限り古い記録を historical
として扱います。malformed、foreign、改ざん、または新しいが無効な記録は引き続き
fail closed です。Contract、Summary、Outcome、Events、Evidence、recovery receipt の
bytes は書き換えません。

これは Rust-native な semantic workflow です。参照 source の `make` command、Python module、generated history は比較材料であり、本 repository の command や Runtime authority ではありません。
