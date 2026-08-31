---
author: AI Cockpit maintainers
title: "WI-441——本地参考源入口与 Agent 语义对齐"
workItemId: WI-441-reference-entrypoint-parity
description: "重新阅读本地参考源的 Agent 规则、读者入口、能力边界和 Task Outcome 入口。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-441-reference-entrypoint-parity
terminalArchive: .ai/work-items/archive/WI-441-reference-entrypoint-parity.contract.json
terminalVerification: .ai/evidence/WI-441-reference-entrypoint-parity.verification.json
terminalFinalization: .ai/decisions/WI-441-reference-entrypoint-parity.finalize.json
terminalDecision: .ai/decisions/WI-441-reference-entrypoint-parity.close.json
---

# WI-441——本地参考源入口与 Agent 语义对齐

本 Work Item 逐个重新阅读维护者本地参考 checkout
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` 中的 9 个文件，固定在提交
`fde3380f81fea5fd2e288f7a8849f737dc074060`。不访问公开参考源。结果是 Rust 目标的文件级语义决定，
不是复制源工程 Agent 文件、Python/Make 命令或源 JSON wire format 的要求。

[English](WI-441-reference-entrypoint-parity.md) · [日本語](WI-441-reference-entrypoint-parity.ja.md)

## 范围与逐文件决定

| 本地参考路径 | 分类 | Rust 对应物或边界 |
| --- | --- | --- |
| `AGENTS.md` | `implemented-different-by-design` | `AGENTS.md`、`.ai/README.md`、`docs/reference/agent-workflow.md` 与 typed lifecycle/adapter 服务保留 Contract-first、最新 base、人工暂停、关闭和清理语义；源 `make ai-*` 不是目标命令。 |
| `GEMINI.md` | `implemented-different-by-design` | `.ai/README.md`、`crates/cockpit-agent`、安装测试和显式 Gemini adapter 路线提供 provider-facing 指导，但不提交 provider 专用文件，也不修改全局配置。 |
| `docs/README.md` | `implemented-different-by-design` | 目标的 current/getting-started/operations/reference 路线保留 reader-first、goal-first 入口，并明确 Rust 边界。 |
| `docs/README.zh-CN.md` | `implemented-different-by-design` | 中文读者入口和互链保留相同意图，并明确 Runtime/adopter 边界。 |
| `docs/README.ja.md` | `implemented-different-by-design` | 日文读者入口和互链保留相同意图，并明确 Runtime/adopter 边界。 |
| `docs/capabilities.md` | `implemented-different-by-design` | 目标能力页保留 Repository Governance Layer 与外部非声明，并补充 Rust CLI、MCP、scaffold、profile、knowledge、Outcome 和隔离路径。 |
| `docs/capabilities.zh-CN.md` | `implemented-different-by-design` | 中文能力路线保留源边界，说明 repository-local Runtime/adopter 继承，不复制源状态。 |
| `docs/capabilities.ja.md` | `implemented-different-by-design` | 日文能力路线保留源边界，说明 repository-local Runtime/adopter 继承，不复制源状态。 |
| `docs/features/task-outcome-report.md` | `implemented-different-by-design` | `OutcomeV2`、CLI/MCP 面向人的 handoff 和不可变 evidence 保留 report/status/PR 分离；源报告文字和 Make 命令不是 wire 要求。 |

## 边界决定

当前参考源本身是 specification corpus，且已经切换为私有本地源。某个源文件不在 Rust repository 中，
只有在其可迁移责任已由共享 Runtime、repository-local `.ai/` 路线或显式 provider adapter 提供时，才不构成遗漏。
反之，源特有命令和生成记录仍是 reference-only。对象工程继承同一边界：机器只安装一份 Runtime，调用显式绑定
repository context，evidence 保存在各自 repository，Agent 配置只有在 owner 显式安装 adapter 时才由 provider 负责。

## 验证边界

inventory 保留 `sourceChangedSincePrevious`、`previousBatch` 和 `previousClassification` 审计历史，同时解决本批 9 个当前记录。
本地参考策略要求 checkout 干净且提交匹配 lock，`network_access = false`；托管 CI 只使用已提交离线语料。文档验收、parity、
governance integrity、status consistency 和 locked workspace tests 是本 Work Item 的验证证据。
