---
author: AI Cockpit maintainers
title: "性能基线"
description: "可复现的本地性能证据及其发布限制。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - performance_baseline
---

# 性能基线（本地证据）

本基线使用以下命令采集：

```text
command: cargo test -p cockpit-cli --test performance -- --nocapture
source base: 9177b119d3232bbc48dacca71c0beff31089e82b
host: aarch64-apple-darwin（Darwin arm64）
toolchain: rustc/cargo 1.94.1
profile: dev，增量测试夹具
date: 2026-08-21
```

采集时源码树是带未提交变更的本地候选。因此这些数字只是本机基线，不是发布证据；
公开发布前必须从不可变的发布候选重新采集。

| 面 | 夹具 | 结果 |
| --- | --- | --- |
| `status` 温热启动 | 12 个样本 | 中位 23 ms |
| repository observation（增量缓存命中） | 200 个生成文件，读取 405 个文件 | 63 ms |
| knowledge 无关查询 | 10,000 条记录 | 访问历史记录 0 条 |

本次 status 目标（<50 ms）和增量 observation 目标（<100 ms）均达成。首次未缓存扫描会单独
测量；验收目标适用于增量缓存命中路径。原始命令输出必须与发布候选的验收记录一并保留。

## 历史配对采集（WI-876）

当前候选在同一台 `aarch64-apple-darwin` 机器、`rustc/cargo 1.98.1` 和 Runtime `0.2.93` 上，使用外部 baseline 二进制与独立构建的候选二进制进行配对采集。覆盖七种隔离夹具：小型干净、多文件干净（ORG-X）、大量历史 Work Item（ai-investigation-orchestrator）、单文件变更、多文件变更、大文件变更和 evidence 路径变更。每项操作均有 100 个有效 warm 样本；原始样本、身份、计数及不可用原因保存在 `.ai/evidence/WI-876-performance-current-proof/paired-current-seven-scenarios.json`。

配对比较使用 5 ms 噪声预算，**尚不能证明改善**：23 项在噪声内、2 项改善、10 项超过临时噪声预算。这是测量证据，不是发布性能通过。诊断 on/off 开销单独记录在 `diagnostics-overhead-org-x.json`。开发周期成本另行报告：当前仅取得 Contract→checkpoint 的 63,000 ms 单样本，agent 操作/前置拒绝计数、验证→finish 和合并后清理均明确不可用。

## 历史对象仓库采集（WI-889）

WI-889 在与 `0.2.93` 基线相同的 `aarch64-apple-darwin`、Rust/Cargo
`1.98.1` 环境上，补充了当前 `0.2.95` 的配对采集。goods-garden、sentinel
和 ai-investigation-orchestrator 只通过临时视图观察。七个场景共 38 项操作
比较，每项都有 100 个有效 warm 样本；在 5 ms 噪声预算下，p50/p95 判定全部为
`within_noise`。p99 也作为尾部诊断保留。这证明了可比较的当前证据，但没有证明
速度已经改善。由于所有提供对象仓库都超过 harness 的 `<=100` 个 tracked 文件
阈值，small-clean 场景不可取得。完整原始采集和计数保存在 WI-889 证据归档中。

## 当前 v0.2.98 补充采集（WI-905）

WI-905 在不改写 WI-876、WI-889 历史证据的前提下，补充公开版本身份。
在同一台 `aarch64-apple-darwin`、Rust/Cargo `1.98.1` 环境中，对公开
`v0.2.93` 与 `v0.2.98` 二进制进行配对，并保证每个比较操作有 100 个有效
warm 样本。选取的干净对象视图为 goods-garden（396 个 tracked 文件）和
ORG-X（1,240 个 tracked 文件）；没有修改或合并任一主分支。精确环境身份、
原始采集、校验和保存在 `.ai/evidence/WI-905-performance-v098-evidence/raw/`。

首轮十项比较使用既有 5 ms 噪声预算，出现 3 个暂定回归标记。随后独立的
100 样本 observe 重测没有重现 observe 标记（goods-garden：p50 -2.397 ms、
p95 +2.148 ms；ORG-X：p50 -0.240 ms、p95 +1.314 ms）；inspect 尾部尚未重测，
仍是未解决诊断。内部 `git_snapshot` p95 在 goods-garden 从 30.687 ms 降至
24.813 ms，在 ORG-X 从 34.796 ms 降至 31.576 ms，但端到端 CLI 仍不能证明
用户可见提速。候选 `work-item-outcome --delivery --json` 的 100 个有效样本为
p50 241.329 ms、p95 264.831 ms、p99 371.828 ms。诊断开关开销单独报告；
Runtime 未暴露的缓存失效事件保持为 unknown，不以零代替。除已取得的
Contract→可评审 PR 样本外，其他开发周期阶段仍不可用，也不写成零。
