---
author: AI Cockpit maintainers
title: 治理成本指标
description: 针对单个 repository-bound Work Item 的仅事实执行成本报告。
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# 治理成本指标

Rust Runtime 将测量成本作为 advisory telemetry。检查一个 repository，也可以选择一个 Work Item：

```sh
ai-cockpit diagnose --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

JSON 结果绑定 repository identity，报告 snapshot 的 Git 调用、读取/哈希文件数、verification 次数、执行与复用节点、耗时、瓶颈提示、evidence 引用和显式 unknown。Verification receipt 还提供 typed `VerificationCostEstimate` 与 `VerificationCostObservation`，包括计划节点、resource units、并行度、进程数和执行耗时。

## Advisory 边界

成本不是权威。estimate 或 observation 不会改变 `VerificationTier`、`EvidenceAssurance`、policy 要求、protected nodes、scope 或最终 Outcome。未知 worker/resource budget、缺失 identity 或无效缓存观察保持为 `unknown`/`partial`，不会变成绿色治理结果。物理执行复用与每个 Work Item 自己的 identity-bound evidence receipt 彼此分离。

参考源的 JSONL 阶段/等待解析器和报告 wire shape 不是 Rust 协议要求。若 repository 没有提供，Runtime 不会编造 provider wait、human wait、token usage、P95 或生命周期分类。

## 对象工程继承

所有 adopter repository 使用同一命令和 advisory 边界。Runtime 共享，但 snapshot、Work Item、evidence 和成本事实按 request-scoped、repository-local 隔离。一个成本报告不能授权另一个 repository 或另一个 Work Item 的变更。

