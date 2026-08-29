---
author: AI Cockpit maintainers
title: "WI-374——v0.2.39 发布与精确验证复用验收"
description: "在修复恢复状态 parity 投影后发布动态验证复用 Runtime，并在隔离 repository 中验收公开 artifact。"
workItemId: WI-374-release-v0-2-39
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-374-release-v0-2-39
terminalArchive: .ai/work-items/archive/WI-374-release-v0-2-39.contract.json
terminalVerification: .ai/evidence/WI-374-release-v0-2-39.verification.json
terminalFinalization: .ai/decisions/WI-374-release-v0-2-39.finalize.json
terminalDecision: .ai/decisions/WI-374-release-v0-2-39.close.json
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-374——v0.2.39 发布与精确验证复用验收

[English](WI-374-release-v0-2-39.md) · [日本語](WI-374-release-v0-2-39.ja.md)

## 意图

从经过审查且已同步的 `main` 准备 v0.2.39，使动态、身份绑定的精确验证复用优化可以发布。发布前修复 WI-370 与 WI-371 恢复 receipt 的 parity 投影阻断问题。公开 artifact 与 adopter 验收明确交由不可变 tag 产生后的后继 WI-376。

## 范围与边界

- 在三种支持语言中同步 Cargo metadata、lockfile、版本、发布与分发文档。
- 三语 parity ledger 引用带摘要后缀的权威恢复 receipt，不重写前置 evidence。
- 执行不可变 tag 前所需的严格 release policy 与 staged 检查；发布前不宣称公开 artifact。
- 保留发布后交接，由后继 WI-376 只下载公开 artifact 并安装到本 repository 与全新隔离 adopter。

Runtime 语义、历史 evidence 重写、全局 Agent/MCP 配置、源码构建 fallback，以及第二技术栈 adopter 不属于本 Work Item。

## 验收

1. Cargo metadata 与 lockfile 一致报告 v0.2.39。
2. 恢复 parity 行指向权威恢复 receipt，严格文档/治理 gate 全部通过。
3. 公开 Release asset、校验和、SBOM、Formula 与 provenance 的验收明确延后到后继 WI-376，本 WI 不作声明。
4. 公开 binary 身份与无 fallback 验收明确延后到后继 WI-376。
5. 精确复用与全新 adopter 验收明确延后到后继 WI-376。
6. 隔离、清理、生命周期和失败时 Release truth 不变的验收明确延后到后继 WI-376。
7. 完成审查合并、finalization、close、默认分支同步和精确分支/工作树清理后，repository 为 `ready_on_base`。

## 验证边界

发布前使用严格 repository gate manifest 与 staged 检查。发布后验收属于后继 WI-376：只下载不可变的 v0.2.39 artifact，并记录 tag、archive 摘要、binary 摘要、平台和来源。优化只复用完全匹配的验证；首次或失效验证仍会执行，测得收益不外推到这些路径。
