---
author: AI Cockpit maintainers
title: "WI-150——v0.2.16 发布基线"
description: "准备 v0.2.16 不可变 Runtime 发布，并保持源码、文档和发布身份一致。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-150-release-v0-2-16
workItemId: WI-150-release-v0-2-16
---

# WI-150——v0.2.16 发布基线

WI-150 对齐了 v0.2.16 Runtime 的 workspace metadata、lockfile、发布文档和发布策略检查。
在 Runtime 验证路线继续收敛期间，CI 中既有 Cargo 检查仍作为 shadow comparison 保留；本
Work Item 不改变治理语义，也不改写历史记录。

不可变公开 Release 是 [v0.2.16](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.16)，
绑定 tag commit `521177b`。完整发布 workflow（构建、manifest、checksum、SBOM、provenance、
smoke、adopter 和 N-1 验收）记录在
[workflow run 32602194567](https://github.com/xinglun/ai-cockpit/actions/runs/32602194567)。

本 Work Item 的本地验证证据是 `.ai/evidence/WI-150-release-v0-2-16.verification.json`。
公开发布和安装 Runtime 验收属于发布后独立证据，由 WI-151 投影，不会回写本归档记录。
