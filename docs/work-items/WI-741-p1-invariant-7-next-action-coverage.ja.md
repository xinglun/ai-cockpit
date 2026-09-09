---
author: AI Cockpit maintainers
title: "WI-741 — P1 不変量7(次アクションの正しさ)カバレッジ"
description: "協作場景行列の実観測場景の一部に結びついた境界の明確なテストで、docs/reference/collaboration-invariant-coverage.md が示す不変量7の欠落を解消する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-741-p1-invariant-7-next-action-coverage
status: in_progress
authority: authorized
lastVerifiedBy: WI-741-p1-invariant-7-next-action-coverage
---

[English](WI-741-p1-invariant-7-next-action-coverage.md) · [简体中文](WI-741-p1-invariant-7-next-action-coverage.zh-CN.md)

# WI-741 — P1 不変量7(次アクションの正しさ)カバレッジ

## 意図

`docs/reference/collaboration-invariant-coverage.md` が示す唯一残る欠落を
解消する。不変量7「表示される次のステップが現在の Runtime 状態/ポリシー
と一致すること」については、これまでレンダリングされた復旧/次アクショ
ンのテキストを独立に記録された期待値と突き合わせて検証するテストが存在
しなかった。あらゆる Runtime 状態に対する汎用的な判定器を構築するのは現
実的ではないため、本 Work Item は同文書が既に推奨していた境界の明確な手
法を採る:`docs/reference/collaboration-scenario-matrix.json` のうち
`sourceType: "observed"` の一部の場景(SCN-001、SCN-002、SCN-016)につい
て、その正確な次アクションのテキストを各場景に記録された
`expected.keyMessage` と突き合わせて検証する。リポジトリ所有者からの明示
的な委任に基づき、AI Cockpit 協作言語専項を継続する。

## 境界

これはテストのみの Work Item である。新規に追加するファイルは
`crates/cockpit-repository/tests/scenario_matrix_next_action.rs` の1件の
みである。いかなる本番ソースコードも、既存のテストファイルも、協作不変
量カバレッジ文書や場景行列そのものも変更しない(不変量7の行を本カバレ
ッジ追加を反映するよう更新することは明示的かつ独立した後続作業であり、
本納品の対象外である。したがって本 Work Item は前身 Work Item の結論を
黙って書き換えるものと解釈されてはならない)。

## 受け入れとライフサイクル

- `crates/cockpit-repository/tests/scenario_matrix_next_action.rs` は、
  `cockpit_repository` ライブラリ関数の直接呼び出しによって生成される正
  確な(SCN-001、SCN-016)または部分一致(SCN-002)の次アクションのテキ
  ストが、`docs/reference/collaboration-scenario-matrix.json` の各場景に
  記録された `expected.keyMessage` と一致することを検証する。
- 同ファイル内の4件目の防護テストは、SCN-001、SCN-002、SCN-016 が引き続
  き `sourceType: "observed"` として宣言され、不変量7を引き続きカバーし
  ていることを確認し、本テストが守るべき文書から静かに乖離しないように
  する。
- `cargo test -p cockpit-repository --test scenario_matrix_next_action`、
  `cargo fmt --check`、`cargo clippy --tests -- -D warnings` がすべて合格
  する。
- `start → preflight → checkpoint → verify → finish → archive → close` が統治
  された経路であり、`user_visible_benefit_not_declared` は明示されたまま
  である。

## 証拠

- archive: `.ai/work-items/archive/WI-741-p1-invariant-7-next-action-coverage.contract.json`
- verification: `.ai/evidence/WI-741-p1-invariant-7-next-action-coverage.verification.json`
- finalization: `.ai/decisions/WI-741-p1-invariant-7-next-action-coverage.finalize.json`(合併後に生成予定)
- close: `.ai/decisions/WI-741-p1-invariant-7-next-action-coverage.close.json`(合併後に生成予定)
