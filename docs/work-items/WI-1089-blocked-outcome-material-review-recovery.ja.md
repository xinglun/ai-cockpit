---
author: AI Cockpit maintainers
title: "WI-1089 — ブロックされた Outcome の素材レビュー復旧"
description: "失敗 Outcome と機微な Contract 変更の後に限定した復旧。"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1089-blocked-outcome-material-review-recovery
status: in_progress
lastVerifiedBy: WI-1089-blocked-outcome-material-review-recovery
---

# WI-1089 — ブロックされた Outcome の素材レビュー復旧

機微な Contract 変更後も、保存された失敗 Outcome は赤のままです。新しい
preflight と本人に結び付いた `confirm_review` は現行 Contract のレビューだけを
示し、素材検査の残余リスクを受け入れません。クリーンなコミット済み Rust ソースに
許可されたレビュー可能な構文 Unknown が一つだけあり、Finding がなく、現行要求と
Contract の素材レビューポリシーが一致する場合に限り、Runtime は
`record_material_review_decision` を提示できます。素材の決定には別個の権限が
必要です。

有効な素材レシートの後、通常の前提条件を満たす新しい検証を提示できます。実際に
成功した検証レシートが記録されるまで現在の失敗 Outcome は赤のままです。その後
現在の投影が更新され、追記のみの失敗イベントは引き続き読めます。それまでは finish
と archive を許可しません。汚れたソース、利用不能
な素材投影、Finding、矛盾した証拠、古いまたは異なるレビュー入力、誤った復旧
判断は拒否します。

現行の実装範囲は repository 状態投影、repository/CLI/MCP の回帰テストと本 WI
の三言語文書です。reference-parity 登録には別の Contract 範囲判断が必要です。
