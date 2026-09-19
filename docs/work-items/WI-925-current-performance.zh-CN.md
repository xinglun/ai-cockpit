---
author: AI Cockpit maintainers
workItemId: WI-925-current-performance
title: 当前版本对象仓库性能测量
description: 在不修改对象仓库的前提下重现实 Runtime 与开发周期测量。
audience: [adopter, contributor, maintainer]
status: implemented
authority: human:xinglun
lastVerifiedBy: WI-925-current-performance
terminalArchive: .ai/work-items/archive/WI-925-current-performance.contract.json
terminalVerification: .ai/evidence/WI-925-current-performance.verification.json
terminalDecision: .ai/decisions/WI-925-current-performance.close.json
---

[English](WI-925-current-performance.md) · [日本語](WI-925-current-performance.ja.md)

# WI-925——当前版本对象仓库性能测量

本 Work Item 产出当前版本证据，不预设或声称优化收益。使用已安装 Runtime
和同一 Rust/Cargo 工具链，只读观察 ORG-X、sentinel、goods-garden、
ai-investigation-orchestrator 的隔离视图。不得改变它们的 main/default 分支
或既有工作区字节。

## 验收边界

- 每个发布级操作至少有 100 个有效 warm 样本；少于 100 个只能诊断，必须被
  发布级比较器拒绝。
- 保留原始样本、p50/p95/p99、Runtime/二进制/仓库/工具链/环境身份、执行与
  复用计数，以及明确的不可取得或失效原因。
- Runtime 延迟与 Contract→可评审 PR、验证→finish、合并后清理成本分开报告。
  缺少生命周期数据时记为 `unknown`，不能填零。
- 只有配对比较越过冻结的噪声预算时才报告提速；否则保持 `within_noise` 或
  `unknown`。

## 验证边界

使用 `CARGO_INCREMENTAL=0` 和共享验证 target 目录。采集前先完成格式、范围和
投影等廉价检查。在采集前后核对每个对象仓库的分支、HEAD 和工作区，并在
Work Item 证据中保留原始采集结果。本 Work Item 不修改对象仓库，不优化
Runtime 代码，也不发布版本。

开发周期报告必须区分实际测得的区间和 Runtime 无法观察的指标，包括 agent 操作
次数或 provider 清理时间戳。最终人类 Outcome 会明确保留尚未证明的收益未知项。
