---
author: AI Cockpit maintainers
workItemId: WI-1025-release-v0-2-113
title: v0.2.113 发布
description: 将已审查的 main 版本作为四目标 v0.2.113 发布，并完成公开验收和精确生命周期清理。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1025-release-v0-2-113
---

[English](WI-1025-release-v0-2-113.md) · [日本語](WI-1025-release-v0-2-113.ja.md)

# WI-1025——v0.2.113 发布

本 Work Item 将已审查的 `main` 版本发布为 v0.2.113。它绑定不可变 tag、公开 Release、四个受支持目标的制品、下载验收、Runtime 生命周期关闭、文档投影和精确清理证据。

## 边界

- 正式发布目标为四个：`aarch64-apple-darwin`、`aarch64-unknown-linux-gnu`、`x86_64-unknown-linux-gnu` 和 `x86_64-pc-windows-msvc`。
- 不生成 Intel macOS 发布资产，不新增任务进度账本，不改写历史 Release 或 tag，也不修改全局 Agent/MCP 配置。
- 绿色构建或公开 Release 本身不证明用户可见收益；该收益仍是独立证据边界。
