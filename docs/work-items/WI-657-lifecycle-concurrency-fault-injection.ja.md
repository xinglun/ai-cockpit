---
author: AI Cockpit maintainers
title: WI-657 — 並行 finish_work_item のフォールトインジェクション
description: 2スレッドによる制御されたフォールトインジェクションテストが atomic_write の実在の衝突を発見・修正し、より深い第二の問題を意図的に未修正のまま文書化する。
workItemId: WI-657-lifecycle-concurrency-fault-injection
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-657-lifecycle-concurrency-fault-injection
terminalArchive: .ai/work-items/archive/WI-657-lifecycle-concurrency-fault-injection.contract.json
terminalVerification: .ai/evidence/WI-657-lifecycle-concurrency-fault-injection.verification.json
terminalFinalization: .ai/decisions/WI-657-lifecycle-concurrency-fault-injection.finalize.json
terminalDecision: .ai/decisions/WI-657-lifecycle-concurrency-fault-injection.close.json
---

# WI-657 — 並行 finish_work_item のフォールトインジェクション

本 Work Item はアーキテクチャ最適化専項の P2-C にあたる: 既存のメカニズムを
まず調査し、欠陥の存在を先入観で決めつけることなく、制御されたフォールト
インジェクションによって並行実行下でのマルチファイル一貫性とリカバリを
検証する。

## 何を、なぜテストしたか

`finish_work_item_internal` は、P0マップ
（`docs/reference/architecture-responsibility-map-2026-09.md`）が特定した
読取+判定+永続化混在の関数の中で最大のものであり、同一 Work Item を巡って
二つの呼び出し元が競合する状況が一度も検証されていなかった。既存のテスト
群は、失敗後の逐次リトライ（`recovery_decision.rs`）や単一呼び出し元での
部分書き込みロールバックは既にカバーしているが、同一 Work Item に対して
二つのスレッドが同時に `finish_work_item` を呼ぶ状況は存在しなかった。
新設のテスト `crates/cockpit-repository/tests/lifecycle_concurrency.rs` は
まさにそれを行う: `std::sync::Barrier` で同期させた2つのスレッドが、
`checkpointed` 状態に達した同一 Work Item に対して `finish_work_item` を
同時に呼び出す。

## 発見1（修正済み）: atomic_write の一時ファイル名衝突

`atomic_write`（`crates/cockpit-repository/src/lib.rs`）は一時ファイル名を
`std::process::id()` のみから導出していた。そのため、同一プロセス内の
2スレッドが同じ書き込み先パスに書き込むと、*同一*の一時ファイルパスを
奪い合うことになる: 後から `fs::rename` を呼んだスレッドは、自分の一時
ファイルが既に相手のリネームによって消費済みであることに気づき、書き込み
先パスに対する紛らわしいファイルシステムレベルの「not found」エラーで
失敗する ── ビジネスレベルの却下ではなく。これは実在する欠陥であり、
経験的に確認済みである（修正前は繰り返し実行で再現し、修正後は再現しなく
なった）。

修正は、既存の `NEXT_ATOMIC_WRITE_ID` アトミックシーケンスカウンタと pid を
組み合わせる ── 本ファイル内で `write_cap_immutable` とパラレルスロットの
リース書き込みが既に使っているのと同じパターンであり、新規のメカニズムでは
なく、既に実証済みのものを4箇所目の呼び出しに適用しただけである。この修正
は `atomic_write` のみに限定されており、他の関数は変更していない。

## 発見2（意図的に未着手）: ロールバックが並行した成功を上書きしうる

一時ファイル名の修正後でも、負けたスレッドは（例えば完了イベントの重複
チェックなど）正当なビジネスレベルの却下で失敗しうる。その負けたスレッドの
`finish_work_item_internal` 内の手書きロールバック
（`atomic_json(&summary_path, &original_summary)` の呼び出し）は、*自分
自身の*試行前の古いスナップショットを無条件に復元する。その際、ディスク上
の現在の状態が並行して成功した相手側の呼び出しによって既に進んでいるかを
確認していない。これにより、正当な並行成功を `checkpointed` へと巻き戻し、
成功した `finish` を静かに握りつぶすことがありうる。

これは仮説上の問題ではなく実在する正しさのギャップである。適切に修正する
には、`finish`/`archive`/`close` の周りにミューテックス境界（ロック、または
ロールバック書き込み前に summary 自身のダイジェスト/状態と照合する
compare-and-swap チェック）を設ける必要があり、これは本 Work Item の範囲を
超える、より大きく高リスクな変更である。本専項自身のリスク規律
（WI-654 で検証不十分な修正を見送った前例）に従い、本 Work Item ではこれに
着手しない。文書化せず放置するのではなく、既知の未着手の限界として本書に
記録する。

## テスト設計

新設のテストは、修正が保証する内容のみを検証し、未解決の第二の問題は
検証しない:

- 2つの並行 `finish_work_item` 呼び出しのうち、少なくとも1つは成功する。
- 成功・失敗を問わず、いずれの結果もファイルシステムレースの生の痕跡
  （"No such file or directory" / "os error 2"）を含まない。
- どちらの試行の書き込みが最後にディスクに残るかに関わらず、
  `summary.json` と `outcome.json` は有効でパース可能なJSONのままである。

意図的に、Work Item が `finish_ready` で終わることは検証せず、その後
`archive_work_item` も呼んでいない ── どちらも発見2が修正されていることに
依存しており、それを検証してしまうとギャップを覆い隠すか、より大きな再設計
を待つ間テストが不安定になってしまう。

## 正しさの検証

`cargo test -p cockpit-repository --test lifecycle_concurrency` は、修正後に
6回の繰り返し実行で一貫して成功した（バリアによるフォールトインジェクション
を実行ごとに決定的にするため、シングルスレッドランナーを使用）。`cargo test
--locked --workspace`、`cargo fmt --all -- --check`、`cargo clippy --locked
--workspace --all-targets --all-features -- -D warnings` はすべて成功する。

## 対象外/フォローアップ

`finish_work_item_internal` のロールバックを並行性下で安全になるよう再設計
すること（発見2）は本 Work Item の対象外である。`archive_work_item` と
`close_work_item_with_structured_decision_internal` にも類似の手書き
ロールバックコードがあり、本 Work Item ではフォールトインジェクションを
行っていない ── 発見2を修正する試みの前に、将来の P2-C フォローアップで
同じバリアベースの手法をこれらにも拡張すべきである。実質的なミューテックス
による修正は、おそらくこの3つの関数をまとめて対処する必要があるためである。
