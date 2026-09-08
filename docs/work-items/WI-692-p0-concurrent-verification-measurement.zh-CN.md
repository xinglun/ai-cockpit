---
author: AI Cockpit maintainers
title: "WI-692——P0 并发验证测量"
description: "在考虑接入 PhysicalSingleFlightCoordinator 前，测量真实并发验证请求路径。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-692-p0-concurrent-verification-measurement
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-692-p0-concurrent-verification-measurement
---

[English](WI-692-p0-concurrent-verification-measurement.md) · [日本語](WI-692-p0-concurrent-verification-measurement.ja.md)

# WI-692——P0 并发验证测量

## 意图

测量同一仓库、同一命令的并发验证请求是否重复执行物理命令，并据此决定
是否值得在后续接入 `PhysicalSingleFlightCoordinator`。North Star 仍是
Calibrated Human-Agent Trust：证据、仓库隔离、授权、fail-closed 行为和恢复能力
优先于未经证实的吞吐收益。

## 范围与有效边界

本 WI 只增加可复现的外部测量器及其证据。测量器通过公开的
`ai-cockpit verify` CLI，在一个干净 fixture 上启动相互独立的进程，保留请求顺序、
原始结果、退出码、命令身份材料、receipt 指标、轮次墙钟时间和环境身份。它不直接
调用 coordinator，也不修改 `crates/**`、验证语义、门禁或生产 caller。

因此，观察到的每轮四次物理执行只是独立 CLI 进程基线，不能证明进程内 MCP/service
请求可以安全共享一次物理执行。只有在真实路径证明同身份重复执行且目标成本具有实质性，
同时保留逐请求授权、证据绑定、失败、取消和资源边界后，才可考虑后续接入。

## 瓶颈证据与假设

假设是：同身份并发请求可能重复执行物理命令，从而形成可测量的 single-flight 优化目标。
测量使用 release 构建的 Runtime `0.2.87`，文件摘要为
`sha256:7610b70b38dca6520ee8ee8cc15b838bb3c1d5d7f4b350234b02f055334fb3a6`，
fixture 为 `99f7d2323ffb59b1e3edd6c1c833b00f50fb9698` 的干净 detached 工作树。
fixture 有 9,955 个 tracked 文件、工作树无变化，仓库身份符合预期。

该 revision 的生产调用图只找到 coordinator 的定义和实现，没有生产 caller；引用仅出现在
`crates/cockpit-verification/tests/physical_execution.rs`。因此，独立 CLI 进程的重复执行是真实
观测，但不足以把当前未被生产使用的 in-process coordinator 接入生产。

## 测量 Contract 与原始结果

测量器按轮次和请求索引保留原始样本，仅对副本排序计算 nearest-rank 分位数。它记录样本数、
原始样本、预热/身份探针顺序、Runtime 与仓库身份、命令身份、进程数和每个不可用指标的原因。
它不宣称真正 cold cache：测量前已经调用 Runtime 获取身份，`coldCacheClaim` 为 `false`。
第一次运行暴露出测量器分类缺陷：即使治理 receipt 为 `passed: false`，也曾因退出码为 0
被计为成功。缺陷原始运行保留在
`.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.raw-initial-measurement-bug.json`；
之后修正为必须同时满足退出码为 0 且 `governancePassed == true`，并重新运行最终测量，没有静默替换记录。

最终轮次墙钟结果如下：

| 场景 | 轮次 × 并发 | 成功 / 失败请求 | 每轮物理执行 | p50 / p95 轮次墙钟 ms |
| --- | ---: | ---: | ---: | ---: |
| 同身份 | 20 × 4 | 80 / 0 | 每轮均为 4 | 720.006 / 1094.730 |
| 不同命令身份 | 20 × 4 | 80 / 0 | 每轮均为 4 | 503.963 / 726.116 |
| 失败传播 | 20 × 4 | 0 / 80 | 每轮均为 4 | 473.366 / 964.666 |
| 资源争用 | 20 × 4 | 80 / 0 | 每轮均为 4 | 674.100 / 993.346 |
| 取消 | 1 × 1 | 0 / 1 | 不可用：无治理 receipt | 106.182 / 不可靠 |

20 轮的 p50/p95 轮次汇总满足测量器可靠性下限；p99 低于声明的 100 个样本下限，不能作可靠
结论。取消只有一个样本，也不作可靠分位数声明。完整原始值和派生汇总保存在上述证据文件以及
`.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.measurement-summary.json`。

请求排队时间、CPU 时间、峰值内存、读取字节和哈希字节在该公开 CLI 路径或当前平台上不可用，
均显式记录为 unavailable，未填零。测量器记录了观察到的请求并发数和 receipt 进程数，但未测量
嵌套 Cargo 资源、常驻 MCP 延迟或进程内 service 排队。

## 决策

当前 **暂不接入 coordinator**。四个独立 CLI 进程每轮确实执行了四次物理命令，但不存在生产
caller，且当前路径不能建立安全共享所需的身份和证据边界。后续 successor 的前置条件是：

1. 在真实的进程内 MCP 或 service 路径上，使用相同仓库、WI、命令、Runtime 和 toolchain 身份测量。
2. 在共享物理结果前绑定每个等待者的授权和证据 receipt。
3. 在该路径上测量失败、超时、取消、等待者退出以及嵌套 Cargo 资源行为。

## 正确性、隔离与治理

聚焦测试验证样本顺序保持、不可用指标不编码为 0、命令身份参数保持不变。调用图证据区分定义、
测试引用和生产 caller。测量绑定仓库，使用外部可执行文件，fixture 保持干净，不改变 Rust 生产代码。
既有治理门禁、授权、证据语义和恢复行为未被弱化。

本 WI 在 Runtime verify、评审后的 PR 交付、archive、close 和文档晋级完成前保持 `in_progress`。
不宣称生产性能收益；暂缓决定和所有未知项均绑定在测量汇总证据中。
