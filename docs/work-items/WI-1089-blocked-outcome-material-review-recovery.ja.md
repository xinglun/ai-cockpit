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

保持された canonical レシートは Missing ではありません。限定した nofollow 読み取りの
同じバイト列に対し、型付き計画、実行、保持ポリシーの結合を検証します。前身 Contract
は検証済みのネイティブ変更履歴からだけ取得します。履歴の snapshot と producer は
現行ソースと実行権限から分離します。不完全または未対応の履歴は限定診断とともに
拒否し、一般的な Stale を許可理由にしません。

同等フィクスチャは実際の旧四項成功レシートと blocked Outcome を保持します。受入には
現行の厳密な素材判断、plan-only のゼロプロセス、一 worker による四項実行とゼロ再利用、
新しい green preflight、その後の実際のフィクスチャ finish/archive が必要です。
別のテストは同じ結合での正当な再利用を保持し、破損・不完全履歴を拒否します。
実際のチェック失敗には厳密なネイティブ retry が必要です。これは Sentinel の実受入でも
実 Work Item の finish/archive 許可でもありません。

変更後の範囲は repository の変更履歴・証拠検証・状態投影の三実装、repository/CLI
テスト、六つの三言語 WI/parity 文書です。以前の六ノードと統合ソースの結果は履歴として
保持します。新候補には独自の完全な検証と独立した厳密な素材判断が必要で、以前の限定的
リスク同意を新 Rust バイトへ流用しません。PR1048 は Draft、WI1090 は停止のままです。
