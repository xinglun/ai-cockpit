---
author: AI Cockpit maintainers
workItemId: WI-1028-release-v0-2-113-strict-lineage-repair
title: 严格发布证据 lineage 修复
description: 为 v0.2.113 历史发布证据绑定 Runtime 创建的严格 successor，并在不改写历史证据的前提下收敛文档投影。
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1028-release-v0-2-113-strict-lineage-repair
---

[English](WI-1028-release-v0-2-113-strict-lineage-repair.md) · [日本語](WI-1028-release-v0-2-113-strict-lineage-repair.ja.md)

# WI-1028 — 严格发布证据 lineage 修复

本 Work Item 通过受支持的 Runtime successor 路径记录 v0.2.113 的归档发布证据，保留前序字节，并使终态文档投影可确定收敛。

## 边界

- 复用既有发布、验证和恢复证据。
- 不改写历史归档记录，不重跑发布或 workspace。
- 不增加任务进度账本，不改变产品行为。
