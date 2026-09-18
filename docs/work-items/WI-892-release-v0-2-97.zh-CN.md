---
author: AI Cockpit maintainers
workItemId: WI-892-release-v0-2-97
title: v0.2.96 不可变候选拒绝后的 v0.2.97 发布
description: 修正工作区版本身份并发布修复后的候选版本。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: user:release-after-all-work-items
lastVerifiedBy: WI-892-release-v0-2-97
---

# WI-892 — v0.2.96 不可变候选拒绝后的 v0.2.97 发布

v0.2.96 标签因其源码工作区仍标识为 0.2.95 而被保留为不可变失败候选。本
Work Item 修正工作区身份并发布下一个未占用版本，不移动或删除失败标签。

## 验收边界

- 将工作区和生成制品统一为 0.2.97。
- 保持 v0.2.96 不可变，并记录 aggregate 失败证据。
- 只有评审检查、manifest/checksum/SBOM 绑定，以及隔离根目录中的下载制品
  fresh-install 和 v0.2.93 升级验收全部通过后才发布。
- 保持对象仓库以及 Outcome/HCI/性能范围不变。

## 验证计划

先执行格式和版本检查，再执行声明的工作区验证、发布工作流和下载制品验收。
保留所有失败运行记录，不移动标签，也不重跑无关 Work Item。
