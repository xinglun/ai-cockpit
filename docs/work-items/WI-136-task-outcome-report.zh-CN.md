---
author: AI Cockpit maintainers
workItemId: WI-136-task-outcome-report
title: Rust-native Task Outcome 与 Human Benefit 报告
description: 增加 evidence-bound 报告投影、追加事件源和生命周期绑定的报告产物。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-136-close-verification
---

# WI-136 — Rust-native Task Outcome 与 Human Benefit 报告

## Intent

当前 Rust Runtime 有窄版 OutcomeV2 和 human handoff；参考源还把完成内容、问题、停止、
解决、风险、未知、证据和恢复条件显式化。本 WI 增加该投影，但不把 presentation 变成授权。

## 边界

- 新生成 OutcomeV2 带严格的增量 `taskOutcomeReport`，claim 绑定 evidence，section 名称稳定。
- `finish` 写入 typed report JSON、Markdown 投影和追加的 `<id>.events.jsonl`；`archive` 移动并
  计算 digest；`close` 在 repository-bound decision receipt 中记录已校验的 `finalReport` 和 digest。
- 事件 identity、repository/Work Item 绑定、关系顺序、不安全路径和疑似 secret 内容 fail closed。
  历史 bytes 不变。
- CLI 和 MCP 使用同一本地化 renderer。Contract 原文、人类决定、外部 provider claim 和发布事实不变。

## 不在范围内

新的生命周期状态、完整事件驱动的 paused/blocked/stale/cancelled/rollback 重建、adopter capability
manifest、第二技术栈验收、provider identity、全局 Agent/MCP 配置和复制参考源 Python/Make/V1 资产。

## 验收

- Protocol 测试覆盖严格 report schema、未知字段拒绝和 claim provenance。
- Repository 测试覆盖 report/event 生成、malformed/foreign 事件拒绝、archive digest 绑定和 close final report 绑定。
- CLI/MCP 展示同一份 report，保留三语标题和 Contract-language 验收原文。
- 中英日 feature/reference 文档准确描述已实现与后置边界。

## 验证

归档 evidence 与 close decision 是当前权威验证记录：

- `.ai/evidence/WI-136-task-outcome-report.verification.json`
- `.ai/work-items/archive/WI-136-task-outcome-report.archive.json`
- `.ai/decisions/WI-136-task-outcome-report.close.json`
