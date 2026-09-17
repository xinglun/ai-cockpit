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

## 当前配对采集（WI-876）

当前候选在同一台 `aarch64-apple-darwin` 机器、`rustc/cargo 1.98.1` 和 Runtime `0.2.93` 上，使用外部 baseline 二进制与独立构建的候选二进制进行配对采集。覆盖七种隔离夹具：小型干净、多文件干净（ORG-X）、大量历史 Work Item（ai-investigation-orchestrator）、单文件变更、多文件变更、大文件变更和 evidence 路径变更。每项操作均有 100 个有效 warm 样本；原始样本、身份、计数及不可用原因保存在 `.ai/evidence/WI-876-performance-current-proof/paired-current-seven-scenarios.json`。

配对比较使用 5 ms 噪声预算，**尚不能证明改善**：23 项在噪声内、2 项改善、10 项超过临时噪声预算。这是测量证据，不是发布性能通过。诊断 on/off 开销单独记录在 `diagnostics-overhead-org-x.json`。开发周期成本另行报告：当前仅取得 Contract→checkpoint 的 63,000 ms 单样本，agent 操作/前置拒绝计数、验证→finish 和合并后清理均明确不可用。
