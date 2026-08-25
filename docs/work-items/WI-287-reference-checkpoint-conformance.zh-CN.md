---
author: AI Cockpit maintainers
title: "WI-287 — 参考源 checkpoint 一致性"
workItemId: WI-287-reference-checkpoint-conformance
description: "以 Rust-native fail-closed regression 和真实台账关闭参考源 checkpoint 文件比对缺口。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-287-reference-checkpoint-conformance
authority: canonical
---

# WI-287 — 参考源 checkpoint 一致性

## 目的

本有界批次关闭参考源 checkpoint 的逐文件比对缺口，使 conformance 台账
真实可审计。不会把 Python、Make、YAML 或 V1 wire format 复制进 Runtime。

## 比对文件

| 参考文件 | Rust 对应 | 边界 |
| --- | --- | --- |
| `scripts/ai_checkpoint.py` | `cockpit-protocol` typed `CheckpointPolicy`/`CheckpointEvidence` 与 repository checkpoint/amendment validator | 语义 parity，不声明直接 JSON-wire 兼容 |
| `tests/test_ai_checkpoint.py` | `agent_risk_checkpoint.rs`、`lifecycle_order.rs` | 覆盖顺序、resume stale、amendment lineage 与证据不可替换 |
| `tests/test_outcome_lifecycle_rules.py` | `agent_rule_parity.rs`、`AGENTS.md`、`.ai/README.md`、`docs/reference/agent-workflow.md` | 项目原生 Agent 规则投影，不复制模板 |

## 变更

- 已存在任何 verification result 时，`before_edit` checkpoint fail-closed。
- 最新 `resumeHistory.recordedAt` 无效时拒绝，不再静默当作缺失。
- 加强 current Work Item 修复、可见 Outcome 终态和窄 successor 规则的静态 parity 断言。
- 将两个 checkpoint 源文件登记为 `implemented-different-by-design`。

## 对象工程边界

这些控制属于 Runtime 与 repository protocol，因此新 adopter 也继承相同
行为。所有命令仍必须显式指定 repository；unknown 保持可见，human Outcome
仍是交付边界。

## 验证

执行 `cargo test --locked --workspace`、conformance 台账回归、文档验收和
repository governance gate。
