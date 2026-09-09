---
author: AI Cockpit maintainers
title: "WI-756——P1 仅凭 Runtime 的交接重建"
description: "在受控测试仓库中增加有界检查，验证新的 Agent 或会话无需对话历史即可重建协作交接。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-756-p1-handoff-reconstruction
status: recovered
authority: authorized
lastVerifiedBy: WI-756-p1-handoff-reconstruction-revalidation
terminalArchive: .ai/work-items/archive/WI-756-p1-handoff-reconstruction.contract.json
terminalVerification: .ai/evidence/WI-756-p1-handoff-reconstruction.verification.json
terminalFinalization: .ai/decisions/WI-756-p1-handoff-reconstruction.finalize.json
terminalDecision: .ai/decisions/WI-756-p1-handoff-reconstruction.close.json
---

[English](WI-756-p1-handoff-reconstruction.md) · [日本語](WI-756-p1-handoff-reconstruction.ja.md)

# WI-756——P1 仅凭 Runtime 的交接重建

## 意图

交付协作语言专项第五节：在临时受控仓库中证明新的子进程无需对话历史，即可
重建目标、范围、完成状态、当前证据绑定、授权以及已持久化的阻断原因。

## 边界

本 Work Item 修改一个 CLI 集成测试和三语协作参考页面。生产行为、Outcome
schema、外部参与者招募/访谈/评价，以及另行批准的 P2 入门/贡献轨道均不在范围
内。测试只读取临时受控仓库记录，不会在本仓库创建授权记录。

## 验收

- `crates/cockpit-cli/tests/collaboration_handoff.rs::new_agent_reconstructs_handoff_from_runtime_records_without_conversation_history`
  在新子进程边界上读取 Contract、Summary、status、持久化的 active Outcome 和
  当前验证证据。
- 测试重建目标/范围、完成与待完成状态、当前证据绑定与新鲜度、已授权的
  authority、明确的 `finish.governance` 恢复条件，以及未声明的用户可见收益；
  不推断完成或收益。
- 场景矩阵将该实际观测检查登记为 SCN-026；不变量覆盖和契约页面将其描述为
  有界证据，不以外部参与者验证为前提。
- `cargo fmt --check`、定向测试、`cargo clippy --tests -- -D warnings` 以及
  文档验收检查全部通过。

## 证据

- archive: `.ai/work-items/archive/WI-756-p1-handoff-reconstruction.contract.json`
- verification: `.ai/evidence/WI-756-p1-handoff-reconstruction.verification.json`
- finalization: `.ai/decisions/WI-756-p1-handoff-reconstruction.finalize.json`
- close: `.ai/decisions/WI-756-p1-handoff-reconstruction.close.json`
