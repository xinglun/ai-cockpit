---
author: AI Cockpit maintainers
title: WI-647 — ベンチマークの cold/warm 分類の正しさ
description: ランタイム最適化に着手する前に、開発用性能計測ハーネスの cold/warm サンプル誤分類を修正する。
workItemId: WI-647-benchmark-cold-warm-grouping
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-647-benchmark-cold-warm-grouping
terminalArchive: .ai/work-items/archive/WI-647-benchmark-cold-warm-grouping.contract.json
terminalVerification: .ai/evidence/WI-647-benchmark-cold-warm-grouping.verification.json
terminalFinalization: .ai/decisions/WI-647-benchmark-cold-warm-grouping.finalize.json
terminalDecision: .ai/decisions/WI-647-benchmark-cold-warm-grouping.close.json
---

# WI-647 — ベンチマークの cold/warm 分類の正しさ

本 Work Item は AI Cockpit 性能最適化専項の P0 にあたる。ランタイム最適化に着手する前に、
開発用性能計測ハーネス (`tests/performance/runtime_benchmark.sh`) を信頼できるものにする。
変更するのは計測ツールのみであり、ガバナンス判定・証拠の意味論・必須 verification グラフは変更しない。

## 発見した欠陥

`runtime_benchmark.sh`（commit `acb3c386`、WI-402 で導入）は収集した全サンプルをソートし、
最小値を "cold" として報告していた:

```python
values.sort()
warm = values[1:]
...
{"name": f"{name}.cold", "elapsedMs": round(values[0], 3), "iterations": 1}
```

ソートしてから分類すると、最初のプロセス呼び出しが、後続のどれか最速の呼び出しに
すり替わってしまう。固定シーケンス `[120, 20, 22, 21]`（最初が遅く、以降が速い）では、
旧コードは `20` を cold として報告し、`[21, 22, 120]` を warm として報告していた。
本来の最初の呼び出し (`120`) は cold ではなく warm 集団に紛れ込んでいた。WI-402 自身の
報告書は本スクリプトの出力をそのまま信頼しており、ソート順の妥当性を検証していなかった。

## 修正内容

- `tests/performance/runtime_benchmark_stats.py`（新規）: 純粋関数 `summarize(name,
  raw_ms, prior_probe_calls)`。`raw_ms[0]` を常に cold として報告し、`raw_ms[1:]` を
  元の呼び出し順のまま warm サンプル (`rawMs`) として保持する。percentile の計算には
  別途ソートしたコピーのみを使う。
- `tests/performance/runtime_benchmark_stats_test.py`（新規）: 固定シーケンス
  `[120, 20, 22, 21]`（cold は必ず `120`）と、逆方向の外れ値を持つシーケンス、
  および下記の信頼性フロアを検証する。
- `p50Ms`/`p95Ms` は、既定の信頼性フロア（`MIN_SAMPLES_FOR_P50=5`、
  `MIN_SAMPLES_FOR_P95=20`）を下回るサンプル数では、明示的な `insufficient_samples`
  理由付きで抑制され、少なすぎるサンプルから算出した percentile を報告しない。
  報告される `elapsedMs` は、percentile が信頼できない場合でも budget gate が
  fail-closed になるよう、観測された最悪値にフォールバックする。
- 各計測で `environment` ブロック（ハードウェア、OS、ファイルシステム、リポジトリの
  head/branch/dirty 状態、追跡ファイル数）と `preMeasurementProcessInvocations` の
  一覧を記録するようにした。`--version`/`inspect`/`status` の identity probe は
  計測開始前に既に実行されているため、`.cold` サンプルは最初に *計測された* 呼び出しで
  あって真の OS cold cache 呼び出しではない。ハーネスはそのことを明示するようになった。
- `measurementModel` に、本ハーネスが独立 CLI プロセスの latency のみを測定し、
  常駐 MCP セッションの latency は測定しないことを記録する。
- `tests/performance/regression_gate.sh` は `elapsedMs`/`maxElapsedMs` に Python の
  `int` を要求していた。実際の `runtime_benchmark.sh` の出力は常に丸められた float
  であるため、あらゆる実測値がどの budget file に対しても `sample_malformed` で
  失敗していた ― この gate は実測 evidence を一度も受理できていなかった。
  `int` または `float`（`bool` を除く）を受け付けるよう修正した。`iterations` は
  引き続き厳密な `int` のままとする。

## 対象外（後続の P0/P1 Work Item へ委譲）

- 8 シナリオのフィクスチャ行列（小/大規模クリーンリポジトリ、単一/複数/大容量ファイル
  変更、多数の履歴 Work Item、並行 verification、常駐 MCP 反復クエリ）。
- フェーズ単位の内訳（git クエリ／ファイル読取+ハッシュ／証拠検証／スケジューリング+
  子プロセス実行／outcome 投影）とリソース指標（読取/ハッシュバイト数、git 呼び出し数、
  起動プロセス数、キャッシュ失効理由、ピークメモリ）を `status`/`doctor`/`observe` にも
  露出する対応 ― 現状 `filesRead`/`filesHashed`/`gitCalls` を報告するのは `inspect` のみで、
  プロセス数やピークメモリを報告するコマンドは存在しない。
- 明示的に紐付けられた baseline/candidate の Runtime identity の差異を許容しつつ、
  各側の証拠完全性は検証する開発専用比較器。
- 常駐 MCP セッションの latency 計測。
- P1〜P3 のランタイム最適化全般（request-scoped 重複排除、`IncrementalMerkle`、
  `PhysicalSingleFlightCoordinator`、ポーリング待機の変更、常駐キャッシュ、PGO）。

## 検証

`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets
--all-features -- -D warnings`、`cargo test --locked --workspace` は変更なく成功する
（Rust ソースには一切手を入れていない）。`python3
tests/performance/runtime_benchmark_stats_test.py` と `bash
tests/performance/regression_gate_test.sh` も成功する。インストール済み v0.2.87
バイナリに対して本リポジトリで `runtime_benchmark.sh` を実行し、その JSON 出力を
実際の（float の）budget file とともに `regression_gate.sh` に通し、合格ケースと
budget 超過ケースの両方で、gate が実測 evidence を受理しつつ本物の regression には
fail-closed のままであることを確認した。

### ローカル計測（参考情報）

2026-09-07、macOS arm64（Darwin 25.6.0、arm64、論理 CPU 10 個）、本リポジトリ
（アーカイブ済み Work Item 580 件、追跡ファイル 9614 件、HEAD `1623ee5a` はクリーン）に対して、
6 回のイテレーションで次を記録した（括弧内は旧スクリプトの誤った分類）:
`inspect.cold` 98.951 ms、`status.cold` 1887.792 ms、`doctor.cold` 50.172 ms、
`observe.cold` 119.701 ms。`status` は `inspect`/`doctor`/`observe` と比べて明らかに高コストであり、
これは次の P0 ボトルネック順位付け Work Item のための生データであって、本 Work Item が
最適化したという主張ではない。これらはローカルなプロセス latency の観測結果であり、
provider や enterprise の保証ではない。
