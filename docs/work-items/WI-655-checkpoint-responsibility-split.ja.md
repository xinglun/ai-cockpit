---
author: AI Cockpit maintainers
title: WI-655 — checkpoint_work_item を Observation/Governance/Persistence に分割
description: 最初に実証されたP2-Aのユースケース。同一ファイル・同一シグネチャで挙動を一切変えないリファクタリング。
workItemId: WI-655-checkpoint-responsibility-split
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-655-checkpoint-responsibility-split
terminalArchive: .ai/work-items/archive/WI-655-checkpoint-responsibility-split.contract.json
terminalVerification: .ai/evidence/WI-655-checkpoint-responsibility-split.verification.json
terminalFinalization: .ai/decisions/WI-655-checkpoint-responsibility-split.finalize.json
terminalDecision: .ai/decisions/WI-655-checkpoint-responsibility-split.close.json
---

# WI-655 — checkpoint_work_item を Observation/Governance/Persistence に分割

本 Work Item はアーキテクチャ最適化専項の P2-A にあたる。P0マップの
「`finish`/`archive`/`close` に着手する前に最小の生命周期関数から始める」という
推奨に従い、Observation/Governance/Evidence+Projection の責任抽出を初めて
完全な形で実証したユースケースである。

## なぜこの関数か、なぜ同一ファイル内か

`checkpoint_work_item`（`crates/cockpit-repository/src/lib.rs`）は、観察
（`summary.json`/`contract.json` の読み取り、Git snapshot のキャプチャ）、
ガバナンス判定（状態/重複/新鮮度/verification順序のチェック、および共有の
preflight ガバナンス判定への呼び出し）、永続化（`atomic_json` への書き込みと
`LifecycleReceipt` の構築）を、約120行の1関数内に混在させていた。
`crates/cockpit-repository/tests/` 内の17ファイルに既にテストカバレッジが
あり、実質的な回帰網となっている。

WI-654 は、隣接するガバナンスコードへの一見明白な変更が、既存のテストが
たまたまカバーしていないエッジケースで挙動を静かに変えてしまう可能性があることを
示した。そのリスクを踏まえ、本 Work Item は意図的に `checkpoint_work_item` の
本体を**同一ファイル内で、公開シグネチャ・エラーメッセージ・書き込まれる
JSONフィールド・書き込み順序を一切変えずに**、名前付きの内部ヘルパーへと
再編成する ── モジュールや crate をまたぐ抽出ではない。これは、より大きく
リスクの高い変更を検討する前に、責任境界を証明するための保守的で低リスクな
方法である。

## 分割の設計に影響した細部

元のコードは、既に読み込み済みの `summary.json` のフィールドだけを必要とする
3つの安価なチェック（重複checkpoint、生命周期状態、preflight状態の存在）が
通過するまで、Contract の読み取りや Git snapshot のキャプチャを行わない。
この順序は意図的なフェイルファストである: checkpoint が安価な理由で無効な場合、
関数はまず Git snapshot のコストを払ってからそれでも却下する、という
ことをしてはならない。さらに重要なのは、複数の問題が同時に存在する場合、
表面化すべきなのは*最初に*違反したチェックのエラーであり、並べ替えた版が
たまたま最初にヒットするエラーではない、という点である。

そのため本分割はこの正確な順序を保持している: 3つの安価なチェックは
`checkpoint_work_item` 内にインラインのまま残り、新設の `checkpoint_observe`
呼び出しより前に実行される。元のコードで既に Contract と snapshot を必要と
していたチェック（snapshot新鮮度、contract新鮮度、preflightガバナンス、
verification順序）だけが、新設の `checkpoint_governance_checks` に移動し、
観察の後に、元と同じ相対順序で呼び出される。

## 変更内容

- `CheckpointObservation`（private構造体）: `contract_path`、`contract`、
  `snapshot`、`current_snapshot_digest`、`current_contract_digest`。
- `checkpoint_observe(root, work_item_id) -> Result<CheckpointObservation,
  ObserverError>`: 以前インラインにあった contract 読み取り+snapshot
  発見+digest計算のコードそのもので、内容もエラーパスも変更していない。
- `checkpoint_governance_checks(root, summary_path, summary, preflight_state,
  observation) -> Result<(), ObserverError>`: 元のコードで観察の後に実行
  されていた4つのチェックを、同じ順序、同一のエラーメッセージで実行する。
  このヘルパーが I/O フリーであるとは主張していない:
  `require_green_or_yellow_preflight_governance` は以前と全く同様に、
  自らの観察を内部で行い続ける。
- `checkpoint_work_item` は3つの安価な事前チェックをそのまま保持し、
  その後 `checkpoint_observe`、続いて `checkpoint_governance_checks` を呼び、
  最後に変更していない永続化コード（`append_checkpoint_evidence`、summary
  フィールドの書き込み、`atomic_json`、`LifecycleReceipt` の構築）を実行する。

## 正しさの検証

`checkpoint_work_item` を呼び出す既存の17テストファイル
（`agent_risk_checkpoint.rs`、`archive_integrity.rs`、`contract_preflight.rs`、
`intelligence.rs`、`evidence_assurance.rs`、`knowledge_projection.rs`、
`knowledge_cache.rs`、`lifecycle_order.rs`、`outcome_report.rs`、
`preflight_review.rs`、`recovery_events.rs`、`recovery_revalidation.rs`、
`task_outcome_events.rs`、`recovery_decision.rs`、`status_projection.rs`、
`verification_route.rs`、`resource_finalization_transition.rs`）はすべて
変更なく成功する ── テストの追加・削除・アサーション変更は一切ない。
`cargo test --locked --workspace` は成功する（120件の test result ブロック、
失敗0件）。`cargo fmt --all -- --check` と `cargo clippy --locked --workspace
--all-targets --all-features -- -D warnings` も成功する。

## 対象外/フォローアップ

`finish_work_item`、`archive_work_item`、
`close_work_item_with_structured_decision` は同様の読取+判定+永続化混在を
持つ、より大きく高リスクな関数であり、本 Work Item では着手していない。
`docs/reference/architecture-responsibility-map-2026-09.md`（WI-652、まだ
マージされておらず本ブランチには存在しない）は、マージ後に、これを最初に
実証された P2-A のユースケースとして記録するよう更新すべきである。
