---
author: AI Cockpit maintainers
title: "WI-431——v0.2.47 发布恢复"
description: 在不移动不可变 v0.2.46 标签的前提下恢复发布。
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-431-release-v0-2-47-recovery
lastVerifiedBy: WI-431-release-v0-2-47-recovery
---

# WI-431——v0.2.47 发布恢复

## 意图与边界

第一次 v0.2.46 发布尝试被 release source gate 拒绝，原因是已关闭 Work
Item 的文档尚未 promotion。该标签保留为不可变的失败交付历史，永不移动或
重新标记。本 Work Item 先完成终态文档 promotion，再创建新的 patch release，
并端到端证明公开制品路径。

这是发布/文档恢复，不修改 Runtime 源码、CI workflow policy 或 Repository
Protocol。

## 验收

- 发布打 tag 前，已关闭 Work Item 的三语文档均已 promotion。
- Cargo metadata 与 lockfile 从 v0.2.46 准确推进一个 patch 到 v0.2.47，
  不复用失败的 v0.2.46 标签。
- v0.2.47 公开制品通过 manifest、校验和、SBOM、provenance、平台 smoke、
  adopter 与 N-1 验收，且只使用下载制品。
- v0.2.46 失败保持为有记录的、未公开、不可变标签。
- 经审查的 merge、finalization、close、同步和精确清理后，默认分支保持
  `ready_on_base`。

## 验证边界

发布 route 执行仓库 strict gate manifest、workspace tests、三语文档检查和
公开发布 harness。发布后 receipt 必须绑定下载 binary、release manifest、
校验和、Runtime identity、adopter repository identity、隔离 manifest 与清理结果。
