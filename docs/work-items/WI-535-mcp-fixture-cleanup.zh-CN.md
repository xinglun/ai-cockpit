---
author: AI Cockpit maintainers
title: "WI-535——MCP 测试 fixture 清理"
description: "使 delegated-evidence 集成 fixture 在失败路径也安全清理，并登记受治理交付。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-535-mcp-fixture-cleanup
lastVerifiedBy: WI-535-mcp-fixture-cleanup
---

[English](WI-535-mcp-fixture-cleanup.md) · [日本語](WI-535-mcp-fixture-cleanup.ja.md)

## 目标

确保 delegated-evidence MCP 集成测试在成功、断言失败和 panic 时都删除临时仓库，
避免后续运行继承过期 Work Item 状态。

## 范围与边界

- `crates/cockpit-mcp/tests/rpc.rs` 及本 Work Item 的三语文档/Parity 投影。
- Runtime 生命周期语义、provider 状态和对象工程不在本 Work Item 范围内。

## 验收

- fixture 使用 RAII 临时目录所有者并可重复运行。
- 清理失败不会给后续测试留下重复 Work Item。
- archive 前登记到三份 Parity 台账。

## 验证

```text
cargo test --locked -p cockpit-mcp --test rpc delegated_evidence_list_exposes_only_repository_bound_receipts
tests/docs/parity_status_check.sh
tests/ci/governance_integrity_gate.py --repo <repo>
```
