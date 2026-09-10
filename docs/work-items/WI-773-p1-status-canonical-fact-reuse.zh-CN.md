---
author: AI Cockpit maintainers
title: "WI-773——P1 status canonical fact 复用"
description: "测量 status 请求内 canonical finalization receipt 事实复用；未达到预注册收益阈值，拒绝候选实现。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-773-p1-status-canonical-fact-reuse
lastVerifiedBy: WI-773-p1-status-canonical-fact-reuse
---

# WI-773——P1 status canonical fact 复用

## Contract 与边界

本 Work Item 从远程 `origin/main` revision `5a24d4c0df865ece469822dbdc0dcc36eda07d85`
开始。假设是 `status` 的历史 finalization 投影在外层 inventory 已观察 canonical
`*.finalize.json` 后，又重新读取和解析同一 receipt。候选复用严格限制在一次不可变
status observation 内；跨请求、跨仓库、跨修改前后观察、治理规则变化及其他性能工作均不在范围内。

Contract 预先规定：warm 独立 CLI `status` 的 p50 和 p95 均须至少提升 5%，且治理输出
一致、非目标预算不超限，才接受候选。

## 测量与判断

配对原始证据为：

- `tests/performance/fixtures/WI-773-status-canonical-fact-reuse-baseline.json`
- `tests/performance/fixtures/WI-773-status-canonical-fact-reuse-candidate.json`

两轮使用同一个 detached clean fixture、repository identity、HEAD、445 个 canonical
finalization receipt 和 651 个历史 Work Item contract。harness 保留原始顺序、首次测量、
一次独立 CLI 预热、20 个 warm 样本、nearest-rank 分位数、场景事实及 unavailable 原因。
fixture 分类为 `many-historical-wi`。

| 路径 | warm p50 | warm p95 | 判断 |
| --- | ---: | ---: | --- |
| baseline `status` | 1062.470 ms | 1095.271 ms | 参考 |
| candidate `status` | 1089.892 ms | 1723.202 ms | 拒绝 |

本配对运行中，candidate 的 p50 慢约 2.58%，p95 慢约 57.3%，未达到 Contract 阈值。
因此撤销 candidate 生产改动；没有改变治理行为、证据语义、仓库隔离、授权边界或恢复能力。

本 macOS 主机无法可靠取得 filesystem type：`stat -f %T` 返回 `/`，因此 comparison key
和 filesystem comparability 明确记录为 unavailable。Runtime 未暴露的阶段耗时、实际读取字节、
哈希字节、Git 调用数、子进程数、峰值内存和缓存失效原因均保持 unavailable，不填零。
常驻 MCP 和并发验证不由本 CLI harness 测量。

## 正确性与验证

候选 probe 使用一次 observation 内的 fact 路径，并验证 canonical head read wrapper 不会被
第二次调用。probe 通过，但由于端到端配对结果未达阈值，候选被拒绝；保留树仍是原始实现，
没有候选 helper。

保留的 raw record 已运行 P0 gate。gate 对 unavailable filesystem comparison 以及第二轮
candidate `status` p95 budget 正确 fail closed。这是测量限制和候选拒绝证据，不是弱化 gate
的理由。下一 Work Item 应在重新取得瓶颈 profile 后，只针对主要实测阶段独立立项。

## Outcome

Outcome: 🟡 候选拒绝，测量证据保留。问题数：1（收益不足且尾部不稳定）。阻塞：本假设没有
可接受的生产优化。已解决：重复读取假设已隔离、测量并 fail-closed 拒绝。风险：当前 status
路径保持不变，原有成本仍在。验证：raw 配对 benchmark、P0 gate 输出、focused probe 及撤销
后的仓库测试。下一步：使用 P0 场景矩阵选择下一个独立瓶颈；没有新的 Contract 和证据时不恢复本候选。
