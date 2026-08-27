---
author: AI Cockpit maintainers
title: "WI-320——checkpoint evidence 快照生命周期"
workItemId: WI-320-checkpoint-evidence-snapshot-lifecycle
description: "允许事前编辑检查点保留历史快照，同时要求终态检查点绑定当前快照。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-320-checkpoint-evidence-snapshot-lifecycle
terminalArchive: .ai/work-items/archive/WI-320-checkpoint-evidence-snapshot-lifecycle.contract.json
terminalVerification: .ai/evidence/WI-320-checkpoint-evidence-snapshot-lifecycle.verification.json
terminalFinalization: .ai/decisions/WI-320-checkpoint-evidence-snapshot-lifecycle.finalize.json
terminalDecision: .ai/decisions/WI-320-checkpoint-evidence-snapshot-lifecycle.close.json
---

# WI-320——checkpoint evidence 快照生命周期

## 意图与边界

`before_edit` 是在实现前记录的授权边界。后续编辑和新的 preflight 必然会
产生更新的仓库快照；这段历史必须保持有效，同时不能削弱
`before_finish` 边界。终态检查点仍必须绑定当前 Contract、仓库和快照，
声明的 verification check 也必须对应真实结果。

## 范围与验收

- 身份、结构、阶段和 amendment chain 有效的历史 `before_edit` 与 amendment
  记录可以保留；它们不能被静默当作当前终态证据。
- `before_finish` 必须绑定当前快照；过期、跨仓库、格式错误、重复或符号
  链接证据都必须 fail closed。
- 必需 checkpoint check 必须确定性推导，不能产生 verification 无法提供的
  虚构名称。
- amendment、resume、生命周期和仓库隔离回归测试继续通过。
- 英文、简体中文和日文文档说明该时间性证据边界，并链接最终 Runtime 回执。

## 验证

运行 checkpoint/lifecycle 定向测试、锁定 workspace 测试、文档/parity 门和
审查分支的托管检查。所有仓库绑定 Runtime 命令都显式传入仓库路径。

## 不在范围内

Planner 与并行执行、性能、CI/release/adopter harness、全局 Agent 或 MCP
配置，以及后续大型 repository 模块的架构拆分均不在本次有界修正内。

[English](WI-320-checkpoint-evidence-snapshot-lifecycle.md) ·
[日本語](WI-320-checkpoint-evidence-snapshot-lifecycle.ja.md)
