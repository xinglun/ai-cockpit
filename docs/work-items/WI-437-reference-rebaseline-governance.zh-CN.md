---
author: AI Cockpit maintainers
title: "WI-437——本地参考源治理规则重新比对"
workItemId: WI-437-reference-rebaseline-governance
description: "重新阅读维护者本地参考源中变化的七个治理文件。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-437-reference-rebaseline-governance
---

# WI-437——本地参考源治理规则重新比对

本文档与 conformance Work Item 重新阅读此前台账之后源内容发生变化的 7 个文件。语义参考源为
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` 本地 checkout，不访问公开参考仓库。
本任务只记录语义对齐决定，不向 Rust 工程复制 Python、Make、YAML 或源 JSON 产物。

[English](WI-437-reference-rebaseline-governance.md) · [日本語](WI-437-reference-rebaseline-governance.ja.md)

## 范围

- 重新阅读 `.ai/cockpit/README.md`、`.ai/cockpit/README.ja.md`、
  `.ai/cockpit/adoption.ja.md`、`.ai/guards/changed_critical_coverage_policy.json`、
  `.ai/guards/coverage_policy.yaml`、`.ai/quality/governance-routing.yaml` 和
  `.ai/schemas/task_outcome.schema.json`。
- 为每个文件记录明确的 Rust 对应物或不可移植原因。
- 更新机器台账、三语比较/ parity 文档和回归断言，不改变 Runtime 行为。

## 文件级决定

7 个文件全部为 `implemented-different-by-design`。源工程变化属于 Python/Make 表面清理：移除过时
`REPORT_LANGUAGE` 参数、删除 Python 专用 coverage 关联、将 route 选择与重复 gate 元数据分离、简化
Python Task Outcome schema。Rust 保留自己的 typed OutcomeV2/humanHandoff 与动态 gate 边界；源 wire
shape 不是兼容性要求。

## 验证

必须通过本地参考源策略、inventory 回归、文档验收、parity 状态、governance integrity gate 与 Runtime
验证。台账保留 `previousBatch`、`previousClassification`、`sourceChangedSincePrevious` 溯源信息，同时
7 个当前记录不再是 `deferred-next-batch`。
