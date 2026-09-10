---
author: AI Cockpit maintainers
title: "WI-778——WI-776 documentation promotion"
description: "将已关闭的 WI-776 文档投影提升为有终态 evidence 绑定的状态。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization-for-documentation-promotion
workItemId: WI-778-wi776-doc-promotion
lastVerifiedBy: WI-778-wi776-doc-promotion
---

[English](WI-778-wi776-doc-promotion.md) · [日本語](WI-778-wi776-doc-promotion.ja.md)

# WI-778——WI-776 documentation promotion

## 意图与边界

WI-778 是 WI-776 关闭后的有界文档 Work Item。它把不可变的 WI-776 Contract、verification、
finalization 和 close 记录投影到英文、简体中文、日文 Work Item 页面及 reference-parity 表。

不修改 Runtime 源码、产品行为、治理规则、性能实现、发布状态，或 WI-774/WI-775/WI-776 的不可变记录。

## 范围

- 根据不可变终态 evidence 提升六个 WI-776 文档与 parity 投影。
- 在验证前登记本 Work Item 的三语页面。
- 保持 promotion helper 与文档 acceptance 检查可复现。

## 验证

声明的 workspace 验证为 `cargo test --locked --workspace`。
文档专用 promotion 检查为：

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-776-wi775-archive-evidence-recovery --check`
