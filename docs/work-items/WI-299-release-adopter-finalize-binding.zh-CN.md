---
author: AI Cockpit maintainers
title: "WI-299——发布 adopter finalization 基线绑定"
workItemId: WI-299-release-adopter-finalize-binding
description: "让发布 adopter 的 finalization receipt 始终绑定归档 Work Item Contract 基线。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-299-release-adopter-finalize-binding
authority: canonical
---

# WI-299——发布 adopter finalization 基线绑定

## 意图

v0.2.32 staged adopter 验收发现了一个真实的 fail-closed 不匹配：脚本把
变更后的 HEAD 写入 `pullRequest.baseRevision`，但 Runtime 要求该字段等于
归档 Contract 的基线版本。

## 范围

两个发布 adopter 脚本都会在变更前读取并校验各自 Work Item Contract 的
`baseRevision`。finalization receipt 继续在 `headRevision` 中记录变更后的
HEAD，同时将 `pullRequest.baseRevision` 绑定到保存的 Contract 基线。静态
回归检查覆盖 staged 和 N-1 升级路径。

## 边界

本 WI 只修正验收脚本和回归测试，不改变 Runtime 生命周期语义，不改写
v0.2.32 历史字节，也不增加新的 adopter 技术栈。已有的清理、隔离、不可变
制品和结构化决定检查继续有效。

## 验证

- adopter 与 upgrade 静态测试通过。
- candidate 验收必须到达 `finalize-verify` 和结构化 close。
- receipt 必须区分 Contract 的 `baseRevision` 与变更后的 `headRevision`。
