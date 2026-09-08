---
author: AI Cockpit maintainers
title: 性能最適化専項の総括 (2026-09)
description: 何を計測して修正したか、何を計測した上で意図的に着手しなかったか、何がまだ未計測か。
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-651-performance-initiative-synthesis
---

# 性能最適化専項の総括 (2026-09)

本文書は 2026-09 の AI Cockpit 性能最適化専項を締めくくるものである。目的は、
何を計測して変更したか、何を計測した上で*意図的に*変更しなかったか（その判断の
根拠とともに）、そして何がまだ未計測かを一箇所に記録し、将来の contributor が
同じ計測を再導出したり優先度を推測したりせず、検証済みの事実から出発できる
ようにすることである。

North Star: Calibrated Human-Agent Trust。以下の変更はいずれもガバナンス判定・
リポジトリ隔離・証拠の有効性・回復動作を保持しており、必須チェックを弱めたり、
実測前後比較のない性能上の主張をしたりしていない。

## 完了した Work Item

### WI-647 — ベンチマークの cold/warm 分類の正しさ (P0)

`tests/performance/runtime_benchmark.sh` は全サンプルをソートしてから最小値を
"cold" として選んでおり、最初のプロセス呼び出しが後続の最速呼び出しにすり替わって
いた。元の呼び出し順で先に分類するよう修正し
（`tests/performance/runtime_benchmark_stats.py`）、固定シーケンステスト
（`[120, 20, 22, 21]` → cold は必ず `120`）で固定した。既定のサンプル数下限を
下回る `p50`/`p95` の主張も抑制し、計測環境と計測前 identity probe の開示を記録し、
`regression_gate.sh` の `int` 限定の型チェック（このハーネスが生成した実測 float
の `elapsedMs` を一度も受理できていなかった）も修正した。Rust ソースは変更して
いない。`docs/work-items/WI-647-benchmark-cold-warm-grouping.md` を参照。

### WI-648 — status のボトルネック診断 (P0)

修正済みハーネスを用いて計測したところ、本リポジトリで `status` は約1.8秒
（`inspect`/`doctor`/`observe` は110ms未満）だった。一時的な、コミットしない
プロファイリングにより、`historical_finalization_inventory`
（`crates/cockpit-repository/src/lib.rs:3832`）がレガシーな
`.ai/decisions/*.finalize.json` receipt 1件ごとに
`resolve_resource_finalization_head` を呼び、この関数自体が毎回 `.ai/decisions`
ディレクトリ全体を再スキャンしていた（O(decisionsエントリ数×レガシーreceipt数)、
O(エントリ数)ではない）ことを根本原因として特定した。診断のみでコードは変更して
いない。`docs/work-items/WI-648-status-bottleneck-diagnosis.md` を参照。

### WI-649 — status の履歴スキャン修正 (P1)

WI-648 が特定したボトルネックを修正: `historical_finalization_inventory` は
`.ai/decisions` を一度だけ読み、各 work item の事前グループ化済み transition
候補を新設の `resolve_resource_finalization_head_with_candidates` に渡すように
なり、receipt ごとの再スキャンをやめた。元の `resolve_resource_finalization_head`
（`finalize`/`finalize-verify`/`finalize-recovery-plan` が使用）は変更していない。
バイト単位で `status` の JSON 出力が完全一致すること（レガシー receipt 383件、
`runtimeDigest` のみバイナリ変更により相違）と、独立した work item 間で候補が
混同されないことを証明する新規回帰テストで検証した。計測: `status` は独立した
2組の計測で cold/warm ともに約20〜23%改善。残る約1.24秒の
`historical_finalization_inventory` のコスト（元の約1.7秒から減少）は
receipt 単位の処理（`closed_finalization_projection_kind`、
`archived_contract_digest`、transitionファイルの読み取り）であり、本 Work Item
では対応していない。`docs/work-items/WI-649-status-history-scan-fix.md` を参照。

### WI-650 — 検証子プロセスのブロッキング待機化 (P1)

`execute_captured` は全ての子プロセスを `child.try_wait()` + `sleep(10ms)`
ループで待機していた。独立したマイクロベンチマーク（無関係なノイズを除くため
CLI の外側で計測）により、ほぼ瞬時に終わるコマンドで約11msの純粋な待機追加
（平均12.19ms対1.19ms）、数秒かかるコマンドでは計測可能な差がないことを確認した。
Unix のみ修正: 子プロセスを専用スレッドに move し、そのスレッドが
`child.wait()` でブロックしてチャンネル経由で報告し、呼び出し側は1回だけ
有界の `recv_timeout` を行う。Windows は文字通り変更しておらず、明示的に
未検証である（Windows 環境が利用できないため）。`ai-cockpit verify --command
true` によるエンドツーエンド計測: baseline約104〜108ms対candidate約94〜98ms。
`docs/work-items/WI-650-verification-wait-blocking.md` を参照。

## 検討したが着手しなかった候補とその根拠

### P1 — 大容量ファイルのストリーミングハッシュ／有界並列読み取り

着手せず。本リポジトリで `ai-cockpit inspect --repo` は、9614個の追跡ファイルが
あるにもかかわらず `filesRead: 2, filesHashed: 2` を報告する ── WI-395 の
過去の最適化により、これは既に request-scoped でリポジトリサイズに依存しない
実装になっており、全ツリー走査ではない。本リポジトリで最大の追跡ファイルは約
2.1MB（`tests/conformance/reference_file_inventory.json`）で、いくつかの
adopter-acceptance マニフェストファイルが約1.3〜1.6MBである。このサイズの
ファイル全体読み取りはローカルSSD/APFS上でサブミリ秒である。本リポジトリでは
ファイル数もファイルサイズも、P0/P1 で見つかったレイテンシのいずれにも寄与して
いるという実測証拠はない ── 計測された唯一のボトルネック（`status`の約1.8秒）は
ディレクトリエントリの再スキャンと receipt 単位の解決ロジックにあり、ファイル
I/O の量ではなかった。今この時点でストリーミングハッシュや有界並列読み取りを
実装すると、実測された利益なしに並行制御の複雑さ（有界なファイルハンドル、
決定論的な出力順序）が増すだけである。将来、別のリポジトリやシナリオ
（まだ構築していない P0 の「大容量ファイル修正」シナリオなど）で異なる結果を
示す計測が得られた場合にのみ再検討すべきである。

### P2 — リクエスト/検証の重複排除 (`PhysicalSingleFlightCoordinator`)

接続せず。ソースコードの直接調査
（`crates/cockpit-verification/src/lib.rs:1473`）により、
`PhysicalSingleFlightCoordinator` は自身のテストファイル
（`crates/cockpit-verification/tests/physical_execution.rs`）以外に
`cockpit-cli`、`cockpit-mcp`、`cockpit-agent`、`cockpit-core` のいずれからも
呼び出されていないことを確認した ── 実装・テストはされているが、どのコマンド
経路にも接続されていない。これを接続するには、本専項自身の受入基準に従い、
完全な実行 identity の一致、Work Item ごとの認可と証拠紐付けの検証、常に
fresh でなければならないチェックへの明示的な fail-closed 動作が必要であり、
リポジトリ隔離と認可に関する実質的なリスクを伴う、些細ではない変更となる。
本専項のいかなる計測も具体的な並行重複コストを捉えていない: 3並行の `status`
実行（読み取り専用の、無関係なコマンド）は verification 実行の重複排除機会
ではなく I/O 競合に一致する wall time を示しており、同一 Work Item/コマンドの
並行 *verification* を検証するシナリオは計測していない。この計測がない以上、
今この coordinator を接続することは実測に基づく最適化ではなく、投機的な
アーキテクチャ変更である。再検討のための具体的な前提条件は、まだ構築して
いない P0 シナリオ「複数の並行 verification リクエスト」（同一リポジトリ・
Work Item・コマンド）を、重複実行回数を記録した上で計測することである。

### P2 — `IncrementalMerkle` の信頼できるキャッシュ化

着手せず、また現時点でリスクでもない。ソースコードの直接調査
（`crates/cockpit-git/src/lib.rs:18`）により、`IncrementalMerkle` は自身の
テストファイル（`crates/cockpit-git/tests/snapshot.rs`）以外に呼び出し元が
なく、`GitRepository::snapshot()` 等いかなる本番経路にも接続されていないことを
確認した。本番でこの型がキャッシュした digest を信頼している箇所が現状存在
しないため、本専項が確認を求めたキャッシュ有効性のリスク（mtime を復元した
同じ長さの編集、ファイルの置換/削除/リネーム/種類変更、リポジトリルートの
変更、読み取り中の並行変更、watcher イベントの消失）は今日、どれも当てはまら
ない ── その出力に依存する現行の保護された判定は存在しない。将来の Work Item
がこれを接続する場合は、いかなる性能上の主張の前に、まずこれらの有効性保証を
確立する（あるいは証明できない場合は明示的に再読み込みするか unknown を返す）
必要がある。本文書は意図的にその接続を事前承認しない。

### P3 — 常駐 MCP のリポジトリ束縛キャッシュ、プロセス内 Git、PGO

着手せず。各 P3 候補は「事前の計測が必要性を証明した場合のみ」という条件付きで
ある。P0〜P2 のいかなる計測も、この3つのいずれかが対処すべきボトルネックを
指し示さなかった: 確認されたボトルネック（`historical_finalization_inventory`
のディレクトリ再スキャン）は純粋なアルゴリズム変更（WI-649）として修正され、
常駐キャッシュ、別の Git 実装、プロファイルガイド最適化のいずれも必要と
しなかった。今これらのいずれかに着手することは、標的のない最適化であり、
本専項自身の基本原則（先に計測する、倍率を仮定しない）が明確に禁じている。

## 既知のギャップ(黙って落としていない)

以下の P0 スコープ項目は本専項では完了しておらず、将来の計測優先 Work Item の
ために未着手のまま残されている:

- 8シナリオのフィクスチャ行列（小/大規模クリーンリポジトリ、単一/複数/大容量
  ファイル変更、多数の履歴 Work Item、並行 verification、常駐 MCP 反復クエリ）
  ── 「多数の履歴 Work Item」シナリオのみ、本リポジトリ自身の実履歴を通じて
  実質的に検証された。
- `status`/`doctor`/`observe`/`diagnose`/`work-item status` へのフェーズ単位の
  リソース指標（読取/ハッシュバイト数、git呼び出し数、起動プロセス数、
  キャッシュ失効理由、ピークメモリ）の露出 ── 現状 `filesRead`/`filesHashed`/
  `gitCalls` を露出するのは `inspect` のみ。
- 常駐 MCP セッションの latency 計測 ── 修正済みハーネス（WI-647）は独立した
  CLI プロセスの latency のみを明示的に計測する。
- 明示的に紐付けられた baseline/candidate の Runtime identity の差異を許容し
  つつ各側の証拠完全性は検証する開発専用比較器 ── `regression_gate.sh` は
  依然として baseline と candidate の `runtimeVersion`/`runtimeDigest` の完全
  一致を要求しており、リリース受入れには正しいが、開発時の identity 束縛が
  異なるビルド同士を比較する用途には未対応のギャップとして残っている。

これらはいずれも独立してスコープ設定可能な正当な Work Item であり、ここで
実装していないのはいずれもまだ計測されていないためであり、本専項の「修正前に
計測する」という規律に整合している。
