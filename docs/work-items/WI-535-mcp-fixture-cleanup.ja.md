---
author: AI Cockpit maintainers
title: "WI-535 — MCP テストフィクスチャのクリーンアップ"
description: "delegated-evidence 統合フィクスチャを失敗時にも安全にクリーンアップし、三言語の記録を登録する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-535-mcp-fixture-cleanup
lastVerifiedBy: WI-535-mcp-fixture-cleanup
terminalArchive: .ai/work-items/archive/WI-535-mcp-fixture-cleanup.contract.json
terminalVerification: .ai/evidence/WI-535-mcp-fixture-cleanup.verification.json
terminalFinalization: .ai/decisions/WI-535-mcp-fixture-cleanup.finalize.json
terminalDecision: .ai/decisions/WI-535-mcp-fixture-cleanup.close.json
---

[English](WI-535-mcp-fixture-cleanup.md) · [简体中文](WI-535-mcp-fixture-cleanup.zh-CN.md)

## Goal

delegated-evidence MCP 統合テストの一時リポジトリを、成功・アサーション
失敗・panic のいずれでも破棄し、後続実行に古い Work Item 状態を残さない。

## Scope and boundary

- `crates/cockpit-mcp/tests/rpc.rs` と本 Work Item の三言語ドキュメント/
  parity 投影。
- Runtime のライフサイクル意味論、provider 状態、対象リポジトリは対象外。

## Acceptance

- フィクスチャが RAII の一時ディレクトリ所有者を使用し、再実行できる。
- クリーンアップ失敗で後続テストに重複 Work Item を残さない。
- archive 前に三つの parity 台帳へ登録される。

## Verification

```text
cargo test --locked -p cockpit-mcp --test rpc delegated_evidence_list_exposes_only_repository_bound_receipts
tests/docs/parity_status_check.sh
tests/ci/governance_integrity_gate.py --repo <repo>
```
