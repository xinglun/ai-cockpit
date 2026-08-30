---
author: AI Cockpit maintainers
title: "WI-428——恢复链收敛"
description: 收束剩余恢复边界，并阻止竞争 successor。
audience: [contributor, maintainer]
status: in-progress
authority: governed
workItemId: WI-428-recovery-chain-finalization
predecessorWorkItemId: WI-426-recovery-successor-binding
lastVerifiedBy: WI-428-recovery-chain-finalization
---

# WI-428——恢复链收敛

## 意图与边界

本 Work Item 通过直接绑定的 successor 解决 WI-426 剩余的恢复边界，并使恢复选择
确定化。所有前置 Work Item 的 Contract、Summary、Outcome、Events、evidence 与
recovery receipt bytes 均保持不可变。

范围包括：

- 拒绝针对不同 Work Item 的第二个 `successor` 决定；
- 保留追加写入的 retry/supersede 决定和稳定的 fail-closed 错误；
- 在三份 reference-parity 台账中登记实际终态 receipt；
- 记录单 successor lineage 规则，并用 Rust 测试验证。

范围外：发布产物、无关 Work Item、全局 Agent/MCP 配置和 Runtime 架构拆分。

## 验收与证据

前置项不得积累含义不明确的竞争 successor 链。WI-426 必须由直接绑定且经过评审的
successor 表示，WI-424 必须由不可变 supersede receipt 表示，三份 parity 台账必须
指向实际 receipt。不得改写历史 bytes。竞争 successor 请求必须以
`recovery_decision_invalid:competing_successor` fail closed。

Verification evidence、archive manifest、finalization、close 和合并后的 PR 均记录在
`.ai/evidence/` 与 `.ai/decisions/` 下。

[English](WI-428-recovery-chain-finalization.md) · [日本語](WI-428-recovery-chain-finalization.ja.md)
