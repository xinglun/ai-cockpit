---
author: AI Cockpit maintainers
title: WI-404 —— 发布文档终态晋级
description: 仅在不可变生命周期证据齐备后晋级已完成 Work Item 的文档。
workItemId: WI-404-release-docs-terminal-promotion
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-404-release-docs-terminal-promotion
---

# WI-404 —— 发布文档终态晋级

本 Work Item 修复 v0.2.41 发布质量门暴露的文档投影问题。它只晋级已完成
WI-402 的三语文档和 parity 行，不改写任何不可变的 `.ai` evidence 或 decision。

## 边界

- 仅更新 WI-402 三语 Work Item 页面和三语 reference parity 行。
- 保留 archive、verification、finalization、close 作为不可变证据引用。
- 不修改 Runtime 语义，也不发布 Release。

## 验证

由已安装 Runtime 记录 repository-bound verification evidence。文档、晋级、parity、
库存和完整 workspace 检查必须在评审前通过。
