---
author: AI Cockpit maintainers
workItemId: WI-889-object-performance-current
title: 当前对象仓库性能证据
description: 在选定对象仓库上建立当前版本 Runtime 与开发周期的配对测量。
audience: [adopter, contributor, maintainer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-889-object-performance-current
terminalArchive: .ai/work-items/archive/WI-889-object-performance-current.contract.json
terminalVerification: .ai/evidence/WI-889-object-performance-current.verification.json
terminalDecision: .ai/decisions/WI-889-object-performance-current.close.json
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

## 当前配对结果

基线（`0.2.93`）与候选（`0.2.95`）在同一台
`aarch64-apple-darwin`、Rust/Cargo `1.98.1` 和相同的隔离对象仓库视图上
完成七个场景的配对采集。每个比较操作都有 100 个有效 warm 样本。比较器
输出 p50、p95 和 p99；5 ms 噪声判定沿用稳定的 p50/p95 预算，p99 作为尾部
诊断保留。38 项比较全部为 `within_noise`；比较有效，但没有证明候选版本
已经改善。

测量视图为 goods-garden（当前仓库和文件变更场景）、sentinel
（many-files-clean）以及 ai-investigation-orchestrator
（many-historical-wi）。曾尝试直接测量 small-clean，但所有提供的对象仓库
都超过 harness 要求的 `<=100` 个 tracked 文件，因此如实标记为不可取得，
没有用合成仓库替代。包含进程/资源计数和失效原因的完整采集器输出保存在
[压缩原始采集包](../../.ai/evidence/WI-889-object-performance-current/raw/runtime-captures.tar.gz)
及其 [SHA-256 校验](../../.ai/evidence/WI-889-object-performance-current/raw/SHA256SUMS)。

诊断开销单独配对测量。在当前仓库，diagnostics-on 相对 off 对
`verification-plan` 增加 p50 5.134 ms、p95 5.255 ms；对 `status` 增加 p50
0 ms、p95 1.183 ms；对 `work-item-outcome` 增加 p50 0.674 ms、p95 1.689 ms。

Runtime 延迟与开发周期成本分开报告。Contract→可创建 PR 已测得
4,744,000 ms（Contract 创建 `2026-09-17T19:23:54Z` 至 PR #865 创建
`2026-09-17T20:42:58Z`）。该阶段没有持久化 agent 操作数，因此该维度保持
不可用；记录的前置拒绝数为 0。验证→finish 和合并后清理仍须等生命周期时间戳
出现后再报告，并在开发周期报告中明确标记不可用。
