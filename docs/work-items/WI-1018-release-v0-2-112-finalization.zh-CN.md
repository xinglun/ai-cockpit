---
author: AI Cockpit maintainers
workItemId: WI-1018-release-v0-2-112-finalization
title: v0.2.112 修正版发布收尾
description: 在不可变的 v0.2.111 取消候选版本基础上，继续完成修正版四目标发布、公开验收和精确生命周期清理。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1018-release-v0-2-112-finalization
---

[English](WI-1018-release-v0-2-112-finalization.md) · [日本語](WI-1018-release-v0-2-112-finalization.ja.md)

# WI-1018 — v0.2.112 发布收尾

本 successor 从同步后的 main 发布修正版 v0.2.112。它保留不可变的 v0.2.111 五目标取消候选记录，并记录公开 Release、下载验收、Runtime 生命周期关闭、文档投影和精确清理证据。

## 边界

- 正式发布目标为四个：`aarch64-apple-darwin`、`aarch64-unknown-linux-gnu`、`x86_64-unknown-linux-gnu` 和 `x86_64-pc-windows-msvc`。
- 不生成 Intel macOS 发布资产，不新增任务进度账本，也不修改全局 Agent/MCP 配置。
- 绿色构建或公开 Release 本身不证明用户可见收益；该收益仍是独立证据边界。
