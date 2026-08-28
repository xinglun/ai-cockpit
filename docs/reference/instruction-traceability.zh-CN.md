---
author: AI Cockpit 维护者
title: 指令可追溯性
description: 从比对指令到 Work Item 和验证的证据绑定关系。
audience:
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/instruction-traceability.md
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - comparison_traceability
---

# 指令可追溯性

[English](instruction-traceability.md) · [简体中文](instruction-traceability.zh-CN.md) · [日本語](instruction-traceability.ja.md)

逐文件比对由机器可读清单 [`tests/conformance/reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json) 治理。每个 pinned source path 恰好有一个分类、有限 counterpart 决定和原因；comparison/parity 页面面向人解释，inventory 是防遗漏检查。

## 正向与反向检查

每个批次的正向链路是：

```text
固定的 source path
  → Work Item Contract
  → target counterpart 或明确边界
  → acceptance 与 verification evidence
  → reviewed PR、merge、close receipt
```

反向检查确认每个列出的 Work Item 都有 source 集合、证据和交付 counterpart，或记录 no-change/reference-only 原因。归档 Work Item 是交付历史真相，不能由未跟踪笔记悄悄替代。Hosted performance（若有）只能使用明确的 `pass`、`not_run` 或 `fail` 及原因。

Inventory 脚本是结构门：它证明覆盖和稳定身份，不证明自然语言声明为真。新的语义责任需要独立的有限 Contract 和证据，不能隐藏到无关 Work Item。

## 不复制与对象工程边界

Rust 项目不把参考 remediation JSON、Make 命令或 Python checker 当作 Runtime authority。对象工程可以继承同样的 inventory 和显式仓库生命周期，但自己的 source path、Work Item、证据和 provider 回执保持独立。
