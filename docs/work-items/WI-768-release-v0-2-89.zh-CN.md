---
author: AI Cockpit maintainers
title: WI-768——v0.2.89 发布恢复
description: 修复严格发布质量边界，并在保留 v0.2.88 失败历史后发布新的不可变版本。
audience: [adopter, maintainer, reviewer]
workItemId: WI-768-release-v0-2-89
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-768-release-v0-2-89
capabilityClaims: [release_distribution, adopter_acceptance, governance_evidence]
---

[English](WI-768-release-v0-2-89.md) · [日本語](WI-768-release-v0-2-89.ja.md)

# WI-768——v0.2.89 发布恢复

## 意图

在不改写 v0.2.88 不可变失败历史的前提下恢复发布路径，并发布 v0.2.89；发布 workflow
必须把当前 Contract-aware Rust gate 与 repository gate receipt 绑定。

## 边界

本 Work Item 只修改发布 workflow 的质量绑定、Runtime 包版本、当前发布/版本文档，以及
验证公开产物和 adopter 边界所需的 evidence。不修改 Runtime 生产行为或性能实现，不改写
WI-764/v0.2.88 历史 bytes，不修改全局 Agent/MCP 配置，不复用 tag，也不预创建 provider Release。

## 验证

reviewed PR 的 hosted checks 和 v0.2.89 发布 workflow 必须证明 Contract-aware source-quality
receipt、repository gate receipt、五个 target archive、SBOM/来源证明、manifest/checksum 及
发布 identity。公开 adopter 安装和 v0.2.87 到 v0.2.89 的升级验收只能使用下载的不可变产物，
并保留隔离与清理 evidence。

Work Item 完成 Runtime lifecycle 后，在此补充终态 Contract、验证、发布、finalization、清理和决定路径。
