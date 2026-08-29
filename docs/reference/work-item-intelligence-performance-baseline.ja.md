---
author: AI Cockpit maintainers
title: Work Item Intelligence performance baseline の境界
description: local performance observation を governance authority にしない測定境界。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: supporting
canonical: docs/reference/work-item-intelligence-performance-baseline.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Work Item Intelligence performance baseline の境界

[English](work-item-intelligence-performance-baseline.md) · [简体中文](work-item-intelligence-performance-baseline.zh-CN.md) · [日本語](work-item-intelligence-performance-baseline.ja.md)

性能測定は再現可能な local observation であり、budget、SLO、assurance claim、Verification を
弱める権限ではありません。isolated temporary fixture を使い、Runtime、repository、profile、
toolchain、input、filesystem identity をレポートに記録します。

## 測定方法

Work Item 数、fact 数、reader concurrency、cold/warm 状態を変え、sample 数、p50/p95/p99 latency、
timeout、lock/resource wait、fixture bytes を記録します。read performance が目的なら cold/warm
とも同じ明示的に構築した projection を読み、rebuild cost は別の観測にします。

Rust の `diagnose` と cost-observation route は bounded な execution、reuse、worker、timing の
事実だけを報告します。reference Python benchmark の数値、provider/human wait、一般的 throughput
は主張しません。将来の性能 Work Item は同一条件の identity を比較し、生成レポートを evidence
と共に保存します。
