---
author: AI Cockpit maintainers
title: "WI-753——P1 中断与恢复过程中的 Runtime 一致性"
description: "在受控测试仓库中增加可执行检查，验证展示状态、选择的选项与 Runtime 行为一致，并覆盖安全中断与恢复。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-753-p1-runtime-consistency
status: implemented
authority: authorized
lastVerifiedBy: WI-753-p1-runtime-consistency
terminalArchive: .ai/work-items/archive/WI-753-p1-runtime-consistency.contract.json
terminalVerification: .ai/evidence/WI-753-p1-runtime-consistency.verification.json
terminalFinalization: .ai/decisions/WI-753-p1-runtime-consistency.finalize.json
terminalDecision: .ai/decisions/WI-753-p1-runtime-consistency.close.json
---

[English](WI-753-p1-runtime-consistency.md) · [日本語](WI-753-p1-runtime-consistency.ja.md)

# WI-753——P1 中断与恢复过程中的 Runtime 一致性

## 意图

交付协作语言专项第四节：在临时受控仓库中证明 Runtime 展示的状态和可用选项
与选定选项及 Runtime 实际允许的转换一致，并覆盖验证被中断后的安全重试。

## 边界

本 Work Item 修改一个 CLI 集成测试和三语协作参考页面。生产行为、Outcome
schema、外部参与者，以及另行批准的 P2 入门/贡献轨道均不在范围内。模拟的人
工选择明确标记为 `TEST DATA ONLY`，只写入临时 fixture，绝不会成为本仓库的授
权记录。

## 验收

- `crates/cockpit-cli/tests/collaboration_consistency.rs` 断言展示
  `confirm_review` 选项、未选择前的拒绝、选择后 checkpoint、中断进程后没有
  验证通过，以及恢复后的当前证据。
- 验证投影变化会使旧 preflight 回执失效；测试在断言当前
  `human_decision_recorded` 前为 fixture 记录新的决定。
- 场景矩阵将此实际观测检查登记为 SCN-025，不宣称穷尽全部状态空间。
- `cargo fmt --check`、定向测试、`cargo clippy --tests -- -D warnings` 以及
  文档验收检查全部通过。

## 证据

- archive: `.ai/work-items/archive/WI-753-p1-runtime-consistency.contract.json`
- verification: `.ai/evidence/WI-753-p1-runtime-consistency.verification.json`
- finalization: `.ai/decisions/WI-753-p1-runtime-consistency.finalize.json`
- close: `.ai/decisions/WI-753-p1-runtime-consistency.close.json`
