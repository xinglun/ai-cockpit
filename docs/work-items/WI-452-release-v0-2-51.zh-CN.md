---
author: AI Cockpit maintainers
title: "WI-452——发布 v0.2.51"
workItemId: WI-452-release-v0-2-51
description: "从不可变公开制品发布并验收 v0.2.51 Runtime。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-452-release-v0-2-51
---

# WI-452——发布 v0.2.51

从审查合并且已同步的 `main` 发布 v0.2.51，并仅使用不可变公开制品完成
安装与 adopter 验收。失败的历史标签保持不可变且绝不复用。

[English](WI-452-release-v0-2-51.md) · [日本語](WI-452-release-v0-2-51.ja.md)

## 范围

- 将 workspace package identity 更新为 v0.2.51。
- 同步当前三语 release、distribution、architecture 与 versioning 文档。
- 发布前运行 release policy、source archive、checksum/SBOM 与 locked workspace gate。
- 合并后发布不可变标签，只安装下载制品，并运行隔离 adopter 验收。

## 边界

不修改对象工程、用户全局 Agent/MCP 配置或失败的 release 标签。源码 checkout
和 workspace binary 不得作为发布验收输入。

## 验证

- `cargo test --locked --workspace`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/release/release_policy_test.sh`
- `bash tests/release/release_source_archive_test.sh`
- `bash tests/release/release_version_consistency_test.sh`
