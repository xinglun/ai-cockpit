---
author: AI Cockpit maintainers
title: "WI-213——v0.2.27 不可变发布与 adopter 验收"
description: "从已合并的 main 发布 v0.2.27，并使用安装的 Runtime 验证公开 artifact。"
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-213-release-v0-2-27
status: current
authority: canonical
lastVerifiedBy: WI-213-release-v0-2-27
---

# WI-213——v0.2.27 不可变发布与 adopter 验收

本 Work Item 发布 v0.2.26 source-quality 失败之后的第一个公开版本。
v0.2.26 tag 作为不可变失败历史保留，绝不重写或复用。v0.2.27 从已合并的
PR #160 构建，只能通过公开下载 artifact 验收。

## 验收

1. Cargo 版本、lockfile、发布文档、三语 parity 和 release workflow policy
   一致标识 v0.2.27。
2. tag 只能创建在 PR #160 的合并后提交上，并通过 source、tag、manifest、
   checksum 和 provenance gates。
3. adopter 和 N-1 验收只使用不可变的公开下载 artifact；禁止源码 checkout、
   workspace binary 和本地 `target` fallback。
4. 验收 receipt 绑定 repository/runtime identity、隔离 manifest、清理状态、
   evidence reuse 和可见的本地化 Outcome。
5. 发布后使用安装的 v0.2.27 Runtime 完成 WI-212 的 post-merge finalization
   transition、`finalize-verify` 和结构化 close。

## 范围外

逐文件参考源 parity 是下一批任务。本 Work Item 不增加无关 Runtime 功能，
也不修改用户全局 Agent/MCP 配置。

## Evidence 边界

公开 Release、下载 archive、manifest、checksum、attestation 和 adopter
receipts 都是不可变 external evidence。发布后失败记录
`releasePublished: true` 与 `adopterAcceptance: failed`，不回写 Release truth。
