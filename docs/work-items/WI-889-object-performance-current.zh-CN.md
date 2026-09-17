---
author: AI Cockpit maintainers
workItemId: WI-889-object-performance-current
title: 当前对象仓库性能证据
description: 在选定对象仓库上建立当前版本 Runtime 与开发周期的配对测量。
audience: [adopter, contributor, maintainer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-889-object-performance-current
---

# WI-889——当前对象仓库性能证据

本 Work Item 在同一机器、工具链、构建模式和场景定义上，将当前 Runtime
与公开基线进行配对测量。选定对象仓库只通过临时隔离视图观察；其 main
分支和既有工作区状态不在范围内。

## 验收边界

- 发布级分位数比较至少需要 100 个有效 warm 样本；99 个只能用于诊断，
  且必须被门禁拒绝。
- Runtime 延迟与开发周期成本分开报告，包含原始样本、p50/p95/p99、环境
  身份、操作计数、复用计数，以及明确的失效或不可取得原因。
- goods-garden、sentinel 和 ai-investigation-orchestrator 只能通过临时
  隔离视图测量，收集后必须清理。
- 归档前保持三语 Work Item 投影和 reference-parity 行同步。本 Work Item
  不包含发布。

## 验证计划

使用现有性能 harness，并设置 `CARGO_INCREMENTAL=0` 和共享验证 target
目录。在昂贵 workspace 验证前先完成格式、性能和文档投影定向检查。保留
原始证据；指标未知时明确写出未知，不以零代替。
