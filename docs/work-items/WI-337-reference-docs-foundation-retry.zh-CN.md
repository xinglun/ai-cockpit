---
author: AI Cockpit maintainers
title: "WI-337——治理文档基础重试"
workItemId: WI-337-reference-docs-foundation-retry
description: "在保留 WI-336 历史的前提下，通过干净 successor lifecycle 重新交付前五个固定参考源治理文档的比较。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-337-reference-docs-foundation-retry
---

# WI-337——治理文档基础重试

## 意图与恢复边界

WI-337 是 WI-336 的显式 successor。由于 Runtime 0.2.33 无法调和其验证前的
Contract amendment chain，前项保持不可变历史。本次重试仍使用同五个固定路径，不增加实现范围。

## 复用的比较结果

文件级分类、Rust 对应物与非 wire 边界记录在 [WI-336 比较](WI-336-reference-docs-foundation.zh-CN.md)
及三语台账中。WI-337 只针对当前仓库与已审阅 PR 上下文重新验证这些 bytes，不复制源 Python、Make、provider 或历史工具。

## 验收与验证

1. 五个固定路径保留唯一且明确的台账分类、对应物和不夸大的理由。
2. English、简体中文与日本語 comparison/parity 台账一致。
3. 在验证前绑定 GitHub resource context，当前 Runtime evidence 绑定仓库与快照。
4. Inventory、文档、parity 与 locked workspace 验证通过。

前项恢复记录：`.ai/decisions/WI-336-reference-docs-foundation.recovery.e7ccd6381b1492fd0ba72be8c7305217748f03d9c7509a7c58db693e8ba13261.json`。

[English](WI-337-reference-docs-foundation-retry.md) ·
[日本語](WI-337-reference-docs-foundation-retry.ja.md)
