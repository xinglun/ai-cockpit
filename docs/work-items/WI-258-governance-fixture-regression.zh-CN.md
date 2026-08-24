---
author: AI Cockpit maintainers
title: "WI-258——治理 fixture registry 回归"
workItemId: WI-258-governance-fixture-regression
description: "在引入 pending parity 校验后，确保治理 fixture 始终生成完整的 registry。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-258-governance-fixture-regression
authority: canonical
---

# WI-258——治理 fixture registry 回归

## 意图

让每个 governance-integrity fixture 都明确生成空的 pending parity
registry。fixture builder 不应因为遗漏 repository-owned control file 而让
测试失败。

## 范围

本 Work Item 只修改 fixture builder、对应回归测试和三语 Work Item/parity
投影，不改变 Runtime validator 或生产治理语义。

## 验收

- 每个生成的 fixture 都有 regular 的
  `docs/reference/pending-parity-registry.json`，内容为
  `schemaVersion: 1`、`entries: []`；只有明确测试 pending registration 时才
  写入条目。
- governance-integrity 和 pending-registry 的正常/对抗测试均通过，报告可
  重复生成且字节稳定。
- 实现及其证据在审阅后绑定归档 Contract、verification、finalization 和
  close 记录。

## 证据边界

空 registry 只是 fixture 基线，不表示真实 Work Item 正在 pending。测试
pending registration 时必须显式写入条目，并继续校验其 identity、parity rows
和 lifecycle。

## 恢复边界

WI-258 保持 immutable 历史交付。其 Runtime close 已确认，但 human decision
是描述性文本，而不是文档 promotion gate 要求的规范 `approved` 值。所有原始
记录均保留；有界 successor [WI-259](WI-259-close-decision-recovery.zh-CN.md)
将该 predecessor 投影为 Recovered，不改写 WI-258 的任何 `.ai` 字节。
