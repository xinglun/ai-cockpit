# WI-1088-runtime-activation-amendment-continuity-recovery — Runtime activation の連続性と現時点の復旧

技術スコープ承認済み。ネイティブ scope 修訂と実際の人間判断ゲートは未完了。製品受入: not_run。

実際の前向き A/B/owner/Runtime/journal フェーズ証拠を保持する。旧履歴の復旧は正確な現時点 baseline の受入のみで、履歴の不明点を保持し、新しい preflight の生成だけを許す。B のレビューや CSV は一般的な免除ではない。

承認された範囲は以下の Rust 12 パス、三言語 commands、本 WI の governance/brief、および三言語 reference-parity の WI-1088 の実際の段階行のみ。

- `crates/cockpit-repository/src/contract_amendment.rs`
- `crates/cockpit-repository/tests/contract_amendment.rs`
- `crates/cockpit-cli/tests/contract_amendment_processes.rs`
- `crates/cockpit-repository/src/lifecycle.rs`
- `crates/cockpit-protocol/src/lib.rs`
- `crates/cockpit-repository/src/status_projection.rs`
- `crates/cockpit-repository/src/lib.rs`
- `crates/cockpit-cli/src/main.rs`
- `crates/cockpit-mcp/src/lib.rs`
- `crates/cockpit-repository/tests/lifecycle_entry.rs`
- `crates/cockpit-mcp/tests/material_review.rs`
- `crates/cockpit-mcp/tests/usage.rs`

実際の人間ゲートの後、まず一時 Git repository で RED を再現し、その後に修正・否定テスト・全検証を行う。WI-1087 の実復旧、merge、close、delete は行わない。

Plan: `../../.ai/evidence/WI-1088-runtime-activation-amendment-continuity-recovery/plan-v2-20261017/implementation-plan.v2.md`. Source: `../../.ai/evidence/WI-1088-runtime-activation-amendment-continuity-recovery/plan-v2-20261017/technical-approval-source.json`.
