---
author: AI Cockpit 维护者
title: "WI-498——批次 28 文档恢复"
description: "修复 Hosted CI 发现的前置项状态过期文档投影。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-498-reference-file-comparison-batch-28-doc-recovery
predecessorWorkItemId: WI-497-reference-file-comparison-batch-28-retry
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-498-reference-file-comparison-batch-28-doc-recovery
canonical: docs/work-items/WI-498-reference-file-comparison-batch-28-doc-recovery.md
---

# WI-498——批次 28 文档恢复

[English](WI-498-reference-file-comparison-batch-28-doc-recovery.md) · [日本語](WI-498-reference-file-comparison-batch-28-doc-recovery.ja.md)

## 边界

WI-497 保留为不可变的 Hosted CI 失败历史。本 successor 只修复权威 recovery
与 parity 记录要求的三语文档投影，不改写前置项 archive/evidence bytes，不改变
Runtime policy，不复制源实现，也不操作对象工程。

## 验收

- WI-496 与 WI-497 页面使用 `recovered` 状态并链接 Runtime recovery receipt。
- 批次 28 的十项分类与源专属边界保持不变。
- 在评审合并与准确清理前，文档、parity、清单、治理和声明的 Runtime 检查通过。
