---
author: AI Cockpit maintainers
title: WI-654 — 观察上下文去重(调查)
description: preflight 中一个实测到的重复读取，以及为何看似显而易见的修复其实并不安全。
workItemId: WI-654-observation-context-dedup
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-654-observation-context-dedup
terminalArchive: .ai/work-items/archive/WI-654-observation-context-dedup.contract.json
terminalVerification: .ai/evidence/WI-654-observation-context-dedup.verification.json
terminalFinalization: .ai/decisions/WI-654-observation-context-dedup.finalize.json
terminalDecision: .ai/decisions/WI-654-observation-context-dedup.close.json
---

# WI-654 — 观察上下文去重(调查)

本 Work Item 是架构优化专项的 P1-B，范围收窄到 P0 地图
（`docs/reference/architecture-responsibility-map-2026-09.md`）发现的最具体
的重复读取案例。本次未交付任何生产代码改动——看起来显然安全的修复，实际上
并不安全，本文档正是记录这一发现。

## 测量

添加了一个临时的、从未提交的测量手段（包裹 `repository_id` 和
`snapshot_digest` 的原子计数器，提交前必定还原——与 WI-648/649/650 相同的
方法），并针对一个刚 attach、带有一个 active Work Item 的夹具仓库运行：

| 命令 | `repository_id` 调用次数 | `snapshot_digest` 调用次数 |
|---|---|---|
| `preflight` | 3 | 2 |
| `checkpoint` | 3 | 2 |
| `status`(有 active Work Item) | 1 | 0 |

这证实了 `preflight`/`checkpoint` 确实存在真实的、request-scoped 的重复计算，
而不仅仅是理论上的担忧。

## 假设，以及它为何是错的

三次 `repository_id` 调用中有一次发生在 `project_governance_unknowns`
（`crates/cockpit-repository/src/project_governance.rs:352`）内部，用于绑定
一个期望身份来校验 `.ai/project/capabilities.json`。最初的假设是：
`contract_freshness_findings`（`crates/cockpit-repository/src/lib.rs:9330`）
在同一个治理判定中更早执行（`governance_decision_for_contract_base_
internal_with_archive` 在第 9434 行调用 `contract_freshness_findings`，
在第 9461 行调用 `project_governance_unknowns`），并且已经检查过
`contract.repository_id != repository_id(&root)`，因此到 `project_governance_unknowns`
运行时，`contract.repository_id` 应该已经等于最新值，复用它可以在不改变行为的
情况下消除一次对 `.ai/cockpit.toml` 的冗余磁盘读取。

对照实际源码验证后，这个假设被推翻了：

```rust
// crates/cockpit-repository/src/lib.rs:9330
if contract.repository_id != repository_id(&root).to_string() {
    findings.push("stale_contract".into());
}
```

`contract_freshness_findings` 在不匹配时只是**记录**一个 `stale_contract`
finding；它不会提前 `return`，也不会阻止
`governance_decision_for_contract_base_internal_with_archive` 之后继续调用
`project_governance_unknowns`。因此，一个外来的或过期的 Contract（其
`repository_id` 字段已不再匹配实际仓库）仍然可能带着一个未经验证、可能是
错误的 `contract.repository_id` 到达 `project_governance_unknowns`。如果用它
替换 `repository_id(&root)`，就会在这一边缘情形下改变 `load_declaration` 的
`expected_repository_id` 比较，可能产生与当前代码不同的
`project_capabilities_repository_mismatch` 类 unknown——这是一个真实的行为
差异，而不是纯粹的重构，尽管在大多数测试所覆盖的（新鲜且匹配的）常见情形下
它是不可见的。

我们编写了一个实现该替换的试验性改动，在完成上述验证后将其还原，遵循的正是
本专项自身的规则：不出货一个兼容性风险未经实际核实的改动。

## 一个安全的修复需要什么

要安全地消除这次调用，需要把一次性解析好的 `repository_id` 值，穿透传递
给三个函数：`contract_freshness_findings`、`governance_decision_for_contract_
base_internal_with_archive`、`project_governance_unknowns`。
`contract_freshness_findings` 是 `pub fn`，在 `lib.rs` 中有两个调用点
（8900、9434）；根据 P0 地图，治理判定函数族每个都有 2-4 个几乎重复的
`_internal`/`_with_archive`/`_with_runtime` 变体，使用范围远超本 Work Item
所测量的 preflight 路径。正确地改动这一组签名、逐一验证每个调用点行为不变，
并用测试证明，其改动规模远超本 Work Item 实测所能证明的收益（消除一次很小的
本地 TOML 读取）。这项工作留给未来的 Work Item，以更强的实测需求，或与该
函数族的其他必要改动合并为前提。

第二个被测量到的重复（`snapshot_digest` 对同一个、已经在内存中的
`RepositorySnapshot` 值被调用了两次，一次在 `project_governance_unknowns`
内，一次在 `apply_preflight_review_evidence` 内）原则上是更干净的候选——两次
调用都可证明作用于相同的输入——但要打通它同样需要上述范围的签名改动，而且
被消除的成本（一次进程内的摘要计算，只有在工作树脏时才会派生 `git` 子进程）
尚未被测量为显著。出于同样的理由，这里也不予采纳。

## 对 P0 地图的后续补充(推迟)

`docs/reference/architecture-responsibility-map-2026-09.md`(WI-652)把这一
模式列为重复读取的示例，但没有评估任何具体修复方案是否安全。本 Work Item
不修改该文档，因为它在本分支上并不存在(WI-652 尚未合并)；待 WI-652 合并后，
应补充本 Work Item 更具体的发现(那个看似显而易见的修复其实不安全，以及
原因)，以免未来的读者重蹈覆辙、再次尝试同一个已被证伪的替换方案。

## 验证

本 Work Item 的最终状态未改动任何生产代码。`cargo fmt`、
`cargo clippy --all-targets --all-features -- -D warnings`、
`cargo test --locked --workspace` 针对未改动的工作区均通过。
