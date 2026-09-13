---
author: AI Cockpit maintainers
title: "WI-833 — 发布脚本来源"
description: "分离发布编排代码与不可变源码身份。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-833-release-script-provenance
lastVerifiedBy: WI-833-release-script-provenance
---

[English](WI-833-release-script-provenance.md) · [日本語](WI-833-release-script-provenance.ja.md)

# WI-833：发布脚本来源

## 意图

发布验收必须执行 workflow 提交中的、经过审查的编排代码。不可变的目标
Release tag 仍是产物与源码身份的唯一来源，不能同时决定验收脚本版本。

## 决定

四条 staged/public adopter job 在 workspace 根目录 checkout `github.sha`，并将
请求的 `to_tag` checkout 到 `release-source`。验收脚本从根目录执行，并通过
`--source-repo` 传入 `release-source`。这样 WI-832 的命令复用修复可以用于验收，
同时不改变不可变 tag 或其产物。

## 验证

adopter 验收回归检查两种 checkout 身份；如果 job 使用目标 tag 作为脚本 checkout
或缺少独立源码 checkout，测试会失败。正式 Runtime verification 保存定向回归结果
及退出码。

## 范围外

Runtime 复用语义、产品构建、不可变 tag 与 Release、历史 Work Item，以及无关的
分支或 worktree 清理。
