---
author: AI Cockpit maintainers
title: "WI-715 — P1 大历史 status 候选决策"
description: "测量大历史 status 路径的请求内去重候选；验收契约未满足时保留有证据的拒绝决定。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-715-p1-large-history-status
lastVerifiedBy: WI-715-p1-large-history-status
---

[English](WI-715-p1-large-history-status.md) · [日本語](WI-715-p1-large-history-status.ja.md)

# WI-715 — P1 大历史 status 候选决策

## 目的

验证在一次不可变 `status` 观察中复用已校验的 close receipt，是否能减少
`many-historical-wi` 路径的重复读取和解析，同时保持 Calibrated Human-Agent
Trust、证据有效性、仓库隔离、授权和恢复行为。

## 边界与验收契约

本 Work Item 仅包含请求内 status 观察、聚焦测试、外部基准证据和三语记录。
不增加持久缓存、跨仓库或跨 WI 复用、协调器、IncrementalMerkle 复用、大文件
流式读取、等待替换或 P3 架构改动。候选实现事先登记的要求是 warm status
p50 和 p95 均至少提升 5%，并通过既有非目标预算和行为检查。

## 测量与结果

配对开发二进制运行使用 20 个 warm 样本和一次 OS 缓存预热。首次测量发生在
身份探针之后，不称为真正冷缓存。p99、常驻 MCP、阶段耗时、Runtime 内部 Git/I/O
计数和峰值内存，在工具或平台无法可靠证明时记录为不可用，不填零。当前主机无法
提供可信文件系统比较键，因此回归门禁 fail closed。

| 顺序 | baseline status warm p50/p95 | candidate status warm p50/p95 | candidate 变化 | 决定 |
| --- | ---: | ---: | ---: | --- |
| baseline → candidate | 3483.095 / 4681.167 ms | 3334.552 / 3478.871 ms | -4.26% / -25.68% | 未达阈值；门禁 fail closed |
| candidate → baseline | 3315.139 / 4260.963 ms | 3435.055 / 4453.557 ms | +3.62% / +4.52% | 未达阈值；门禁 fail closed |

另一次未绑定预算的运行对候选不利（p50 +1.35%、p95 +9.28%），仅作为支持证据保留。
候选代码和测试已移除；没有生产性能改动合入。

### 当前基线重新验证

远程默认分支推进后，本 Work Item 已同步到
`d1141480fb7a045979098480c3770d06002e2a87`，并在包含 10,247 个跟踪文件和 616 个已归档 WI
的干净 fixture 上重新执行实验。当前基线结果如下：

| 顺序 | baseline status warm p50/p95 | candidate status warm p50/p95 | candidate 变化 | 决定 |
| --- | ---: | ---: | ---: | --- |
| baseline → candidate | 3166.894 / 4331.523 ms | 3129.794 / 3310.081 ms | -1.171% / -23.582% | p50 未达阈值；门禁 fail closed |
| candidate → baseline | 3150.083 / 3454.306 ms | 3140.574 / 3474.931 ms | -0.302% / +0.597% | p50 未达阈值；门禁 fail closed |

当前基线 parity 再次覆盖 12 次比较，退出码和规范化输出差异均为 0。两个 P0 门禁仅报告文件系统比较键不可用；
不宣称性能、CPU、I/O、内存或常驻 MCP 收益。当前基线原始记录与旧证据并存于
`.ai/evidence/external/WI-715-p1-large-history-status.*`。

## 正确性与证据

在 clean、changed 和 malformed-close fixture 上，对 `inspect`、`status`、`doctor`、
`observe` 的 baseline/candidate 输出一致：12 次比较，退出码差异为 0，去除仅 runtime
digest 字段后规范化输出差异为 0。原始和派生记录位于
`.ai/evidence/external/WI-715-p1-large-history-status.*`；门禁结果和实验摘要是决定记录。

## 决定与后续

决定：`declined`。本 WI 不宣称延迟、CPU、I/O、内存或常驻 MCP 收益。在可信环境比较键和
更强的阶段级测量确认瓶颈并证明复杂度值得之前，不应把该微优化作为接受的改动重试。后续
优化 WI 必须从最新审查过的默认分支开始，并保留本次拒绝作为不可变证据。
