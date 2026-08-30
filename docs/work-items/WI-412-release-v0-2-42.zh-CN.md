---
author: AI Cockpit maintainers
title: "WI-412——v0.2.42 发布准备"
description: "发布 WI-411 之后审查过的 Runtime，并为独立的公开 adopter 验收保留清洁基线。"
workItemId: WI-412-release-v0-2-42
audience: [adopter, maintainer, reviewer]
status: recovered
authority: human-authorized
lastVerifiedBy: WI-412-release-v0-2-42
capabilityClaims: [release_distribution, repository_isolation]
---

# WI-412——v0.2.42 发布准备

[English](WI-412-release-v0-2-42.md) · [日本語](WI-412-release-v0-2-42.ja.md)

## 目标

从审查后的 WI-411 `main` 发布 v0.2.42，并为独立的不可变公开 adopter
验收留下清洁、经过审查的基线。

## 边界

本 Work Item 只推进 patch 版本、同步三种语言的当前发布与版本文档、验证
严格发布源流程并记录审查后的生命周期。不改变 Runtime 治理语义、历史
证据、全局 Agent/MCP 配置或 adopter 应用源代码。公开 artifact 的 adopter
验收属于独立的发布后 Work Item，本 Work Item 不宣称已经完成。

## 验收

1. Cargo 元数据和 lockfile 从 v0.2.41 恰好推进到 v0.2.42，不复用已有 tag 或 Release。
2. 审查后的发布 workflow 绑定确切审查提交、目标 archive、SBOM、manifest、Formula、
   SHA256SUMS、provenance 以及不可变 tag/Release 身份。
3. 当前发布、安装、版本和 parity 文档在英文、简体中文、日文之间保持同步，历史发布明确标记为历史。
4. 发布后验收在独立隔离 Work Item 中只使用不可变公开 v0.2.42 artifact，不允许源码或 workspace fallback。
5. 审查合并、finalization、close、默认分支同步和精确 branch/worktree 清理后，`main` 达到 `ready_on_base`。

## 验证边界

发布前使用声明的 strict source/release gate。本 Work Item 不得把 staged candidate 或源码构建冒充公开 adopter 证据。任何发布或清理失败都必须保留可见，不能改写已发布的 Release truth。
