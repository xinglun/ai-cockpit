---
author: AI Cockpit maintainers
title: "WI-139F — Runtime v0.2.13 发布验收"
description: "将已合并的 recovery 与 adopter 验收控制发布为不可变公开 Runtime。"
audience:
  - maintainer
status: active
authority: repository-local
lastVerifiedBy: pending-release-evidence
workItemId: WI-139F-runtime-v0-2-13
---

# WI-139F — Runtime v0.2.13 发布验收

本 Work Item 将当前已合并的 Runtime 发布为 `v0.2.13`。完成条件包括不可变公开
Release、公开 fresh-adopter 验收、从 `v0.2.12` 开始的 N-1 升级验收，以及当前
repository 上的安装检查。验收必须只使用下载的公开 binary；源码构建不能替代
Release evidence。

发布 receipt 绑定 tag、archive digest、binary digest、platform、Runtime identity、
adopter repository identity、隔离 manifest、清理结果和 lifecycle evidence。发布后
失败只记录验收失败，不改变已发布的 Release truth。
