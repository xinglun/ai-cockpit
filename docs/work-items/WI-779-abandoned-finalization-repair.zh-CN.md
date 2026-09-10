---
author: AI Cockpit maintainers
title: "WI-779——abandoned finalization 修复"
description: "为明确关闭但未合并的失败交付增加诚实的终态。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-779-abandoned-finalization-repair
lastVerifiedBy: WI-779-abandoned-finalization-repair
---

[English](WI-779-abandoned-finalization-repair.md) · [日本語](WI-779-abandoned-finalization-repair.ja.md)

# WI-779——abandoned finalization 修复

## 意图与边界

WI-779 修复失败交付的 Runtime resource-finalization 边界：provider Pull Request
已明确关闭但没有合并。新的 `abandoned` 终态是诚实的失败记录：必须绑定准确的
`unmerged_pull_request` failure code、不得有 merge commit，并且 branch/worktree
资源必须已经移除。它绝不会被投影为已合并成功。

PR #760 保留为已关闭的历史失败分析输入。本 Work Item 从当前 `main` 开始；不会复活
该 PR 或 branch，也不会重写 WI-774、WI-775、WI-776 或 WI-778 的不可变记录。

## 范围

- 扩展 typed Runtime protocol 以及 repository close/finalization 检查。
- 增加 protocol、repository、CLI 和文档 promotion 回归测试，包括对无效 abandoned
  receipt 的 fail-closed 检查。
- 用英文、简体中文和日文记录该边界，并在 reference-parity 表中登记本 Work Item。

性能实现与测量、发布、无关的产品行为、provider API 行为和用户收益声明均不在范围内。

## 验证

声明的 workspace 验证为 `cargo test --locked --workspace`。聚焦检查覆盖 protocol、
repository、CLI 和 closed-documentation promotion 套件。Hosted quality route 必须通过
文档 acceptance、治理完整性、状态一致性以及现有 repository gates。

