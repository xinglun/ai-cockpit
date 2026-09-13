---
author: AI Cockpit maintainers
title: "WI-831 — 发布身份边界"
description: "分离不可变产物身份与 workflow dispatch 身份。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-831-release-identity-boundary
lastVerifiedBy: WI-831-release-identity-boundary
---

[English](WI-831-release-identity-boundary.md) · [日本語](WI-831-release-identity-boundary.ja.md)

# WI-831：发布身份边界

## 意图

WI-830 的 dispatch 语法修复合并后，发布流程仍把不可变产物源码提交与当前
workflow dispatch 提交当作必须相同。此 Work Item 保持失败关闭边界，同时允许
经过审查的编排提交发布已经创建的不可变 tag。

## 范围与决定

dispatch 预检校验本地 annotated tag 与远端 tag 展开的提交一致。发布校验
manifest 与该 tag 提交一致。dispatch workflow revision 仍是独立的执行身份。
两种身份都不能改写对方，并且不重新创建 v0.2.92 tag。

## 验证

- 策略测试继续保留仅 dispatch 发布和括号平衡的 close 要求。
- 发布前边界会在编译前拒绝可变或已移动的 tag。
- 在 dispatch 已有不可变 v0.2.92 之前，从当前 main 审查 hosted workflow。

## 范围外

历史 Work Item 与 receipt、完整 workspace 重跑、无关分支或 worktree 清理，
以及 Contract 验证前改写或发布 tag。
