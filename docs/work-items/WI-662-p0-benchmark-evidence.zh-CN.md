---
author: AI Cockpit maintainers
title: WI-662——P0 可信性能基准证据
description: 在任何 Runtime 优化前建立保持采样顺序且绑定证据的性能基准。
audience: [maintainer, reviewer, adopter]
workItemId: WI-662-p0-benchmark-evidence
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-662-p0-benchmark-evidence
---

# WI-662——P0 可信性能基准证据

[English](WI-662-p0-benchmark-evidence.md) · [日本語](WI-662-p0-benchmark-evidence.ja.md)

## 意图

建立可信、可比较且可审计的 AI Cockpit 性能测量，为降低从请求进入到可信治理决定的延迟、CPU、I/O 和内存成本提供瓶颈排序与预算依据，同时保持 Calibrated Human-Agent Trust、证据有效性、仓库隔离、授权边界和恢复能力。

## 边界

本 Work Item 只修改性能基准、比较器、测试夹具、性能说明、Work Item 记录、参考 parity 和计划文件。它修正 cold/warm 分组并建立 schema 2 measurement evidence；不改变生产 Runtime 治理运行行为，不实施 P1-P3 优化，不引入常驻缓存、进程内 Git、PGO、增量摘要或新的协调器。

## 验收

- 固定序列 `120、20、22、21` 先按原始顺序分组，首次测量值为 `120`。
- 明确记录原始样本、预热次数、样本数、分位数算法、首次 CLI、OS 缓存预热后的独立 CLI 和常驻 MCP 边界；样本不足时不声明可靠 p95/p99。
- baseline/candidate 可以使用不同 Runtime 身份，但各自证据、仓库快照和环境必须完整且可比较；保留既有 schema 1 负向 gate 覆盖。
- 阶段耗时、场景矩阵、实际读取/哈希字节、Git 调用、进程和峰值内存无法可靠取得时显式标记不可用，不填零。

## 证据边界

当前文档只登记 Work Item 和测量契约，不宣称性能收益。sanity output 仅用于验证证据结构；发布验收仍需同环境、同场景、足够样本和既定发布二进制。

## 验证

以 Contract 和计划文件中的 focused tests、documentation acceptance、parity check、治理 gate 及 `git diff --check` 为准。完成前不得将此 Work Item 标记为 verified 或 closed。
