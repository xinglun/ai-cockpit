---
author: AI Cockpit maintainers
title: "WI-415——v0.2.43 发布"
description: "从 WI-414 之后审查过的 Runtime 发布 v0.2.43，并建立下一次公开 artifact 验收基线。"
workItemId: WI-415-release-v0-2-43
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-415-release-v0-2-43
terminalArchive: .ai/work-items/archive/WI-415-release-v0-2-43.contract.json
terminalVerification: .ai/evidence/WI-415-release-v0-2-43.verification.json
terminalFinalization: .ai/decisions/WI-415-release-v0-2-43.finalize.json
terminalDecision: .ai/decisions/WI-415-release-v0-2-43.close.json
capabilityClaims: [release_distribution, repository_isolation]
sourceCommit: 107dfab6e6e331041a73fce7406f573bfbd7610c
canonical: docs/work-items/WI-415-release-v0-2-43.md
---

# WI-415——v0.2.43 发布

[English](WI-415-release-v0-2-43.md) · [日本語](WI-415-release-v0-2-43.ja.md)

## 意图

从审查后的 WI-414 `main` 发布 v0.2.43，并为独立的不可变公开 adopter
验收保留干净、经过审查的基础。

## 边界

本 Work Item 只推进 patch 版本，同步三语的发布、安装、版本和 parity 指引，
并验证严格发布 source route。不改变 Runtime 治理语义、历史 evidence、全局
Agent/MCP 配置或 adopter 应用源码。公开 artifact 的 adopter 验收仍由发布后的
独立 Work Item 负责。

## 验收标准

1. Cargo 元数据和 lockfile 从 v0.2.42 恰好推进到 v0.2.43，不复用已有 tag 或 Release。
2. 审查后的 release workflow 绑定准确的 reviewed commit、target archive、SBOM、manifest、Formula、SHA256SUMS、provenance 以及不可变 tag/Release 身份。
3. 英文、简体中文和日文的当前发布、安装、版本与 parity 文档保持同步，历史发布明确保留为历史记录。
4. 发布后验收只在独立 Work Item 中使用不可变公开 v0.2.43 artifact，不允许源码或 workspace fallback。
5. 经审查的合并、finalization、close、默认分支同步和精确 branch/worktree 清理后，`main` 达到 `ready_on_base`。

## 验证边界

发布前使用 Contract 声明的 strict source/release gates。本 Work Item 不把 staged
候选或源码构建当作公开 adopter evidence。任何发布或清理失败都必须保留可见，
不能改写已发布的 Release truth。

[English](WI-415-release-v0-2-43.md) · [日本語](WI-415-release-v0-2-43.ja.md)
