---
author: AI Cockpit maintainers
title: WI-655 — 将 checkpoint_work_item 拆分为 Observation/Governance/Persistence
description: 第一个被证明可行的 P2-A 用例：同一文件、同一签名、零行为变化的重构。
workItemId: WI-655-checkpoint-responsibility-split
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-655-checkpoint-responsibility-split
terminalArchive: .ai/work-items/archive/WI-655-checkpoint-responsibility-split.contract.json
terminalVerification: .ai/evidence/WI-655-checkpoint-responsibility-split.verification.json
terminalFinalization: .ai/decisions/WI-655-checkpoint-responsibility-split.finalize.json
terminalDecision: .ai/decisions/WI-655-checkpoint-responsibility-split.close.json
---

# WI-655 — 将 checkpoint_work_item 拆分为 Observation/Governance/Persistence

本 Work Item 是架构优化专项的 P2-A：按照 P0 地图"在动
`finish`/`archive`/`close` 之前先从最小的生命周期函数开始"的建议，第一个
完整地证明了 Observation/Governance/Evidence+Projection 职责提取可行的
用例。

## 为什么选这个函数，为什么放在同一文件内

`checkpoint_work_item`（`crates/cockpit-repository/src/lib.rs`）把观察
（读取 `summary.json`/`contract.json`、捕获 Git 快照）、治理判定（状态/
重复/新鲜度/verification 顺序检查，以及对共享 preflight 治理判定的调用）、
持久化（`atomic_json` 写入与 `LifecycleReceipt` 构建）混在一个约 120 行的
函数内。它在 `crates/cockpit-repository/tests/` 中的 17 个文件里已经有测试
覆盖，构成了一张实质性的回归网。

WI-654 已经表明，对相邻治理代码看似显而易见的改动，可能会在现有测试恰好未
覆盖的边缘情形下悄悄改变行为。考虑到这一风险，本 Work Item 有意地将
`checkpoint_work_item` 的函数体重组为命名的内部辅助函数，**在同一个文件内，
不改变公共签名、错误信息、写入的 JSON 字段或写入顺序**——而不是跨模块或跨
crate 的抽取。这是在考虑更大、风险更高的改动之前，证明职责边界的一种保守、
低风险的方式。

## 影响拆分设计的一个细节

原始代码在通过三个只需要已读取的 `summary.json` 字段的低成本检查（重复
checkpoint、生命周期状态、preflight 状态是否存在）之前，不会读取 Contract
或捕获 Git 快照。这个顺序是故意设计成快速失败的：如果 checkpoint 因为一个
低成本的原因就已经无效，函数不应该先付出捕获 Git 快照的代价，结果还是要
拒绝它——更重要的是，如果同时存在多个问题，应该浮现出来的是*第一个*被违反
的检查的错误，而不是重新排序后的版本恰好先碰到的那个错误。

因此本次拆分保留了这个精确的顺序：三个低成本检查仍然内联在
`checkpoint_work_item` 中，在新增的 `checkpoint_observe` 调用之前执行；
只有原始代码中本就需要 Contract 和快照的检查（快照新鲜度、contract 新鲜度、
preflight 治理、verification 顺序）被移到了新增的
`checkpoint_governance_checks` 中，在观察之后按原有的相对顺序调用。

## 改动内容

- `CheckpointObservation`（私有结构体）：`contract_path`、`contract`、
  `snapshot`、`current_snapshot_digest`、`current_contract_digest`。
- `checkpoint_observe(root, work_item_id) -> Result<CheckpointObservation,
  ObserverError>`：正是此前内联的读取 contract + 发现快照 + 计算摘要的代码，
  内容和错误路径都未改变。
- `checkpoint_governance_checks(root, summary_path, summary, preflight_state,
  observation) -> Result<(), ObserverError>`：原始代码中在观察之后运行的
  四个检查，顺序相同，错误信息完全一致。这个辅助函数**并不**声称自己没有
  I/O：`require_green_or_yellow_preflight_governance` 仍然像之前一样在
  内部执行自己的观察。
- `checkpoint_work_item` 保留三个低成本的前置检查，然后调用
  `checkpoint_observe`，接着调用 `checkpoint_governance_checks`，最后执行
  未改动的持久化代码（`append_checkpoint_evidence`、summary 字段写入、
  `atomic_json`、`LifecycleReceipt` 构建）。

## 正确性验证

调用 `checkpoint_work_item` 的全部 17 个现有测试文件
（`agent_risk_checkpoint.rs`、`archive_integrity.rs`、`contract_preflight.rs`、
`intelligence.rs`、`evidence_assurance.rs`、`knowledge_projection.rs`、
`knowledge_cache.rs`、`lifecycle_order.rs`、`outcome_report.rs`、
`preflight_review.rs`、`recovery_events.rs`、`recovery_revalidation.rs`、
`task_outcome_events.rs`、`recovery_decision.rs`、`status_projection.rs`、
`verification_route.rs`、`resource_finalization_transition.rs`）均无变化地
通过——没有新增、删除任何测试，也没有改动任何断言。`cargo test --locked
--workspace` 通过（120 个 test result 区块，0 个失败）。`cargo fmt --all --
--check` 与 `cargo clippy --locked --workspace --all-targets --all-features
-- -D warnings` 均通过。

## 不在本次范围内/后续工作

`finish_work_item`、`archive_work_item`、
`close_work_item_with_structured_decision` 是具有相同"读取+判定+持久化"
混合问题、体量更大、风险更高的函数，本 Work Item 未涉及。
`docs/reference/architecture-responsibility-map-2026-09.md`（WI-652，尚未
合并，本分支上不存在）在合并后应当更新，将此记录为第一个被证明可行的 P2-A
用例。
