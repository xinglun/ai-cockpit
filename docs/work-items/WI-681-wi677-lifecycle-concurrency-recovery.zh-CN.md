---
author: AI Cockpit maintainers
title: WI-681——WI-677 生命周期并发恢复重新交付
description: 从最新默认分支重新交付 P2-C 生命周期并发与恢复边界。
workItemId: WI-681-wi677-lifecycle-concurrency-recovery
audience:
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-681-wi677-lifecycle-concurrency-recovery
---

# WI-681——WI-677 生命周期并发恢复重新交付

WI-681 是 WI-677 的明确 successor。由于远程默认分支推进，已归档 PR 发生冲突；本
Work Item 保留前置 Work Item 的 archive、evidence、Outcome、finalization 和 recovery
决定，并从最新 `origin/main` 重新交付相同的 P2-C 边界。

## 边界

对每个 Work Item 的 `finish`、`archive`、`close`、recovery 决定记录和活动产物
reconcile 使用稳定的操作系统 advisory lock 串行化。临时文件名在进程内保持唯一，
失败投影不得覆盖已经提交的终态。保持公共 JSON、生命周期语义、archive 布局和
Runtime 兼容性不变。

## 验证

并发测试覆盖同进程和跨进程 finish 竞争、archive/close 竞争，以及缺失或损坏投影的
fail-closed 行为。合并前仍需通过完整 locked workspace、格式化、Clippy、文档验收、
治理完整性和 hosted checks。
