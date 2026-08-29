---
author: AI Cockpit maintainers
title: Work Item Intelligence 性能基线边界
description: 如何记录本地 Intelligence 性能而不把它变成治理权限。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: supporting
canonical: docs/reference/work-item-intelligence-performance-baseline.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Work Item Intelligence 性能基线边界

[English](work-item-intelligence-performance-baseline.md) · [简体中文](work-item-intelligence-performance-baseline.zh-CN.md) · [日本語](work-item-intelligence-performance-baseline.ja.md)

性能测量是可复现的本地观测，不是预算、SLO、assurance 声明或削弱 Verification 的权限。
应使用隔离临时 fixture，并记录 Runtime、repository、profile、toolchain、输入和文件系统
identity。

## 测量建议

改变 Work Item 数、fact 数、reader 并发度及 cold/warm 状态，记录样本数、p50/p95/p99
延迟、超时、锁/资源等待和 fixture 字节数。若目标是读取性能，cold/warm 必须查询同一
个显式构建的投影；重建成本单独观测。

Rust `diagnose` 和 cost-observation 路由只报告受界定的执行、复用、worker 与耗时事实，
不声称参考源 Python benchmark 数字、provider/human wait 或通用吞吐。未来性能 Work Item
必须比较同等 identity，并将生成报告与 evidence 一起保存。
