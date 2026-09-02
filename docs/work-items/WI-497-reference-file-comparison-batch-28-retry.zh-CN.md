---
author: AI Cockpit 维护者
title: "WI-497——参考源文件比对批次 28 重试"
description: "在 WI-496 不可变 Hosted CI parity 顺序失败后，重新交付同一十文件比对。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-497-reference-file-comparison-batch-28-retry
predecessorWorkItemId: WI-496-reference-file-comparison-batch-28
status: recovered
authority: human-authorized
lastVerifiedBy: WI-498-reference-file-comparison-batch-28-doc-recovery
successorWorkItemId: WI-498-reference-file-comparison-batch-28-doc-recovery
recoveryDecision: .ai/decisions/WI-497-reference-file-comparison-batch-28-retry.recovery.json
canonical: docs/work-items/WI-497-reference-file-comparison-batch-28-retry.md
---

# WI-497——参考源文件比对批次 28 重试

[English](WI-497-reference-file-comparison-batch-28-retry.md) · [日本語](WI-497-reference-file-comparison-batch-28-retry.ja.md)

## 边界

WI-496 作为不可变的 Hosted CI 失败历史保留。本 successor 从最新
`origin/main` 重新交付同一组固定的 10 个参考路径，只修正 parity 注册与
verification evidence 的顺序证明。不复制源 Python/Make 实现、源 receipt、
provider 决定，也不操作对象工程。

## 验收

- 保留原十项分类和源端边界。
- 在新 verification evidence 之前注册三语 WI-497 parity 行。
- 绑定 WI-496 recovery receipt，且 predecessor bytes 保持不变。
- 通过清单、文档、parity、治理和 Contract 声明的 Runtime 检查，完成评审合并与精确清理。
