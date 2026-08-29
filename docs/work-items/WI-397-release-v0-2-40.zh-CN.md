---
author: AI Cockpit maintainers
title: "WI-397——v0.2.40 发布与公开性能继承"
description: "发布 WI-396 clean-snapshot 优化，并在本 repository 与全新 adopter 中验证下载的发布 binary。"
workItemId: WI-397-release-v0-2-40
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-397-release-v0-2-40
terminalArchive: .ai/work-items/archive/WI-397-release-v0-2-40.contract.json
terminalVerification: .ai/evidence/WI-397-release-v0-2-40.verification.json
terminalFinalization: .ai/decisions/WI-397-release-v0-2-40.finalize.json
terminalDecision: .ai/decisions/WI-397-release-v0-2-40.close.json
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-397——v0.2.40 发布与公开性能继承

[English](WI-397-release-v0-2-40.md) · [日本語](WI-397-release-v0-2-40.ja.md)

## 意图

从经过审查的 `main` 发布 v0.2.40，使 WI-396 的 Rust clean-snapshot 快速路径通过共享外部 Runtime 可用。发布和 adopter 验收必须使用不可变公开 artifact 并保持 repository 隔离；源码构建不构成安装证据。

## 边界

本 Work Item 只推进 patch 版本、同步发布工作流与三语分发文档，并执行公开 Release adopter 与 N-1 验收。不改变治理语义、历史 evidence、全局 Agent/MCP 配置，也不为通过检查而修改性能预算。每个 adopter 都使用显式 `--repo` 绑定共享 binary，并保留自己的 `.ai/` 状态。

## 验收

1. Cargo metadata 与 lockfile 从 v0.2.39 正好推进一个 patch 到 v0.2.40；不复用已有 tag 或 Release。
2. 经审查的 workflow 从精确合并提交构建所有声明的 target，并绑定 manifest、Formula、SHA256SUMS、按 target 的 SBOM、provenance 和不可变 tag/Release identity。
3. 从公开 Release 下载的 v0.2.40 binary 经过 checksum 校验，并记录版本、binary 摘要、平台和 Runtime identity；不使用源码或 workspace fallback。
4. 公开 adopter 验收证明 attach/profile/Agent doctor、`first-adopter-smoke` 的 `not_ready` 边界、生命周期与 evidence reuse、隔离、清理及 repository/runtime identity。
5. 适用时执行 N-1 验收，保留历史 bytes；发布后检查失败仍记录 `releasePublished: true`。
6. 当前 repository 与全新 adopter 通过共享 Runtime 继承 WI-396 的实测 clean-snapshot 优化，不引入全局 repository 或跨 repository cache。
7. 审查合并、finalization、close、同步和精确清理完成后，`main` 为 `ready_on_base`，没有未关闭 PR 或 `codex/*` 分支。

## 验证边界

发布前使用严格 source 与 staged release gate。发布后只下载不可变公开 artifact，并持久化 Runtime、adopter、隔离、清理和 checksum evidence。发布不从本地测量推导 provider 或 enterprise 性能声明。
