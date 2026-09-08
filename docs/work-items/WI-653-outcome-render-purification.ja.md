---
author: AI Cockpit maintainers
title: WI-653 — Outcome 描画層の純化
description: render_human_outcome がリポジトリの事実を取得・検証しないようにし、新設のユースケースが一度だけ組み立てる。
workItemId: WI-653-outcome-render-purification
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-653-outcome-render-purification
terminalArchive: .ai/work-items/archive/WI-653-outcome-render-purification.contract.json
terminalVerification: .ai/evidence/WI-653-outcome-render-purification.verification.json
terminalFinalization: .ai/decisions/WI-653-outcome-render-purification.finalize.json
terminalDecision: .ai/decisions/WI-653-outcome-render-purification.close.json
---

# WI-653 — Outcome 描画層の純化

本 Work Item は AI Cockpit アーキテクチャ最適化専項の P1-A にあたる。
`crates/cockpit-repository/src/outcome_render.rs` を、既に観察・検証済みの
事実だけを扱う純粋な表示層にする。

## 発見した問題

`render_human_outcome(root: &Path, outcome: &OutcomeV2, language: &str)` は
リポジトリのルートを受け取り、描画処理の内部で自らリポジトリ I/O とガバナンス
検証を行っていた:

- アーカイブ済み Contract ファイルの存在確認と、`close_decision_is_valid_for_status`
  （`.ai/decisions/{id}.close.json` を読むガバナンスチェック）の呼び出しにより
  `archived_unclosed` を導出していた。
- `load_human_decision(root, work_item_id)` を呼び、再度
  `.ai/decisions/{id}.close.json` を読み込み、repository-id の紐付け、
  レコードの状態、決定の確定、構造化決定の全フィールドを検証していた ──
  完全なガバナンス検証ロジックがテキスト整形関数の内部に埋め込まれていた。

既存の4つの呼び出し元（`crates/cockpit-cli/src/main.rs` の
`WorkItemCommand::Outcome`、`print_lifecycle_result`、
`emit_blocked_lifecycle_handoff`；`crates/cockpit-mcp/src/lib.rs` の
`work_item_outcome`）はすべて `render_human_outcome` を呼ぶ直前に既に
`outcome_v2_with_runtime` を呼んでおり、この重複読み取りは特定の1呼び出し元
だけの偶発的な問題ではなく、全呼び出し元に共通する構造的な問題だった。

## 変更内容

- 新設の `OutcomeRenderInput { outcome: OutcomeV2, human_decision:
  HumanDecisionProjection, archived_unclosed: bool }` と、2つの組み立て関数
  `outcome_render_input`/`outcome_render_input_with_runtime` を追加。これらは
  `outcome_v2`/`outcome_v2_with_runtime` と、既存の
  `close_decision_is_valid_for_status`/`load_human_decision` のロジックを
  それぞれ一度だけ呼び出す。
- `render_human_outcome` は `&OutcomeRenderInput` と言語コードのみを受け取る
  ようになった ── 関数本体のどこにも `root: &Path`、ファイルシステムアクセス、
  ガバナンス検証呼び出しはない。
- `HumanDecisionProjection`（以前は描画モジュールローカルの private enum）は、
  公開の `OutcomeRenderInput` のフィールドとなったため `pub` にした。
- 4つの呼び出し元すべてを `outcome_render_input(_with_runtime)` の後に
  `render_human_outcome(&input, language)` を呼ぶ流れに移行。`--json`/機械
  可読JSON分岐は `input.outcome`（従来と同一の `OutcomeV2` 値）を使うため
  JSON 出力に影響はない。
- `cockpit-repository` と `cockpit-mcp` にまたがる、旧来の
  「root + OutcomeV2」シグネチャを呼んでいた既存テスト6件を新しい
  組み立て→描画フローに更新し、結果テキストと `OutcomeV2` フィールドへの
  既存アサーションはそのまま維持した。

文言・ローカライズ文字列・JSONスキーマ・ガバナンス判定ロジックはいずれも
変更していない。

## 正しさの検証

- 5つの新規ユニットテスト（`crates/cockpit-repository/src/outcome_render.rs`
  の `#[cfg(test)] mod render_tests`）は `OutcomeRenderInput` を完全に
  メモリ上で構築する ── 一時ディレクトリもファイルシステムも使わない ──
  人工決定の欠落、有効な人工決定、無効/不正な人工決定、archived-but-unclosed、
  historical/superseded の各ケースを網羅する。これにより、描画関数のテストに
  リポジトリアクセスが不要であることを直接示している。
- `cockpit-repository` の `recovery_decision.rs`、`archive_integrity.rs`、
  `evidence_assurance.rs`、`status_projection.rs`、`recovery_events.rs`、
  および `cockpit-mcp` の `rpc.rs` にある既存の統合テスト6件は、更新後も
  描画テキストへのアサーション（`"Outcome: 🟡"` プレフィックス、
  `"provider finalization"` の復旧文言、`"决定: continue"` の人工決定文言など）
  がそのまま成功する ── これらのテストが検証するすべてのケースについて、
  新旧のコードパスがバイト単位で出力一致することを証明している。
- `cargo test --locked --workspace` は成功する（ワークスペース全体で120以上の
  test result ブロック、失敗0件）。

## 検証

`cargo fmt --all -- --check` と `cargo clippy --locked --workspace
--all-targets --all-features -- -D warnings` は成功する。CLI/MCP の JSON
出力、終了コード、ガバナンス上の意味論はいずれも変更していない ── 変更したのは
人間可読な handoff テキストを生成する内部の Rust 呼び出し経路のみである。

## 対象外

P0 の責任/依存関係マッピング（別 Work Item）、P1-B の観察コンテキスト、P2の
ライフサイクル/ストレージ抽出、P2-B の状態型、P2-C のマルチファイル一貫性、
P3 の物理実行境界は本 Work Item では対応していない。
