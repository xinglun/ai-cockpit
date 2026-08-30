---
author: AI Cockpit maintainers
title: "WI-409——v0.2.41 发布与 adopter 验收"
description: "发布 WI-408 之后的审查 Runtime，并在全新 adopter 中验证不可变制品。"
workItemId: WI-409-release-v0-2-41
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-409-release-v0-2-41
capabilityClaims: [release_distribution, adopter_acceptance, repository_isolation]
---

# WI-409——v0.2.41 发布与 adopter 验收

[English](WI-409-release-v0-2-41.md) · [日本語](WI-409-release-v0-2-41.ja.md)

## 意图

从 WI-408 之后经过审查的 `main` 发布 v0.2.41，并证明下载的不可变
Release binary 可以在全新 adopter 中 attach 和治理，而不会复制参考源或
V1 Runtime 残留。

## 边界

本 Work Item 只推进 patch 版本、更新当前三语发布/版本文档、执行严格发布
workflow，并记录公开 adopter/N-1 验收。不改变治理语义、历史 evidence、全局
Agent/MCP 配置或无关 adopter 业务源代码。Runtime 分发与 repository attach
保持分离。

## 验收

1. Cargo metadata 与 lockfile 从 v0.2.40 正好推进一个 patch 到 v0.2.41，
   不复用既有标签或 Release。
2. 经审查的 workflow 从精确合并提交构建声明的 target，并绑定 manifest、
   Formula、SHA256SUMS、SBOM、provenance 及不可变 tag/Release identity。
3. 从公开 v0.2.41 Release 下载的 binary 经过 checksum 校验，并记录 Runtime
   版本、archive digest、binary digest、平台和下载来源；不使用源码或 workspace
   fallback。
4. 全新 adopter 验收证明 attach/profile/agent doctor、`first-adopter-smoke`
   的 `not_ready` 边界、生命周期与 evidence reuse、repository/Runtime 隔离和
   临时目录清理。
5. 当前 repository 与全新 adopter 通过共享 Runtime 继承 WI-408 的只读
   `work-item inspect` 边界。
6. 审查合并、finalization、close、同步、精确分支清理和发布文档晋升完成后，
   `main` 保持 ready on base。

## 验证边界

发布前使用严格 source/staged gate；发布后只下载不可变公开制品并持久化
Runtime、checksum、adopter、隔离与清理 evidence。发布失败不能改写已发布的
Release truth。ORG-X 等 adopter 只做不复制参考源残留的检查。
