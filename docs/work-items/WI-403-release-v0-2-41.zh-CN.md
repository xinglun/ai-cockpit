---
author: AI Cockpit maintainers
title: "WI-403——v0.2.41 发布与 adopter 验收"
description: "发布性能批次后的 Runtime，并在当前仓库和全新 adopter 中验证不可变公开制品。"
workItemId: WI-403-release-v0-2-41
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-403-release-v0-2-41
capabilityClaims: [release_distribution, adopter_acceptance, runtime_installation]
---

# WI-403——v0.2.41 发布与 adopter 验收

[English](WI-403-release-v0-2-41.md) · [日本語](WI-403-release-v0-2-41.ja.md)

## 意图

在 Rust 性能批次之后，从已审查且同步的 `main` 发布 v0.2.41，并证明不可变公开制品可以治理当前仓库和全新的 adopter。

## 边界

本 Work Item 只覆盖精确补丁版本发布、发布/分发文档、不可变制品安装和发布后 adopter 验收。不改变治理语义、参考源功能实现、业务工程或全局 Agent/MCP 配置。验收必须拒绝源码构建、workspace binary 和移动分支兜底。

## 验收

1. Cargo metadata 与 lockfile 从 v0.2.40 精确升级到 v0.2.41。
2. reviewed main 产生公开 Release，archive、SBOM、provenance、manifest 和 SHA-256 身份一致。
3. 下载的公开 binary 在当前仓库以显式 repository context 安装并验证，`inspect`、`status`、`doctor` 健康。
4. 全新隔离 adopter 完成 attach、scaffold、生命周期、证据复用与清理；`first-adopter-smoke` 保持 `not_ready`。
5. 保留 Runtime/repository identity、制品摘要、隔离 manifest、验收输出和清理事实。
6. 英文、简体中文、日文发布/parity 文档一致；精确清理后同步 main 报告 `ready_on_base`。

## 验证边界

发布与 adopter 验收是两种独立事实。发布后验收失败必须记录 `releasePublished: true` 与 `adopterAcceptance: failed`，不得改写已发布 Release 或历史证据。只有验收 harness 可以把下载的 binary Runtime identity 绑定到发布证据。
