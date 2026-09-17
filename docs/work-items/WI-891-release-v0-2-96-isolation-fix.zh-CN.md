---
author: AI Cockpit maintainers
workItemId: WI-891-release-v0-2-96-isolation-fix
title: v0.2.96 发布验证隔离修复
description: 修复不可变 v0.2.95 候选暴露的显式验证目标目录传递问题。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-891-release-v0-2-96-isolation-fix
---

# WI-891 — v0.2.96 发布验证隔离修复

本 successor 保留不可变的 v0.2.95 失败候选历史，并在 v0.2.96 发布前修复
Runtime 环境边界。使用固定可执行身份执行验证时，验收根显式提供的
`CARGO_TARGET_DIR` 必须保持不变。

## 验收边界

- 保留 v0.2.95 不可变，不修改对象仓库。
- 通过一个 PR 和 hosted checks 评审并合并 Runtime 修复。
- 只有候选和下载制品的安装、N-1 升级验收证明 HOME、XDG_CONFIG_HOME、
  CARGO_HOME 和 target 根隔离且已清理后，才发布 v0.2.96。
- 保留宿主展示和性能声明的未知项；本 WI 只修复发布隔离阻塞。

## 验证计划

使用 `CARGO_INCREMENTAL=0` 和共享验证目标运行定向环境测试，再运行声明的
发布及文档门禁。保留 v0.2.95 失败工作流作为外部证据，不因消息交付重复
验证。

