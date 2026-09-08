---
author: AI Cockpit maintainers
title: "WI-696——P0 场景测量"
description: "使用现有 P0 基准测量真实场景矩阵，绑定瓶颈排序和有界后续预算。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-696-p0-scenario-measurement
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-696-p0-scenario-measurement
---

[English](WI-696-p0-scenario-measurement.md) · [日本語](WI-696-p0-scenario-measurement.ja.md)

# WI-696——P0 场景测量

## 目的与边界

本 Work Item 在不修改 Runtime 治理行为、授权、仓库隔离或恢复语义的前提下，使用现有 schema 2 基准测量真实场景。North Star 仍为 Calibrated Human-Agent Trust。开发 Runtime 从 `origin/main` revision `513e7e72523097893c658ec76c25706fd5b266b8` 构建，版本 `0.2.87`，文件摘要为 `sha256:faacc6a3368f56bc9274264c111500157fb79354260dba60898a3351b5c3e4dd`，复制到被测仓库之外；发布验收仍需遵守已发布二进制边界。

每个可复现场景保留首次测量、一次 OS 缓存预热、20 个独立 warm 进程、原始样本顺序、nearest-rank 分位数、Runtime/仓库身份、快照、环境、数据规模和不可用指标原因。首次测量不称为真正冷缓存；20 个 warm 样本可作 p95，p99 因少于 100 个样本明确不可用。

## 结果

最终开发基准原始证据位于 `.ai/evidence/external/WI-696-p0-scenario-measurement.*.budgeted2.dev-faacc6a3.json`，汇总见 `.ai/evidence/external/WI-696-p0-scenario-measurement-summary.json`。

| 场景 | 规模 | status warm p50/p95 ms | observe warm p50/p95 ms |
| --- | --- | ---: | ---: |
| small-clean | 15 文件、1,913 字节、干净 | 82.696 / 85.517 | 59.106 / 60.704 |
| single-file-change | 8 文件、1 个变更路径 | 93.616 / 103.280 | 71.838 / 72.722 |
| multi-file-change | 8 文件、3 个变更路径 | 95.430 / 99.833 | 72.077 / 73.432 |
| large-file-change | 6 文件、1 个 1 MiB 变更文件 | 99.895 / 101.456 | 75.675 / 77.739 |
| many-files-clean | 10,044 文件、75,325,549 字节 | 3,095.188 / 3,126.662 | 144.646 / 148.613 |
| many-historical-wi | 10,044 文件、604 个历史 WI | 3,103.234 / 3,588.843 | 145.194 / 148.943 |

当前最大瓶颈是大型仓库上的 `status`，尤其是大量历史 WI 场景；这是测量排序，不是未经后续 Contract 授权的优化结论。每个场景每轮观察到 91 个直接 CLI 进程，其中 4 个 Git 调用属于基准自身的元数据收集。Runtime 内部 Git 调用、读取/哈希字节、子进程数、峰值内存、阶段耗时和缓存失效原因均显式不可用，不填零。当前 Darwin 文件系统类型也未被接受为可信比较键，因此本机跨 Runtime comparator 会 fail closed。

并发 harness 的独立 CLI 同身份场景为 20 轮、并发 4、每轮 4 次物理执行，round p50/p95 为 441.652/454.629 ms；当前 `origin/main` 调用图显示 `PhysicalSingleFlightCoordinator` 没有生产调用方，因此暂不接通。resident MCP 没有真实传输可测，保持 `not_measured`。

## 后续预算

预算文件位于 `.ai/evidence/external/WI-696-p0-scenario-measurement-budgets/`。它们是基于重复开发测量和有界噪声余量的回归上限，不是性能收益声明。后续优化必须为目标路径预先登记 paired baseline/candidate 和改善阈值，保持所有非目标场景预算，并通过完整且环境可比的 `p0_regression_gate.sh` 证据。本 Work Item 不实施大文件流式哈希、轮询替换、并行读/哈希、single-flight、增量 Merkle、resident MCP 缓存、进程内 Git 或 PGO。

## 外部验证限制

仓库级 documentation acceptance、status consistency 和
`closed-work-item --check-all` 当前因独立的 WI-694 文档晋级仍为
`in_progress` 而 fail closed。精确输出保存在
`.ai/evidence/external/WI-696-p0-scenario-measurement.validation-limitation.json`。
本 Work Item 不修改 WI-694 文件，也不触碰其他 agent 的 worktree、分支、PR
或证据；WI-694 完成后必须重新运行这些检查。
