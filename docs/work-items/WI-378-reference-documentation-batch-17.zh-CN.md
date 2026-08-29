---
author: AI Cockpit maintainers
title: "WI-378 — 参考文档第 17 批"
description: "比对下一批固定参考源文档，并发布有界的 Rust 原生三语对应物。"
workItemId: WI-378-reference-documentation-batch-17
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-378-reference-documentation-batch-17
terminalArchive: .ai/work-items/archive/WI-378-reference-documentation-batch-17.contract.json
terminalVerification: .ai/evidence/WI-378-reference-documentation-batch-17.verification.json
terminalFinalization: .ai/decisions/WI-378-reference-documentation-batch-17.finalize.json
terminalDecision: .ai/decisions/WI-378-reference-documentation-batch-17.close.json
capabilityClaims: [reference_comparison, documentation_governance, adopter_readiness]
---

# WI-378 — 参考文档第 17 批

[English](WI-378-reference-documentation-batch-17.md) · [日本語](WI-378-reference-documentation-batch-17.ja.md)

## 目的

逐一比对固定参考清单的下一批十个路径，在共享 Rust Runtime 中提供面向读者的治理语义，
不复制源 Python、Make、provider 配置或历史决定。

## 路径与决定

| 固定参考路径 | 决定 |
| --- | --- |
| `docs/reference/remediation-instruction-traceability.json` | `reference-only`；生成的历史计划追踪不是目标 authority。 |
| `docs/reference/repository-workflow.ja.md` | Rust 原生三语工作流文档。 |
| `docs/reference/schemas.md` | Rust 原生三语记录族与校验映射。 |
| `docs/reference/test-architecture.md` | Rust 原生三语分层测试与证据模型。 |
| `docs/reference/test-weakening-guard.{md,zh-CN.md,ja.md}` | Rust 原生、基于快照的弱化路由和有界策略。 |
| `docs/reference/troubleshooting.{md,ja.md}` | Rust 原生显式仓库恢复与工具链边界。 |
| `docs/reference/upgrade.ja.md` | Rust 原生 Runtime 升级与仓库迁移边界。 |

源英文 `upgrade.md` 仍在 deferred 台账中，将在独立的有界批次比对；本批提供目标三语升级页面，
使选定的日语阅读路由完整。

## 边界

这是语义/文档 parity，不是源 JSON-wire、命令、Python、Make 或 provider parity。每个 adopter 使用同一
已安装 Runtime，并显式提供 `--repo`；仓库事实、Work Item、证据与决定保持隔离。文档不会创造 authority、
审批、assurance 或验证证据。

## 验收

- 每个选定路径都有分类以及目标对应物，或明确的 `reference-only` 决定。
- 英文、简体中文、日文阅读入口表达相同边界并互相链接。
- inventory 与 parity 台账在 source commit、Work Item、分类和零 `migrate-gap` 上一致。
- 文档/conformance 检查与已安装 v0.2.39 Runtime 验证通过。
- 保留 Contract 原始语言事实；不把语义 parity 表述为 wire 兼容。

## 验证计划

使用显式仓库上下文运行本地 inventory、文档、conformance、治理和已安装 Runtime 检查。终态 archive、
verification、finalization、close 收据只在 Runtime 验证后生成。

