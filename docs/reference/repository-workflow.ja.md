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

PR merge 前に branch を削除せず、provider の自動削除で finalization を迂回しません。`close` には structured human decision、archive evidence、merge 済み PR identity、削除済み finalization receipt、fast-forward 同期済みの既定 branch、clean worktree が必要です。失敗した postcondition は可視のまま fail closed になります。

close の直後に文書投影を同期します。

```sh
python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all
```

check が stale を示した場合は、狭い範囲の documentation-promotion Work Item を作成して helper を実行し、`ready_on_base` を主張する前に再確認します。helper は reader-facing status/parity だけを更新し、Contract、evidence、archive、decision の履歴を書き換えません。

## Recovery と adoption

Recovery は append-only で identity-bound です。snapshot の変更、stale receipt、provider conflict は retry、successor、supersede decision として記録します。後続を green にするために古い evidence を編集しません。Install、upgrade、adapter setup は独立した repository Work Item とし、immutable public Release を使います。process-wide な current project を選ぶ command や、provider-global Agent/MCP 設定を変更する command はありません。

新しい Runtime が作成する successor には、正確な predecessor Work Item、Contract digest、recovery path、repository binding が必要です。これらの Contract field が存在する前に作成された historical successor については、recovery receipt 自体が predecessor と successor を bind し、successor に検証済み archive、strict verification evidence、confirmed close decision が揃っている場合にだけ狭い互換経路を許可します。新しい append-only recovery receipt には `successorBindingMode: legacy_terminal_evidence` を記録します。欠落、foreign、stale、malformed、symlink、または不完全な evidence は `recovery_decision_invalid` のまま拒否され、transition を認可しません。この互換 projection は未完了 successor を green にせず、predecessor bytes も書き換えません。

これは Rust-native な semantic workflow です。参照 source の `make` command、Python module、generated history は比較材料であり、本 repository の command や Runtime authority ではありません。
