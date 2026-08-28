---
author: AI Cockpit maintainers
title: "WI-374——v0.2.39 发布与精确验证复用验收"
description: "在修复恢复状态 parity 投影后发布动态验证复用 Runtime，并在隔离 repository 中验收公开 artifact。"
workItemId: WI-374-release-v0-2-39
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-374-release-v0-2-39
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-374——v0.2.39 发布与精确验证复用验收

[English](WI-374-release-v0-2-39.md) · [日本語](WI-374-release-v0-2-39.ja.md)

## 意图

从经过审查且已同步的 `main` 发布 v0.2.39，使动态、身份绑定的精确验证复用优化同时服务当前 repository 和未来 adopter。发布前修复 WI-370 与 WI-371 恢复 receipt 的 parity 投影阻断问题。

## 范围与边界

- 在三种支持语言中同步 Cargo metadata、lockfile、版本、发布与分发文档。
- 三语 parity ledger 引用带摘要后缀的权威恢复 receipt，不重写前置 evidence。
- 只发布严格 release workflow 产生的不可变、带校验和、SBOM 绑定及 provenance 绑定的 artifact。
- 只从公开下载 artifact 安装到本 repository 与全新隔离 adopter，并保留 Runtime、repository 隔离和精确复用 evidence。

Runtime 语义、历史 evidence 重写、全局 Agent/MCP 配置、源码构建 fallback，以及第二技术栈 adopter 不属于本 Work Item。

## 验收

1. Cargo metadata 与 lockfile 一致报告 v0.2.39。
2. 恢复 parity 行指向权威恢复 receipt，严格文档/治理 gate 全部通过。
3. 公开 Release 包含目标 archive、manifest、SHA256SUMS、目标绑定 SBOM、Formula 与 provenance evidence。
4. 安装的公开 binary 版本和摘要绑定在验收 receipt 中；不使用源码或 workspace fallback。
5. 当前 repository 与全新 adopter 复用有效精确 evidence；变化、过期或未知输入必须重新执行或 fail-closed 停止。
6. 验收证明 HOME/XDG 隔离、允许的 Runtime 写入根、清理、生命周期 evidence，以及失败时不修改 Release truth。
7. 完成审查合并、finalization、close、默认分支同步和精确分支/工作树清理后，repository 为 `ready_on_base`。

## 验证边界

发布前使用严格 repository gate manifest 和 staged artifact 验收。发布后只下载不可变的 v0.2.39 artifact，并记录 tag、archive 摘要、binary 摘要、平台和来源。优化只复用完全匹配的验证；首次或失效验证仍会执行，测得收益不外推到这些路径。
