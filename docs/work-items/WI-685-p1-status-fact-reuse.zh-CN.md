---
author: AI Cockpit maintainers
title: "WI-685——P1 status 事实复用"
description: "在包含大量历史 Work Item 的仓库中，于一次 status 观察内复用不可变 finalization transition 事实。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-685-p1-status-fact-reuse
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-685-p1-status-fact-reuse
---

[English](WI-685-p1-status-fact-reuse.md) · [日本語](WI-685-p1-status-fact-reuse.ja.md)

# WI-685——P1 status 事实复用

## 意图

降低大量历史 Work Item 仓库中 `status` 重复扫描和解析 finalization
transition 的成本，同时保持 Calibrated Human-Agent Trust、仓库与 WI 隔离、
证据绑定、授权边界和恢复能力。

## 边界

候选实现只在请求内生效。`status_with_runtime` 为当前观察建立一个不可变、
绑定仓库的 `FinalizationTransitionIndex`，交给历史 finalization 投影。索引
保留每个 transition 的路径、内容摘要、解析值及 fail-closed 的读取/解析/摘要
错误。不引入跨请求缓存，不改变 snapshot 边界，不修改证据，也不触及 Runtime、
MCP、doctor、调度、IncrementalMerkle、大文件、并行读取或 P3 路径。

共享解析器会先读取 canonical receipt，再消费索引候选，保留原有 canonical
错误优先级；损坏、缺失、分叉、过期、摘要不匹配、符号链接和路径异常仍然
fail-closed。

## 假设与瓶颈证据

目标仓库包含 395 个 canonical finalization receipt 和 156 个 transition 文件。
改动前，历史投影会对每个旧 Runtime receipt 再扫描一次 decisions 目录，每次
status 观察共 396 次目录枚举。候选路径只枚举两次：一次建立 canonical inventory，
一次建立 transition index；索引仍严格限定在当前仓库和当前观察内。

计数证据见
`.ai/evidence/external/WI-685-p1-status-fact-reuse.observation-counts.json`。

## 测量 Contract 与结果

Contract 已在接受前写入并重新验证：至少 20 个独立 CLI 预热样本；`status`
p50 至少改善 10%、p95 至少改善 5%；非目标路径 p95 回归同时低于 25% 和 20ms。
基准记录首次测量、一次 OS 缓存预热、预热样本数、分位数可靠性、环境、原始样本
和不可用指标，不宣称真正冷缓存或常驻 MCP 行为。

| 配对运行 | status baseline p50/p95 ms | status candidate p50/p95 ms | 变化 |
| --- | ---: | ---: | ---: |
| baseline → candidate | 2058.544 / 2165.179 | 1565.100 / 1596.771 | -23.98% / -26.25% |
| candidate → baseline | 2051.134 / 2104.867 | 1559.946 / 1653.997 | -23.94% / -21.43% |

这两组同窗口运行中，inspect、doctor、observe 均未超过非目标预算。另一次
round40 的 pre-box 候选测量出现明显变慢，且无关路径也同时变慢；由于平台不能取得主机负载，
该时间上不配对的结果作为不利证据保留，不用于接受声明，也没有删除或替换。

原始证据和汇总位于 `.ai/evidence/external/WI-685-p1-status-fact-reuse.*.json`，
重点是 `measurement-summary.json`。

## 正确性与隔离

- transition 聚焦测试 29 项通过，status projection 测试 11 项通过。
- TDD 回归测试先证明 canonical 错误优先级会被改变，随后通过“先观察 canonical
  receipt 再处理候选”的重构恢复原语义。
- clean、单文件修改、损坏 transition 三条 CLI status 路径的退出码均为 0；
  baseline 与 candidate 在只移除允许不同的 `.compatibility.runtimeDigest` 后，
  输出 SHA-256 完全一致。
- 单元测试在两个独立观察中各解析 fixture 一次；观察结束后索引即释放，不在仓库、
  WI 或请求之间共享。

详见 `output-parity.json`、原始 status JSON 和 `preflight-negative-tests.json`。

## 限制与治理状态

本 WI 未测量常驻 MCP 和其余场景矩阵。平台不能可靠提供 Runtime 阶段耗时、实际
读取字节、哈希字节、内部 Git 调用、Runtime 子进程、缓存失效事件或峰值内存，均
明确标记 unavailable，未填零。基准也不能测量主机 CPU 争用，因此 round40 不利
结果单独分类保留。

当前 Work Item 为 `in_progress`，等待 Runtime verify、托管 PR review、merge、
archive、close 和文档晋级；尚未声称 PR 或最终治理决定。
