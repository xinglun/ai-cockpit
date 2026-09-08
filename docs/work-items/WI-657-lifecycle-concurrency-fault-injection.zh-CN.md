---
author: AI Cockpit maintainers
title: WI-657 — 并发 finish_work_item 故障注入
description: 一个受控的双线程故障注入测试发现并修复了 atomic_write 中一个真实存在的竞争问题，并记录了一个被有意搁置、未修复的更深层问题。
workItemId: WI-657-lifecycle-concurrency-fault-injection
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-657-lifecycle-concurrency-fault-injection
terminalArchive: .ai/work-items/archive/WI-657-lifecycle-concurrency-fault-injection.contract.json
terminalVerification: .ai/evidence/WI-657-lifecycle-concurrency-fault-injection.verification.json
terminalFinalization: .ai/decisions/WI-657-lifecycle-concurrency-fault-injection.finalize.json
terminalDecision: .ai/decisions/WI-657-lifecycle-concurrency-fault-injection.close.json
---

# WI-657 — 并发 finish_work_item 故障注入

本 Work Item 是架构优化专项的 P2-C：通过受控的故障注入验证并发情形下的
多文件一致性与恢复能力，且先调查现有机制，不预设缺陷一定存在。

## 测试了什么，为何选择它

`finish_work_item_internal` 是 P0 地图
（`docs/reference/architecture-responsibility-map-2026-09.md`）中已识别出的、
体量最大的"读取+判定+持久化"混合函数，此前从未被两个调用方针对同一个
Work Item 的竞争情形所验证过。现有测试套件已经覆盖了失败后的顺序重试
（`recovery_decision.rs`）以及单一调用方的部分写入回滚，但从未出现过两个
线程同时对同一个 Work Item 调用 `finish_work_item` 的情形。新增的测试
`crates/cockpit-repository/tests/lifecycle_concurrency.rs` 正是做这件事：
两个通过 `std::sync::Barrier` 同步的线程，在同一个已到达 `checkpointed`
状态的 Work Item 上同时调用 `finish_work_item`。

## 发现一（已修复）：atomic_write 的临时文件名冲突

`atomic_write`（`crates/cockpit-repository/src/lib.rs`）此前仅凭
`std::process::id()` 派生临时文件名。因此同一进程内的两个线程写入同一个
目标路径时，会争抢*完全相同*的临时文件路径：后调用 `fs::rename` 的那个
线程会发现自己的临时文件已被对方的重命名消耗掉，从而在目标路径上失败，
报出一个误导性的文件系统级"未找到"错误，而不是一个业务层面的拒绝。这是
一个真实存在的缺陷，已通过实证确认（修复前反复运行会复现该问题，修复后
不再复现）。

修复方式是将 `NEXT_ATOMIC_WRITE_ID` 这个已存在的原子序列计数器与 pid
配对——这与本文件中 `write_cap_immutable` 以及并行槽位租约写入器已经采用
的模式完全相同，不是新引入的机制，只是将一个已被验证过的既有方案应用到
第四个调用点。此次修复仅限于 `atomic_write`，未改动任何其他函数。

## 发现二（有意搁置，未修复）：回滚可能覆盖并发中的成功一方

即使修复了临时文件名问题，落败的线程仍可能因为一个合法的业务拒绝（例如
重复完成事件检查）而失败，而该落败线程在 `finish_work_item_internal` 中
手写的回滚逻辑（`atomic_json(&summary_path, &original_summary)` 调用）
会无条件地恢复*自己*尝试前的旧快照。它并不检查磁盘上的当前状态是否已经
被并发中成功的另一方推进。这可能会把一次合法的并发成功回退到
`checkpointed` 状态，悄悄地抹去一次成功的 `finish`。

这是一个真实存在的正确性缺口，而非假设性的问题。要正确修复它，需要在
`finish`/`archive`/`close` 周围引入互斥边界（锁，或者在回滚写入之前对照
summary 自身的摘要/状态做比较后再交换的 compare-and-swap 检查），这是一项
超出本 Work Item 范围、体量更大、风险更高的改动。遵循本专项自身的风险
纪律（WI-654 曾放弃一个未经充分验证的修复的先例），本 Work Item 不尝试
修复它，而是将其作为一个已知的、被有意搁置的局限性记录在此，而不是
不加说明地放任不管。

## 测试设计

新增测试只验证此次修复所能保证的内容，不验证仍未解决的第二个问题：

- 两次并发 `finish_work_item` 调用中至少有一次成功。
- 无论成功还是失败，任何结果都不包含文件系统竞争的原始痕迹
  （"No such file or directory" / "os error 2"）。
- 无论哪次尝试的写入最终留在磁盘上，`summary.json` 和 `outcome.json`
  仍然是有效的、可解析的 JSON。

本测试有意不去断言 Work Item 最终会停在 `finish_ready` 状态，之后也没有
调用 `archive_work_item`——这两者都依赖于发现二被修复，若断言它们，要么
会掩盖这个缺口，要么会让测试在等待更大规模重新设计期间变得不稳定。

## 正确性验证

修复后，`cargo test -p cockpit-repository --test lifecycle_concurrency`
连续运行 6 次均稳定通过（为了让基于 barrier 的故障注入在每次运行中都是
确定性的，使用了单线程测试运行器）。`cargo test --locked --workspace`、
`cargo fmt --all -- --check`、`cargo clippy --locked --workspace
--all-targets --all-features -- -D warnings` 均通过。

## 不在本次范围内/后续工作

将 `finish_work_item_internal` 的回滚重新设计为在并发下安全（发现二）
不在本 Work Item 范围内。`archive_work_item` 和
`close_work_item_with_structured_decision_internal` 中也存在类似的手写
回滚代码，本 Work Item 未对它们进行故障注入——未来的 P2-C 后续工作应当
在尝试修复发现二之前，先将同样的基于 barrier 的技术扩展到这两个函数上，
因为一个真正有效的互斥修复很可能需要同时覆盖这三个函数。
