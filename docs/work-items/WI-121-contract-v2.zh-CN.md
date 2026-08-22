---
author: AI Cockpit maintainers
workItemId: WI-121-contract-v2
title: Contract V2 语义、严格校验与 fail-closed 事前确认
description: 增加结构化 Contract V2 语义、严格解析和 fail-closed 事前确认。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-121-contract-v2
---

# WI-121 — Contract V2 语义与 fail-closed 评审

## 目的

在不复制参考源 Runtime 的前提下，对齐 Rust Contract 边界。Contract 必须
保留明确的意图、范围、权限、证据和人工决定；未知或格式错误的治理输入
必须在实现前停止。

## 范围

- 增量兼容的 typed Contract V2 字段；
- 结构化 intent、sources、verification、能力和执行声明；
- 未知字段、重复键、schema 和跨字段严格校验；
- 结构化 preflight 人工决定请求与 repository 绑定的评审 receipt；
- fail-closed checkpoint 与生命周期状态转移校验；
- 三语 CLI/MCP 机器输出和面向人的投影。

场景/最终维度聚合属于 WI-122；Contract 并行插槽和串行投影 lease 属于
WI-123。Contract 原文不得被机器翻译，历史 bytes 不得重写。

## 验证

执行协议、preflight、生命周期和投影回归测试，以及锁定的 Rust workspace
质量门。最终面向人的 Outcome 必须保留交通灯、未知项、证据、决定和下一步。
