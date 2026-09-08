---
author: AI Cockpit maintainers
title: WI-649 — 修复 status 的 O(n²) 历史扫描瓶颈
description: 消除 WI-648 发现的 .ai/decisions 重复重扫，并给出前后测量证据。
workItemId: WI-649-status-history-scan-fix
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-649-status-history-scan-fix
terminalArchive: .ai/work-items/archive/WI-649-status-history-scan-fix.contract.json
terminalVerification: .ai/evidence/WI-649-status-history-scan-fix.verification.json
terminalFinalization: .ai/decisions/WI-649-status-history-scan-fix.finalize.json
terminalDecision: .ai/decisions/WI-649-status-history-scan-fix.close.json
---

# WI-649 — 修复 status 的 O(n²) 历史扫描瓶颈

本 Work Item 是针对 WI-648 找到的瓶颈的 P1 修复：`status` 之所以比
`inspect`/`doctor`/`observe` 慢 20–40 倍，是因为 `historical_finalization_inventory`
通过 `resolve_resource_finalization_head`，为每一条历史 receipt 都重新扫描一遍完整的
`.ai/decisions` 目录。

## 改动内容

`crates/cockpit-repository/src/lib.rs`：

- `historical_finalization_inventory` 现在只读取一次 `.ai/decisions`，把目录列表
  物化到内存，并从这一次列表中构建 `work_item_id -> transition 候选` 的索引（按每个
  文件名中第一个 `.finalize.` 边界分组；work item ID 不能包含 `.`，因此这是精确、
  无歧义的切分）。
- 将 `resolve_resource_finalization_head` 拆分为一个薄封装（签名、行为、目录扫描均
  不变）和一个新的 `resolve_resource_finalization_head_with_candidates`：后者执行
  相同的链式解析逻辑，但消费调用方提供的候选列表，而不是自己扫描目录。
- `historical_finalization_inventory` 改为调用 `_with_candidates` 版本，传入已经
  分组好的候选（调用 `resolve_resource_finalization_head` 会导致重新扫描）。其余
  三个调用点（`finalize`、`finalize-verify`、`finalize-recovery-plan`）仍然调用
  未改动的封装函数——本 Work Item 完全没有触碰它们的行为、成本或签名。

没有任何输出字段、schema 或治理判定发生变化。这是一次纯粹的内部重构：只是把一次
目录列表读取结果复用了起来。

## 正确性证据

- `cargo test --locked --workspace` 无变化地通过（121 个 test result 区块，
  0 个失败）。
- 新增测试 `crates/cockpit-repository/tests/historical_finalization_scan.rs` 在
  同一个仓库中构建了两个独立的历史 Work Item——ALPHA 带有两步 transition 链
  （sequence 2），BETA 没有（sequence 0）——并断言 `status_with_runtime` 的
  `historicalFinalization` 条目与*未改动*的 `resolve_resource_finalization_head`
  （通过 `verify_resource_finalization` 独立计算）得到的真值一致，从而证明新索引
  不会在不同 Work Item 之间混淆候选。
- 逐字节直接对比：在本仓库（580 多个已归档 Work Item，383 个历史
  `.finalize.json` receipt）上运行 `ai-cockpit status --repo`，将已安装的 v0.2.87
  二进制的 JSON 输出与本 Work Item 的 release 构建输出对比，除 `runtimeDigest`
  （因二进制字节改变而必然变化）外，所有字段完全一致。全部 383 条
  `historicalFinalization` 记录的 `state`、`sequence`、`predecessorDigest`、
  `historicalKind`、`safeActions` 均一致。

## 测量到的性能（参考信息）

2026-09-08，macOS arm64（与 WI-648 诊断时同一仓库），使用 WI-647 修复后的
冷/热分组测量工具（`tests/performance/runtime_benchmark.sh`；对本仓库做外部调用，
本 Work Item 本身未修改该脚本），12 次迭代，将 baseline 与本 Work Item 的 release
构建做了两组独立测量对比：

| 命令 | 测量组1 | 测量组2 |
|---|---|---|
| status.cold | -20.3% | -22.9% |
| status.warm | -21.2% | -22.0% |

`status` 稳定改善约 20–23%（从约 1.86–1.96 秒降到约 1.48–1.51 秒）。更细粒度的
逐步骤剖析（与 WI-648 相同的临时、从未提交的测量手法）显示，
`historical_finalization_inventory` 本身从约 1.7 秒（WI-648 的基线）降到约
1.24–1.37 秒——被消除的重复目录列表读取正好对应这部分差值。剩余约 1.24 秒花在
逐条 receipt 的处理上（`closed_finalization_projection_kind`、
`archived_contract_digest`，以及 transition 文件本身的读取），本 Work Item 未对此
做改动；要进一步优化，需要未来的 Work Item 另行诊断。

`inspect`/`doctor`/`observe` 的差值在重复测量中噪声很大、正负号会反转（例如
`doctor.warm` 在一组测量中为 +9.2%，而对*同一个*未改动二进制、放在不同文件系统
位置与自身比较的两次测量中，`inspect.cold` 也分别出现 +0.9%/-14.1%）——这些命令
根本不会调用 `historical_finalization_inventory` 或
`resolve_resource_finalization_head`（根据
`crates/cockpit-cli/src/main.rs:686`，只有 `status_with_runtime` 会调用），因此
本改动没有任何代码路径能影响它们；它们较小的绝对量级（低于 120 ms）正是修复后的
测量工具（WI-647）自身 `p95Unreliable`/`p50Unreliable` 可靠性防护要标记的情形。
以上均为本地进程延迟观测，不构成 provider 或 enterprise 层面的保证。

## 不在本次范围内

将 `historical_finalization_inventory` 的结果跨多次 `status` 调用缓存起来
（WI-648 的第二个候选方案），以及诊断剩余约 1.24 秒的逐条 receipt 成本，留给
未来的 Work Item。
