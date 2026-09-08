---
author: AI Cockpit maintainers
title: WI-648 — status 命令瓶颈诊断
description: 在尝试任何优化之前，先找出 status 命令主导耗时的根本原因。
workItemId: WI-648-status-bottleneck-diagnosis
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-648-status-bottleneck-diagnosis
terminalArchive: .ai/work-items/archive/WI-648-status-bottleneck-diagnosis.contract.json
terminalVerification: .ai/evidence/WI-648-status-bottleneck-diagnosis.verification.json
terminalFinalization: .ai/decisions/WI-648-status-bottleneck-diagnosis.finalize.json
terminalDecision: .ai/decisions/WI-648-status-bottleneck-diagnosis.close.json
---

# WI-648 — status 命令瓶颈诊断

本 Work Item 是 AI Cockpit 性能优化专项 P0 阶段的测量/诊断步骤，不改动任何生产
代码或治理行为。它以可复现的证据找出 `ai-cockpit status` 明显慢于
`inspect`/`doctor`/`observe` 的根本原因，让下一个优化 Work Item 拿到一个
已验证的目标，而不是猜测。

## 起始证据

WI-647 修复后的基准测试工具对本仓库的测量结果（macOS arm64，已安装的 v0.2.87
二进制，10 次迭代）：

| 命令 | cold (ms) | warm p50 (ms) |
|---|---|---|
| inspect | 58.3 | 58.3 |
| status | 1788.5 | 1848.4 |
| doctor | 48.0 | 40.7 |
| observe | 109.4 | 105.9 |

无论 cold 还是 warm，`status` 都比其他任何被测命令慢约 20–40 倍。
`inspect`/`doctor`/`observe` 都没有这种开销，说明原因在于 `status` 自身的代码
路径，而不是所有命令共同承担的成本（进程启动、身份解析或 Git 快照捕获）。

## 方法

`crates/cockpit-cli/src/main.rs:686` 显示，只有 `status` 命令会调用
`cockpit_repository::status_with_runtime(&repo, Some(&runtime_context))`；
其他被测命令都走不同的路径。`status_with_runtime` 会调用
`repository_readiness_from_snapshot_with_runtime`
(`crates/cockpit-repository/src/lib.rs:3452`)，其内部依次执行五个步骤。
为了将耗时归因到具体步骤，我们用一个临时的本地补丁（从未提交）为每个步骤包上
`std::time::Instant`/`eprintln!`，例如：

```rust
let __t4 = std::time::Instant::now();
let historical_finalization = historical_finalization_inventory(root, runtime)?;
eprintln!("__PROFILE__ historical_finalization_inventory {:?} count={}", __t4.elapsed(), historical_finalization.len());
```

用 `cargo build --release -p cockpit-cli` 构建后，直接针对本仓库以及新
`attach` 的临时夹具运行，提交前用 `git checkout -- crates/cockpit-repository/src/lib.rs`
还原。任何人都可以根据上面引用的行号复现这一过程。

## 发现：是 O(n²) 的目录重复扫描，而非 O(n) 的历史增长

针对本仓库各步骤的耗时（首次调用预热 OS 缓存之后的 warm 值；`.ai/decisions`
下有 1573 个条目，其中 383 个是 `*.finalize.json`；`.ai/work-items/archive`
中有 580 多个已归档 Work Item）：

| 步骤 | 耗时 | 说明 |
|---|---|---|
| `discover_default_base` | 约 9 ms | 一次 Git 调用 |
| `non_governance_changed_paths` | 小于 1 µs | 在已有快照上的纯内存操作 |
| `unclosed_archived_work_items_with_id` | 约 90 ms | 对 `.ai/work-items/archive` 扫描一次；每项都是 O(1) 文件查找 |
| `classify_historical_debt` | 约 0 µs | 本仓库中 unclosed 项目数为 0 |
| `historical_finalization_inventory` | **约 1.7 秒** | 主导成本 |
| `count_suffix` + `orphaned_active_artifact_names` | 小于 10 µs | `active/` 目录很小 |

对一个刚 `attach` 的空仓库（`.ai/decisions` 下 0 个条目）而言，同样两个步骤合计
只需约 0.05–0.2 ms —— 这项成本并非每个仓库都要承担，只有积累了历史的仓库才会。

`historical_finalization_inventory`（`crates/cockpit-repository/src/lib.rs:3832`）
会遍历 `.ai/decisions` 中每一个 `*.finalize.json` 文件。对于其中记录的
`runtimeVersion`/`runtimeDigest` 与当前运行的 Runtime 不一致的条目（第 3923 行）——
这对几乎所有由旧版本 Runtime 写入的 receipt 都成立，也就是说，对任何有真实历史的
仓库而言这是常见情形而非边缘情形——会调用
`resolve_resource_finalization_head`（`crates/cockpit-repository/src/lib.rs:11634`）。
而这个函数本身又会执行 `fs::read_dir(root.join(".ai/decisions"))`（第 11654 行），
筛选出属于该 `work_item_id` 的 transition 文件。由于这个内层扫描在外层循环的
每一次迭代中都要重新读一遍同一个 `.ai/decisions` 目录列表，总成本是
**O(decisions 目录条目数 × 匹配到的历史 receipt 数)**，而不是 O(条目数)：
本仓库有 1573 个目录条目和 383 个历史 receipt，相当于约 60 万次目录条目比对，
再加上每个匹配 `{work_item_id}.finalize.` 前缀的文件名（本仓库存在 156 个
transition 文件）都要付出一次 `read_resource_finalization_transition` +
`serde_json::to_value` + `digest_json` 的代价。我们只拷贝了 100 个真实的
`.finalize.json` 文件、但不带其对应的 `.ai/work-items/archive/*` 配套文件做了
部分复现，结果只耗时约 20–166 ms，这证实了触发昂贵分支需要完整的归档上下文，
仅靠文件数量无法解释——这排除了"仅文件数量导致"的说法，并把原因收窄到上述
解析链路。

`unclosed_archived_work_items_with_id`（`crates/cockpit-repository/src/lib.rs:3660`）
**没有**表现出同样的模式：它对每一项的检查
（`close_decision_is_valid_for_status`，第 14781 行）是按已知文件名的直接
O(1) 路径查找，而不是目录扫描，这正是它只需约 90 ms（一次目录列举加 580 多次
O(1) 查找）的原因。

## 影响

这整个约 1.7 秒的计算是无条件且未缓存的：每一次 `status` 调用都会完整地重新
执行一遍，尽管对于任何 `runtimeVersion`/`runtimeDigest` 与当前 Runtime 不同的
receipt 而言，其结果是完全确定性的——只要仓库状态不变，反复调用 `status`
每次都在重新计算同一个答案。由于这项成本相对于 `.ai/decisions` 的条目数是二次
函数关系，它恶化的速度会比仓库历史 Work Item 数量增长得更快，这不仅是本仓库的
问题，而是任何拥有真实开发历史的 adopter 仓库都会遇到的结构性问题。

## 对下一个（P1）Work Item 的建议

按照 WI-402/WI-647 的纪律，以下两个独立、范围狭窄、保持语义不变的候选方案应
分别测量（不堆叠、不笼统报告收益）：

1. 每次调用 `historical_finalization_inventory` 时只读取一次 `.ai/decisions`，
   按 `work_item_id` 前缀在内存中建立分组索引，并将该索引传给
   `resolve_resource_finalization_head`，而不是在外层循环的每一项都重新扫描
   目录。由于针对某个 `work_item_id` 的候选集合在两种做法下完全相同，这样可以
   在不改变任何输出值的前提下消除 O(n²) 项。
2. 仅在测量完 (1) 之后再考虑：当某个 receipt 的内容摘要与当前 Runtime 身份
   相对于同一仓库上一次 `status` 调用均未变化时，缓存已解析的
   `HistoricalFinalizationInventoryItem`，并遵循 WI-402 为 verification reuse
   建立的相同 fail-closed/身份规则。

本 Work Item 不实现上述任何一个候选方案；它只是把一个有行号引用、已验证的目标
交给下一个 P1 Work Item，而不是一个假设。

## 验证

未改动任何 Rust 源码、测试或治理文件。`cargo fmt`、`cargo clippy`、
`cargo test --workspace` 相对于已经通过的基线没有变化。上述测量方法可由任何
审阅者根据引用的行号和命令复现。
