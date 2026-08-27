---
author: AI Cockpit maintainers
title: "WI-317——post-close reconciliation quality gate fix"
workItemId: WI-317-post-close-reconciliation-quality-gate-fix
description: "在不改写不可变历史的前提下重新交付 W316 的有界质量门修正。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-317-post-close-reconciliation-quality-gate-fix
---

# WI-317——post-close reconciliation quality gate fix

## 意图与边界

W316 是不可变的归档交付，其 hosted quality run 暴露了三个有界缺陷：parity 行没有跟随
recovery 决定、中文资源收尾页面缺少明确的 close 顺序规则，以及 promotion 回归仍断言旧的
错误消息。本 successor 保留 W316 字节，只从最新 `origin/main` 基线重新交付这些修正。

## 范围与验收

- W316 Contract、evidence、Outcome、Events、archive、recovery 以及 PR #280 历史保持逐字节不变。
- 三份 parity ledger 如实将 W312 标记为已实现、W314/W315 标记为已恢复，并包含准确的 recovery evidence 路径。
- 三语资源收尾工作流文档都明确：只有 `finalize-verify` 成功后才能执行 `close`。
- promotion 回归与当前 helper 错误消息一致；所有聚焦、完整和 hosted quality gate 在不放宽门禁的情况下通过。
- successor 从最新远端 default 基线开始，只在 hosted checks 经评审通过后合并，然后完成 finalize、close 和精确清理。

## 验证

使用已安装 Runtime 运行文档/promotion/resource-finalization 聚焦回归、文档验收、单进程
locked workspace 测试，以及该 reviewed branch 的 hosted CI 检查。

## 相关历史

- W316：被 hosted quality 检查拒绝的不可变交付；其字节作为历史证据保留。
- W317：只修正该次运行发现的问题的有界 successor。

[English](WI-317-post-close-reconciliation-quality-gate-fix.md) ·
[日本語](WI-317-post-close-reconciliation-quality-gate-fix.ja.md)
