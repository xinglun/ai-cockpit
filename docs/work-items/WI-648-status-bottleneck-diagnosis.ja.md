---
author: AI Cockpit maintainers
title: WI-648 — status コマンドのボトルネック診断
description: 最適化に着手する前に、status コマンドの支配的なコストの根本原因を特定する。
workItemId: WI-648-status-bottleneck-diagnosis
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-648-status-bottleneck-diagnosis
terminalArchive: .ai/work-items/archive/WI-648-status-bottleneck-diagnosis.contract.json
terminalVerification: .ai/evidence/WI-648-status-bottleneck-diagnosis.verification.json
terminalFinalization: .ai/decisions/WI-648-status-bottleneck-diagnosis.finalize.json
terminalDecision: .ai/decisions/WI-648-status-bottleneck-diagnosis.close.json
---

# WI-648 — status コマンドのボトルネック診断

本 Work Item は AI Cockpit 性能最適化専項の P0 における計測・診断ステップである。
本番コードやガバナンス動作は一切変更しない。`ai-cockpit status` が
`inspect`/`doctor`/`observe` に比べて著しく遅い理由を、再現可能な証拠とともに
根本原因まで特定し、次の最適化 Work Item に「推測」ではなく「検証済みの標的」を渡す。

## 出発点となる計測結果

WI-647 で修正済みのベンチマークハーネスを用いて本リポジトリを計測した結果
（macOS arm64、インストール済み v0.2.87 バイナリ、10 イテレーション）:

| コマンド | cold (ms) | warm p50 (ms) |
|---|---|---|
| inspect | 58.3 | 58.3 |
| status | 1788.5 | 1848.4 |
| doctor | 48.0 | 40.7 |
| observe | 109.4 | 105.9 |

`status` は cold・warm いずれも他コマンドの約20〜40倍遅い。
`inspect`/`doctor`/`observe` にはこのコストが現れないため、原因は全コマンド共通の
処理（プロセス起動、identity解決、Git snapshot取得）ではなく、`status` 固有の
コードパスにあると判断できる。

## 手法

`crates/cockpit-cli/src/main.rs:686` を見ると、`status` コマンドだけが
`cockpit_repository::status_with_runtime(&repo, Some(&runtime_context))` を
呼び出しており、他の計測対象コマンドは別の経路を通る。`status_with_runtime` は
`repository_readiness_from_snapshot_with_runtime`
(`crates/cockpit-repository/src/lib.rs:3452`) を呼び、5つのステップを順に実行する。
各ステップのコストを特定するため、一時的なローカルパッチ（コミットはしない）で
各ステップを `std::time::Instant`/`eprintln!` でラップした。例:

```rust
let __t4 = std::time::Instant::now();
let historical_finalization = historical_finalization_inventory(root, runtime)?;
eprintln!("__PROFILE__ historical_finalization_inventory {:?} count={}", __t4.elapsed(), historical_finalization.len());
```

`cargo build --release -p cockpit-cli` でビルドし、本リポジトリおよび新規に
`attach` したスクラッチフィクスチャに対して直接実行した後、コミット前に
`git checkout -- crates/cockpit-repository/src/lib.rs` で元に戻した。上記の行番号から
誰でも再現できる。

## 発見: O(n) の履歴増加ではなく O(n²) のディレクトリ再スキャン

本リポジトリに対するステップ別の計測結果（初回呼び出しで OS キャッシュが
温まった後の warm 値。`.ai/decisions` には 1573 エントリ、うち `*.finalize.json`
が 383、`.ai/work-items/archive` には 580 以上のアーカイブ済み Work Item が存在）:

| ステップ | 所要時間 | 備考 |
|---|---|---|
| `discover_default_base` | 約 9 ms | Git 呼び出し1回 |
| `non_governance_changed_paths` | 1 µs 未満 | 既存 snapshot 上のインメモリ処理 |
| `unclosed_archived_work_items_with_id` | 約 90 ms | `.ai/work-items/archive` を1回スキャン、各項目は O(1) のファイル参照 |
| `classify_historical_debt` | 約 0 µs | 本リポジトリでは unclosed 項目が 0 件だった |
| `historical_finalization_inventory` | **約 1.7 秒** | 支配的なコスト |
| `count_suffix` + `orphaned_active_artifact_names` | 10 µs 未満 | `active/` ディレクトリは小さい |

新規に `attach` した空リポジトリ（`.ai/decisions` が 0 エントリ）に対する
同じ2ステップの合計は約 0.05〜0.2 ms であり、このコストは全リポジトリ共通ではなく
履歴が蓄積したリポジトリにのみ発生する。

`historical_finalization_inventory`（`crates/cockpit-repository/src/lib.rs:3832`）は
`.ai/decisions` 内の全ての `*.finalize.json` を走査する。記録された
`runtimeVersion`/`runtimeDigest` が現在実行中の Runtime と一致しない項目
（3923行目）については — これは過去バージョンの Runtime が書いた receipt では
実質すべて該当し、実履歴を持つリポジトリでは特殊ケースではなく通常ケースである —
`resolve_resource_finalization_head`（`crates/cockpit-repository/src/lib.rs:11634`）を
呼び出す。この関数自体が `fs::read_dir(root.join(".ai/decisions"))`（11654行目）を
実行し、その `work_item_id` に属する transition ファイルを絞り込む。この内側の
スキャンが外側ループの反復ごとに同じ `.ai/decisions` のディレクトリ一覧を
ゼロから読み直すため、総コストは **O(エントリ数)** ではなく
**O(decisionsディレクトリのエントリ数 × 該当するレガシー receipt 数)** となる。
本リポジトリでは 1573 エントリ × 383 件のレガシー receipt で、約 60万回の
ディレクトリエントリ照合に加え、`{work_item_id}.finalize.` プレフィックスに
一致するファイル名（本リポジトリには 156 件の transition ファイルが存在）ごとに
`read_resource_finalization_transition` + `serde_json::to_value` + `digest_json`
のコストが乗る。実データの `.finalize.json` を100件だけコピーし、対応する
`.ai/work-items/archive/*` を伴わずに部分再現したところ約 20〜166 ms しか
かからず、コストの発現には完全なアーカイブ文脈が必要であり、ファイル数だけでは
説明できないことを確認した — これにより「ファイル数のみが原因」という説を排除し、
上記の解決チェーンに原因を絞り込めた。

`unclosed_archived_work_items_with_id`（`crates/cockpit-repository/src/lib.rs:3660`）に
はこのパターンは見られない。その項目ごとのチェック
（`close_decision_is_valid_for_status`、14781行目）は既知のファイル名への直接的な
O(1) パス参照であり、ディレクトリスキャンではない。これが約90msで済む理由である
（ディレクトリ一覧取得1回 + 580件以上のO(1)参照）。

## 影響

このおよそ1.7秒の計算全体は無条件かつキャッシュされておらず、`status` が
呼ばれるたびに毎回フルに再実行される。しかし現在実行中の Runtime と
`runtimeVersion`/`runtimeDigest` が異なる receipt に関する結果は完全に
決定論的であり、リポジトリの状態が変わらなければ、繰り返しの `status` 呼び出しは
毎回同一の答えを再計算しているだけである。このコストは `.ai/decisions` の
エントリ数に対して二次関数的であるため、リポジトリの履歴 Work Item 数が
増えるよりも速いペースで悪化し続ける。これは本リポジトリに限らず、実開発履歴を
持つあらゆる adopter リポジトリに構造的に当てはまる。

## 次の（P1）Work Item への提案

WI-402/WI-647 の規律に従い、それぞれ独立に計測すべき2つの狭い範囲の
意味論を変えない候補（成果を積み重ねて曖昧に報告しない）:

1. `historical_finalization_inventory` の呼び出しごとに `.ai/decisions` を
   1回だけ読み、`work_item_id` プレフィックスでメモリ上にグループ化したインデックスを
   構築し、それを `resolve_resource_finalization_head` に渡して、外側ループの
   項目ごとにディレクトリを再スキャンしないようにする。ある `work_item_id` に対する
   候補集合はどちらの方法でも同一になるため、出力値を一切変えずに O(n²) の項を
   除去できる。
2. (1) を計測した後にのみ検討: 現在の Runtime identity と receipt の
   content digest が前回の `status` 呼び出し（同一リポジトリ内）から変わっていない
   場合に、解決済みの `HistoricalFinalizationInventoryItem` をキャッシュすることを
   検討する。WI-402 が verification reuse のために確立した fail-closed/identity
   ルールに従うこと。

本 Work Item はどちらの候補も実装しない。次の P1 Work Item に、推測ではなく
行番号付きで検証済みの標的を引き渡すのみである。

## 検証

Rust ソース・テスト・ガバナンスファイルは一切変更していない。`cargo fmt`、
`cargo clippy`、`cargo test --workspace` はすでに成功しているベースラインから
変化なし。上記の計測手法は、引用した行番号とコマンドから、レビュアーが誰でも
再現できる。
