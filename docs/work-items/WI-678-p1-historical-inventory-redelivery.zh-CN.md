---
author: AI Cockpit maintainers
title: "WI-678——P1 历史 finalization inventory 重新交付"
description: "从最新默认分支重新验证有界的历史 finalization candidate 复用。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-678-p1-historical-inventory-redelivery
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-678-p1-historical-inventory-redelivery
---

[English](WI-678-p1-historical-inventory-redelivery.md) · [日本語](WI-678-p1-historical-inventory-redelivery.ja.md)

# WI-678——P1 历史 finalization inventory 重新交付

## 意图与假设

P0 测量显示历史 finalization inventory 是 status 路径的主要瓶颈。假设是：在
一次不可变 observation 内只读取一次 decisions 目录并复用 transition candidate
列表，可以降低重复 I/O，同时不改变治理语义。

本 Work Item 从 `origin/main` 的
`c1f1f1d9f17ec2242a59f287a4368bd6bda5993e` 重新交付，因为 draft PR #671 未通过
hosted quality gate。PR #671 仅保留为审计上下文，不是 review 或 merge 授权。

## 边界

范围包括历史 inventory/resolver 路径、有界计数测试、三语文档和三个
reference-parity 行。candidate 数据只在同一次不可变 observation 内复用。不增加
进程全局缓存、跨请求 snapshot、授权变化、evidence 重新绑定或无关重构。

## 授权记录

2026-09-08，`human:user-request` 授权本 Agent 在 provider 或生命周期中断后继续，
包括用户请求的暂停/恢复边界。Contract 保存准确范围和证据；这不是伪造 GitHub
review 或 approval。

## 验收与验证

- 使用相同 toolchain、场景、环境和修正后的保序 benchmark 语义，分别取得新的
  baseline 与 candidate 原始样本。
- 保留预热次数、样本数、分位数方法、环境和不可用指标；至少 20 个 warm 样本才
  允许 p95，少于 100 个样本时 p99 保持 unknown。
- 证明 decisions 目录扫描复用有界，并保持 finalization、malformed record、文件名
  digest、fork、目录缺失、隔离、错误、证据绑定、输出顺序和退出码行为。
- 执行 Runtime 生命周期和文档/parity 检查。只有新的目标路径证据覆盖 candidate 且
  correctness 等价时才接受性能收益，否则记录拒绝，不宣称收益。

## 证据与状态

- 前置审计：PR #671，`https://github.com/xinglun/ai-cockpit/pull/671`
- 基线：`origin/main` 的 `c1f1f1d9f17ec2242a59f287a4368bd6bda5993e`
- Runtime 证据：`.ai/evidence/WI-678-p1-historical-inventory-redelivery.verification.json`
- 达到终态后，记录由 Runtime 生成于 `.ai/work-items/archive` 与 `.ai/decisions`

在新的测量、正确性检查、hosted review 和终态 Runtime Outcome 全部绑定前，
Work Item 保持 `in_progress`。
