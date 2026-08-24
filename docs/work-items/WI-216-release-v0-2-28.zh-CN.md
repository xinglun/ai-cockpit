---
author: AI Cockpit maintainers
title: "WI-216——v0.2.28 不可变发布与 adopter 验收"
description: "从已合并的参考源比较基线发布 v0.2.28，并使用已安装 Runtime 验收公开制品。"
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-216-release-v0-2-28
status: recovered
authority: canonical
lastVerifiedBy: WI-216-release-v0-2-28
---

# WI-216——v0.2.28 不可变发布与 adopter 验收

本 Work Item 在首批参考源逐文件比较合并后发布 patch Release。比较基线
已经合并；本边界只建立下一个不可变 Runtime 制品并完成公开验收。

## 验收

1. workspace package 版本、lockfile、发布文档、架构文档和三语入口一致标识 v0.2.28。
2. Tag 只能从已审查的默认分支合并后后代创建，并通过 source、tag、manifest、checksum 和 provenance 门禁。
3. public adopter 与 N-1 验收只使用下载的不可变制品，禁止源码 checkout、workspace binary 和本地 `target` fallback。
4. 验收 receipt 绑定 repository/runtime identity、隔离 manifest、清理状态、evidence reuse 和可见的本地化 Outcome。
5. post-release version consistency、adopter acceptance 和 upgrade acceptance 通过，且不重写 Release truth。

## 不在范围内

下一批参考源逐文件比较是独立边界。本 Work Item 不增加 Runtime 功能、不复制参考实现代码，也不修改用户全局 Agent/MCP 配置。

## Evidence 边界

已发布 Release、下载 archive、manifest、checksum、attestation 和 adopter receipt
都是不可变外部 evidence。发布后失败记录 `releasePublished: true` 与
`adopterAcceptance: failed`，不会重写 Release truth。
