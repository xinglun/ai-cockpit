---
author: AI Cockpit maintainers
title: "WI-272——参考源 Agent 规则批次"
workItemId: WI-272-reference-agent-rule-batch
description: "逐文件比对参考源 Agent/rules 表面，将边界投影到 Rust 工程，不复制模板实现。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-272-reference-agent-rule-batch
authority: canonical
---

# WI-272——参考源 Agent 规则批次

## 意图

逐文件比对参考源 Agent 规则模板、风险门和回归测试语料。将治理含义保留在
repository-local guidance、生成的 Rust Agent adapter、typed Runtime 边界、测试和
parity evidence 中，但不复制参考源 Python 模块、Make 命令或 provider 全局配置。

## 范围

- 将交付顺序、retry checkout、Outcome 终态、事实证据和当前 Work Item 修复边界补入
  生成 adapter、`AGENTS.md`、`.ai/README.md` 和三语 Agent workflow 文档。
- 为投影规则增加 adapter 回归断言。
- 在固定版本台账中为四个延期的参考源 Agent/rules 文件记录准确 Rust counterpart
  和明确的有意差异原因。
- 本批次仅处理 Agent discovery/rule projection；Runtime 架构清洁以及无关 CI/release
  比对留到后续批次。

## 边界

参考源 Python 风险门和测试是规格证据，不是要复制的文件。已有 Rust
Contract/preflight/checkpoint/lifecycle 行为在其已具备的权威范围内进行映射和测试。
更深入的 typed checkpoint-evidence 或 repository-wide 并行 enforcement 缺口另行规划，
不能用文档声明掩盖。

## 验证

- 使用显式 `--repo` 的安装版 Runtime
- `cargo test --locked -p cockpit-agent --all-targets`
- reference inventory、parity、文档和 repository governance gates
- workspace 完整质量检查与 hosted checks
- 可见 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴`，包含状态、未知项、证据、人工决定和下一步

## 终态证据（计划）

- Archive：`.ai/work-items/archive/WI-272-reference-agent-rule-batch.contract.json`
- Verification：`.ai/evidence/WI-272-reference-agent-rule-batch.verification.json`
- Finalization：`.ai/decisions/WI-272-reference-agent-rule-batch.finalize.json`
- Close：`.ai/decisions/WI-272-reference-agent-rule-batch.close.json`
