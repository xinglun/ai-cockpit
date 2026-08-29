---
workItemId: WI-395-runtime-performance-extreme
title: Rust Runtime 性能优化
author: AI Cockpit maintainers
description: 在不削弱治理的前提下，以可测量方式减少重复 snapshot 与 Work Item status 工作。
type: implementation
audience: [adopter, contributor, maintainer, reviewer]
authority: human-authorized
status: implemented
lastVerifiedBy: WI-395-runtime-performance-extreme
terminalArchive: .ai/work-items/archive/WI-395-runtime-performance-extreme.contract.json
terminalVerification: .ai/evidence/WI-395-runtime-performance-extreme.verification.json
terminalFinalization: .ai/decisions/WI-395-runtime-performance-extreme.finalize.json
terminalDecision: .ai/decisions/WI-395-runtime-performance-extreme.close.json
---

# WI-395 — Rust Runtime 性能优化

[English](WI-395-runtime-performance-extreme.md) · [日本語](WI-395-runtime-performance-extreme.ja.md)

## 意图与安装边界

测量并降低 request-scoped status、observe 和聚合 Work Item 投影的 Rust
Runtime 成本。Runtime 仍是机器上共享安装的一份外部 binary。每个 adopter
都必须显式使用 `--repo` 并保持独立 `.ai/` 状态；本 WI 不复制参考源安装器、
SDK/工具链、Make/Python runtime 或 V1 wire 行为。

## 有界优化

- 聚合 Work Item status 在同一请求内复用带 identity 的 repository snapshot。
- 在已有 Git 索引读取中捕获 source-tree 摘要，并用一次受限 Git 查询解析远端默认元数据。
- 避免 repository observation 中反复递归排序。
- 保持变更、未知输入、必需检查、证据绑定和 fail-closed 决策不变。
- 记录前后样本，只报告本地成本事实，不冒充 provider 或企业 assurance。

## 验收边界

性能目标必须在声明的平台上用带 identity 的证据测量。目标未达成时保留可测量的
差距，不能通过跳过验证来“修复”。对象工程使用已安装或发布的 Runtime 重复相同的
冷/热序列，并保留各自的 repository 与 Runtime identity。
