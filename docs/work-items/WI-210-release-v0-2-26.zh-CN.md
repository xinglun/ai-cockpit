---
author: AI Cockpit maintainers
title: "WI-210——v0.2.26 不可变发布与 adopter 验收"
description: "从已合并的默认分支发布 v0.2.26，并使用公开 binary 完成发布治理 transition。"
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-210-release-v0-2-26
status: current
authority: canonical
lastVerifiedBy: WI-210-release-v0-2-26
---

# WI-210——v0.2.26 不可变发布与 adopter 验收

本 Work Item 在已保留且失败的 v0.2.25 发布历史之后，建立下一个不可变公开
Release。它绑定版本一致性、已合并 PR 与 release-tag 证明、公开 binary 的
adopter/upgrade 验收，以及 WI-209 的安装版 Runtime finalization 与结构化关闭。

adopter 边界只接受下载的公开 Release 资产。源码 checkout、`cargo build`、
`cargo run`、workspace binary 和本地 `target` 产物都不能作为 fallback。
`v0.2.25` 保持不可变失败历史，永不移动或复用。

## 验收

1. 在 release verification 前，v0.2.26 版本、分发文档和三语 parity 一致。
2. 只有带有效合并前 finalize 回执且通过 release-tag ancestor proof 的合并提交，
   才能创建不可变 tag。
3. 公开 adopter 与 N-1 upgrade 只使用下载资产，并产生 repository/runtime identity
   与隔离证据。
4. 成功、失败和中断路径都清理临时验收根目录，清理结果进入带校验和的回执。
5. 安装的公开 Runtime 完成 WI-209 finalize、finalize-verify 和结构化 human close，
   并产生可见且本地化的 Outcome handoff。

## 不在范围内

与参考源逐文件对比和功能补齐属于下一批。本 WI 不增加无关 Runtime 功能，也不修改
用户全局 Agent/MCP 配置。

## 证据边界

公开 Release 及其下载的 archive/manifest 是不可变外部证据。发布后失败必须记录
`releasePublished: true` 与 `adopterAcceptance: failed`，不能回写或伪造 Release truth。
