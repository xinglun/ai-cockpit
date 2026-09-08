---
author: AI Cockpit maintainers
title: WI-653 — 纯化 Outcome 展示层
description: render_human_outcome 不再获取或校验仓库事实；新增用例层一次性组装。
workItemId: WI-653-outcome-render-purification
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-653-outcome-render-purification
terminalArchive: .ai/work-items/archive/WI-653-outcome-render-purification.contract.json
terminalVerification: .ai/evidence/WI-653-outcome-render-purification.verification.json
terminalFinalization: .ai/decisions/WI-653-outcome-render-purification.finalize.json
terminalDecision: .ai/decisions/WI-653-outcome-render-purification.close.json
---

# WI-653 — 纯化 Outcome 展示层

本 Work Item 是 AI Cockpit 架构优化专项的 P1-A：让
`crates/cockpit-repository/src/outcome_render.rs` 成为一个只处理已观察、已
验证事实的纯展示层。

## 发现的问题

`render_human_outcome(root: &Path, outcome: &OutcomeV2, language: &str)`
接收一个仓库根路径，并在渲染路径内部自行执行仓库 I/O 与治理校验：

- 检查归档 Contract 文件是否存在，并调用
  `close_decision_is_valid_for_status`（一个读取
  `.ai/decisions/{id}.close.json` 的治理检查）来推导 `archived_unclosed`。
- 调用 `load_human_decision(root, work_item_id)`，该函数再次读取
  `.ai/decisions/{id}.close.json` 并校验 repository-id 绑定、记录状态、
  决定确认状态，以及结构化决定的每一个字段——完整的治理校验逻辑被嵌入在一个
  文本格式化函数内部。

现有的全部四个调用点（`crates/cockpit-cli/src/main.rs` 中的
`WorkItemCommand::Outcome`、`print_lifecycle_result`、
`emit_blocked_lifecycle_handoff`；`crates/cockpit-mcp/src/lib.rs` 中的
`work_item_outcome`）都已经在调用 `render_human_outcome` 之前先调用了
`outcome_v2_with_runtime`——这一冗余读取是所有调用方共同的结构性问题，而非
某一处偶然造成的。

## 改动内容

- 新增 `OutcomeRenderInput { outcome: OutcomeV2, human_decision:
  HumanDecisionProjection, archived_unclosed: bool }`，以及两个组装函数
  `outcome_render_input`/`outcome_render_input_with_runtime`，各自只调用一次
  `outcome_v2`/`outcome_v2_with_runtime` 以及现有的
  `close_decision_is_valid_for_status`/`load_human_decision` 逻辑。
- `render_human_outcome` 现在只接收 `&OutcomeRenderInput` 和语言代码——函数
  体内不再有任何 `root: &Path`、文件系统访问或治理校验调用。
- `HumanDecisionProjection`（此前是渲染模块内部的私有枚举）现在是 `pub`，
  因为它是公开的 `OutcomeRenderInput` 的字段。
- 全部四个调用点都迁移为先调用 `outcome_render_input(_with_runtime)`，再调用
  `render_human_outcome(&input, language)`；`--json`/机器可读 JSON 分支使用
  `input.outcome`（与之前完全相同的 `OutcomeV2` 值），因此 JSON 输出不受
  影响。
- 更新了 `cockpit-repository` 与 `cockpit-mcp` 中调用旧的
  "root + OutcomeV2" 签名的 6 个现有测试，改为新的"先组装后渲染"流程，
  同时保留它们原有的文本结果与 `OutcomeV2` 字段断言。

未改动任何文案、本地化字符串、JSON schema 或治理判定逻辑。

## 正确性验证

- 5 个新增单元测试（`crates/cockpit-repository/src/outcome_render.rs` 中的
  `#[cfg(test)] mod render_tests`）完全在内存中构建 `OutcomeRenderInput`——
  不使用临时目录，也不涉及文件系统——覆盖了：人工决定缺失、有效的人工决定、
  无效/畸形的人工决定、已归档但未关闭、历史/被替代等场景。这直接证明了渲染
  函数无需仓库访问即可测试。
- `cockpit-repository` 的 `recovery_decision.rs`、`archive_integrity.rs`、
  `evidence_assurance.rs`、`status_projection.rs`、`recovery_events.rs`，
  以及 `cockpit-mcp` 的 `rpc.rs` 中更新后的 6 个集成测试，其对渲染文本的
  断言（例如 `"Outcome: 🟡"` 前缀、`"provider finalization"` 恢复文案、
  `"决定: continue"` 人工决定文案）均无变化地通过——证明了新旧代码路径在这些
  测试覆盖的每种情形下逐字节输出一致。
- `cargo test --locked --workspace` 通过（整个工作区 120 多个 test result
  区块，0 个失败）。

## 验证

`cargo fmt --all -- --check` 与 `cargo clippy --locked --workspace
--all-targets --all-features -- -D warnings` 均通过。CLI/MCP 的 JSON 输出、
退出码、治理语义均未改变——改变的只是生成人类可读 handoff 文本的内部 Rust
调用路径。

## 不在本次范围内

P0 的职责/依赖关系调查（另一个 Work Item）、P1-B 的观察上下文、P2 的生命周期/
存储职责提取、P2-B 的状态类型、P2-C 的多文件一致性，以及 P3 的物理执行边界，
均未在本 Work Item 中处理。
