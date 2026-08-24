---
author: AI Cockpit maintainers
title: "WI-186——v0.2.23 发布后公开 adopter 验收"
workItemId: WI-186-release-v0-2-23-post-release-acceptance
description: "记录不可变的 v0.2.23 公开 Runtime 能否从零治理 adopter 与 N-1 升级。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-186-release-v0-2-23-post-release-acceptance
---

# WI-186——v0.2.23 发布后公开 adopter 验收

WI-186 固化安装公开 v0.2.23 Runtime 后的下一轮基线。验收只使用下载的
Release binary，不使用 Cargo 构建、`cargo run`、workspace binary 或本地
`target/` fallback。

不可变 Runtime identity 保存在
`.ai/evidence/external/v0.2.23/adopter/runtime.json`。公开 adopter 验收和
v0.2.22 → v0.2.23 的 N-1 升级验收各自保留 `acceptance.json`、close 回执、
隔离 manifest、清理回执和 `SHA256SUMS`。

Adopter 证据证明 attach、Agent discovery、evidence reuse、
`first-adopter-smoke` 的 `not_ready` 边界以及完整 Work Item lifecycle 都能
在隔离仓库中工作。HOME 和 XDG 配置保持不变；临时目录与 Cargo 目录明确
分类并在结束时清理。

本 Work Item 不改写 Release、tag 或历史 evidence。它记录公开事实，使下一
个 Work Item 可以只使用已安装的 v0.2.23 Runtime 作为治理接口。

证据：`.ai/evidence/external/v0.2.23/adopter/acceptance.json` 与
`.ai/evidence/external/v0.2.23/upgrade/acceptance.json`。

[English](WI-186-release-v0-2-23-post-release-acceptance.md) ·
[日本語](WI-186-release-v0-2-23-post-release-acceptance.ja.md)
