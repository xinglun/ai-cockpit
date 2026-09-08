---
author: AI Cockpit maintainers
title: WI-649 — status の O(n²) 履歴スキャンボトルネックの修正
description: WI-648 が特定した.ai/decisionsの重複再スキャンを、計測済みの前後比較とともに除去する。
workItemId: WI-649-status-history-scan-fix
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-649-status-history-scan-fix
terminalArchive: .ai/work-items/archive/WI-649-status-history-scan-fix.contract.json
terminalVerification: .ai/evidence/WI-649-status-history-scan-fix.verification.json
terminalFinalization: .ai/decisions/WI-649-status-history-scan-fix.finalize.json
terminalDecision: .ai/decisions/WI-649-status-history-scan-fix.close.json
---

# WI-649 — status の O(n²) 履歴スキャンボトルネックの修正

本 Work Item は WI-648 が根本原因を特定したボトルネックに対する P1 の修正である。
`status` が `inspect`/`doctor`/`observe` より20〜40倍遅かったのは、
`historical_finalization_inventory` が `resolve_resource_finalization_head` を通じて
レガシー receipt 1件ごとに `.ai/decisions` ディレクトリ全体を再スキャンしていたためである。

## 変更内容

`crates/cockpit-repository/src/lib.rs`:

- `historical_finalization_inventory` は `.ai/decisions` を一度だけ読み、その一覧を
  メモリ上に保持し、そこから `work_item_id -> transition候補` のインデックスを構築する
  （各ファイル名の最初の `.finalize.` 境界でグループ化する。work item ID は `.` を
  含めないため、これは曖昧さのない厳密な分割である）。
- `resolve_resource_finalization_head` を、薄いラッパー（シグネチャ・挙動・ディレクトリ
  スキャンは変更なし）と、新設の `resolve_resource_finalization_head_with_candidates`
  に分割した。後者は同じチェーン解決ロジックを行うが、自らディレクトリをスキャンする
  代わりに呼び出し元が渡す候補リストを消費する。
- `historical_finalization_inventory` は、既にグループ化済みの候補を使って
  `_with_candidates` 版を呼ぶように変更した（`resolve_resource_finalization_head` を
  呼ぶと再スキャンが発生するため）。他の3つの呼び出し元（`finalize`、
  `finalize-verify`、`finalize-recovery-plan`）は変更前のラッパーを呼び続けており、
  本 Work Item はそれらの挙動・コスト・シグネチャに一切手を加えていない。

出力フィールド、スキーマ、ガバナンス判定への変更は一切ない。これは1回のディレクトリ
一覧取得を使い回すだけの、純粋な内部リファクタリングである。

## 正しさの検証

- `cargo test --locked --workspace` は変更なく成功する（121件のtest resultブロック、
  失敗0件）。
- 新規テスト `crates/cockpit-repository/tests/historical_finalization_scan.rs` は、
  1つのリポジトリ内に独立した2つのレガシー Work Item を構築する ── ALPHA は
  2ステップの transition チェーン（sequence 2）、BETA はチェーンなし（sequence 0）
  ── そして `status_with_runtime` の `historicalFinalization` エントリが、
  *変更していない* `resolve_resource_finalization_head`（`verify_resource_finalization`
  経由）が独立に計算した正解と一致することを両方について検証し、新しいインデックスが
  Work Item 間で候補を混同しないことを証明する。
- バイト単位の直接比較: 本リポジトリ（アーカイブ済み Work Item 580件以上、レガシー
  `.finalize.json` receipt 383件）に対して `ai-cockpit status --repo` を実行し、
  インストール済み v0.2.87 バイナリの JSON 出力と本 Work Item のリリースビルドの
  出力を比較したところ、`runtimeDigest`（バイナリのバイト列が変わったため必然的に
  変化する）を除く全フィールドが完全に一致した。383件の `historicalFinalization`
  エントリすべてについて、`state`、`sequence`、`predecessorDigest`、
  `historicalKind`、`safeActions` が一致している。

## 計測された性能(参考情報)

2026-09-08、macOS arm64（WI-648 の診断と同一リポジトリ）にて、WI-647 で修正済みの
cold/warm 分類ハーネス（`tests/performance/runtime_benchmark.sh`。本リポジトリに対して
外部から実行しており、本 Work Item 自体はこのスクリプトを変更していない）を用い、
12イテレーション、baseline 対 本 Work Item のリリースビルドを、独立した2組の計測で
比較した:

| コマンド | 計測1 | 計測2 |
|---|---|---|
| status.cold | -20.3% | -22.9% |
| status.warm | -21.2% | -22.0% |

`status` は一貫して約20〜23%改善している（約1.86〜1.96秒 → 約1.48〜1.51秒）。
より詳細なステップ別プロファイル（WI-648と同じ、一時的でコミットしない計測手法）では、
`historical_finalization_inventory` 自体が約1.7秒（WI-648のbaseline）から
約1.24〜1.37秒に減少している ── 除去した重複ディレクトリ一覧取得がこの差分に
相当する。残る約1.24秒は receipt 単位の処理（`closed_finalization_projection_kind`、
`archived_contract_digest`、transition ファイル自体の読み取り）に費やされており、
本 Work Item はこれを変更していない。さらに最適化するには、将来の Work Item で
別途診断する必要がある。

`inspect`/`doctor`/`observe` の差分は計測を繰り返すたびに符号が反転するほど
ノイズが大きかった（例えば `doctor.warm` はある計測では+9.2%、*同一の* 未変更
バイナリを異なるファイルシステム上の場所で自分自身と比較した2回の計測でも
`inspect.cold` が+0.9%/-14.1%とばらついた）── これらのコマンドは
`historical_finalization_inventory` も `resolve_resource_finalization_head` も
一切呼び出さない（`crates/cockpit-cli/src/main.rs:686` の通り、呼ぶのは
`status_with_runtime` だけである）ため、本変更がこれらに影響を与えるコードパスは
存在せず、これらの絶対値（120ms未満）の小ささには修正済みハーネス（WI-647）自身の
`p95Unreliable`/`p50Unreliable` 信頼性ガードが該当する。これらはローカルな
プロセス latency の観測であり、provider や enterprise の保証ではない。

## 対象外

`historical_finalization_inventory` の結果を複数回の `status` 呼び出しをまたいで
キャッシュすること（WI-648の第2候補）、および残る約1.24秒の receipt 単位コストの
診断は、将来の Work Item に委ねる。
