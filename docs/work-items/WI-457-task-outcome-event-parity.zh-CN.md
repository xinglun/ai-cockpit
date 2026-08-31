---
author: AI Cockpit maintainers
title: "WI-457——Task Outcome 事件语义对齐"
workItemId: WI-457-task-outcome-event-parity
description: "增加 Rust 原生、仓库绑定的追加式 Task Outcome 事件投影与 finding/risk 指纹校验。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-457-task-outcome-event-parity
terminalArchive: .ai/work-items/archive/WI-457-task-outcome-event-parity.contract.json
terminalVerification: .ai/evidence/WI-457-task-outcome-event-parity.verification.json
terminalFinalization: .ai/decisions/WI-457-task-outcome-event-parity.finalize.d2e8f8795a6a88fc3fcd8bf2633813d2e20d0443e4c48397b5bab254b0ba8a70.json
terminalDecision: .ai/decisions/WI-457-task-outcome-event-parity.close.json
---

# WI-457——Task Outcome 事件语义对齐

WI-457 增加仓库绑定的 Rust Task Outcome 事件投影。事件流保持追加式，校验
identity 与 evidence 引用，并为 finding/risk 事件记录确定性指纹，避免把
重复问题静默计为新的进展。本实现是对本地参考源的语义对齐，不是 Python
wire 兼容。

[English](WI-457-task-outcome-event-parity.md) · [日本語](WI-457-task-outcome-event-parity.ja.md)

## 已交付边界

- 严格校验参考事件族、correction/supersession 顺序、仓库与 Work Item
  identity、安全 evidence 路径和未知字段。
- 为 finding/risk 事件生成确定性 `findingFingerprint`；除明确关联
  correction/supersession 外拒绝重复指纹。
- Runtime 为 Outcome 报告章节生成追加式事件，但不擅自生成权限、批准、发布、
  provider assurance 或用户收益。
- archive 保持事件字节不变，close 时再次校验事件流。
- 三语文档说明语义、隐私、本地化和非 wire 兼容边界。

## 验证证据

终态 verification 保存在 `.ai/evidence/WI-457-task-outcome-event-parity.verification.json`；
archive/close 记录绑定相同的仓库、Contract 和 Runtime identity。finalization 历史记录
了经过审查的 merge observation 及精确的分支/worktree 清理；不可变 Runtime 记录未被改写。

## 相关文档

- [Task Outcome 事件](../reference/task-outcome-events.zh-CN.md)
- [Task Outcome 报告](../features/task-outcome-report.zh-CN.md)
- [参考源对齐](../reference/reference-parity.zh-CN.md)
