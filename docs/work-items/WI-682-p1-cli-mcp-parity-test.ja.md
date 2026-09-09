---
author: AI Cockpit maintainers
title: "WI-682 — P1 CLI/MCP Outcome 一致性テスト"
description: "WI-681 が指摘した不変量5の欠落を埋める:実際の CLI バイナリを起動し、同一リポジトリに対して本番の MCP handler を呼び出し、両者の Outcome 表現が完全に一致することを断言する新規統合テストを追加する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-682-p1-cli-mcp-parity-test
status: implemented
authority: authorized
lastVerifiedBy: WI-682-p1-cli-mcp-parity-test
terminalArchive: .ai/work-items/archive/WI-682-p1-cli-mcp-parity-test.contract.json
terminalVerification: .ai/evidence/WI-682-p1-cli-mcp-parity-test.verification.json
terminalFinalization: .ai/decisions/WI-682-p1-cli-mcp-parity-test.finalize.json
terminalDecision: .ai/decisions/WI-682-p1-cli-mcp-parity-test.close.json
---

[English](WI-682-p1-cli-mcp-parity-test.md) · [简体中文](WI-682-p1-cli-mcp-parity-test.zh-CN.md)

# WI-682 — P1 CLI/MCP Outcome 一致性テスト

## 意図

WI-681 のカバレッジ対応付け(`docs/reference/collaboration-invariant-coverage.md`、
本 Work Item の時点ではまだデフォルトブランチに存在しない。PR 作成待ち)
が指摘した不変量5の欠落を埋める:「同一の事実が CLI・MCP・要約・完全なレ
ポートで一致する」ことについて、真にプロセスをまたいだ自動チェックがこれ
まで存在しなかった。本 Work Item は、`crates/cockpit-cli/tests/outcome_handoff.rs`
のサブプロセス起動パターンと `crates/cockpit-mcp/tests/rpc.rs` のプロセス内
MCP handler 呼び出しパターンという、既に実証済みの二つのパターンを組み合
わせた新規統合テストを一件だけ追加するものであり、新しいテスト基盤を導入
するものではない。リポジトリ所有者からの明示的な委任に基づき、AI Cockpit
協作言語専項を継続する。

## 境界

これはテストのみの Work Item である。
`crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs` という新規ファイルを
1件追加するのみで、本番のソースファイルは一切変更せず、既存のテストファ
イルも一切変更しない。

## 受け入れとライフサイクル

- 新規テストは、同一のリポジトリ fixture と同一の Work Item に対して、
  CLI 側では実際の `ai-cockpit` バイナリをサブプロセスとして起動し、MCP
  側ではプロセス内で `cockpit_mcp::handle_request_for_repo()` を呼び出し、
  両者の Outcome 表現が完全に等しいことを断言する。
- `cargo test --locked -p cockpit-cli --test cli_mcp_outcome_parity`、
  `cargo fmt --check`、`cargo clippy --tests -- -D warnings` がすべて合格
  する。
- `start → preflight → checkpoint → verify → finish → archive → close` が統治
  された経路であり、`user_visible_benefit_not_declared` は明示されたまま
  である。

## 証拠

- archive: `.ai/work-items/archive/WI-682-p1-cli-mcp-parity-test.contract.json`
- verification: `.ai/evidence/WI-682-p1-cli-mcp-parity-test.verification.json`
- finalization: `.ai/decisions/WI-682-p1-cli-mcp-parity-test.finalize.json`(合併後に生成予定)
- close: `.ai/decisions/WI-682-p1-cli-mcp-parity-test.close.json`(合併後に生成予定)
