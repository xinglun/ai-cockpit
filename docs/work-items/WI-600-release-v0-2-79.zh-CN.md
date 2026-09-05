---
author: AI Cockpit maintainers
title: "WI-600——v0.2.79 发布与对象验收"
description: "发布 WI-599 流程顺序修正后的版本，并验证不可变公开产物。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-600-release-v0-2-79
lastVerifiedBy: WI-600-release-v0-2-79
---

[English](WI-600-release-v0-2-79.md) · [日本語](WI-600-release-v0-2-79.ja.md)

# WI-600——v0.2.79 发布与对象验收

## 目标

在 WI-599 文档门禁顺序修正后，从已审查且同步的默认分支发布 v0.2.79，
并确认公开产物可以在不使用源码或工作区 fallback 的情况下治理全新的对象工程。

## 边界

本 Work Item 只修改包版本元数据及当前发布/版本文档。Runtime 源码、对象工程、
全局 Agent/MCP 配置、历史 evidence 字节和参考源实现均不在范围内。发布后的
adopter 与 N-1 验收只能使用下载的不可变公开产物。

## 验收

1. Cargo 元数据和锁文件解析为 v0.2.79；失败历史标签保留且不复用。
2. 发布策略和 hosted checks 将 annotated tag、五个目标产物、校验和、SBOM/来源证明
   及 Runtime identity 绑定到同一审查提交。
3. Public adopter 与 N-1 harness 只使用 v0.2.79 产物，证明禁止写入根和成功/失败路径
   的临时运行目录清理。
4. 英文、简体中文、日文的当前发布、架构和版本页面一致；对象工程保持不变。
5. 发布后的失败保留已发布事实并记录失败 receipt，不重写标签或历史 evidence。

## 验证

执行 Contract 声明的 locked workspace、文档、Parity、发布策略、staged acceptance
及发布后公开产物检查。只有 hosted checks 通过、v0.2.79 发布、adopter/N-1 receipt
保留且精确分支/worktree 清理完成后，才完成 lifecycle。
