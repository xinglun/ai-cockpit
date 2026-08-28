---
author: AI Cockpit maintainers
title: 治理性能预算
description: 不削弱必需验证的 identity-bound 本地性能测量。
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# 治理性能预算

性能测量是本地工程证据，不是省略必需检查的许可，也不是 hosted provider evidence。Rust verification crate 提供 typed `PerformanceBaseline`、`PerformanceSample`、`PerformanceBudget` 和 `PerformanceAssessment`。baseline 必须包含 Runtime version/digest、repository identity、采集时间、samples 和明确的最大耗时预算。

便携 regression gate 消费 baseline 与 candidate JSON：

```sh
tests/performance/regression_gate.sh baseline.json candidate.json
```

它会拒绝缺失或零迭代 sample、无效 identity 和预算回归；不会构建 source fallback，也不会改变 Contract 的 required verification graph。verification command 的 resource weight 与显式 resource budget 同样在执行前 fail-closed。

## 测量边界

参考源的 profile P95 报告不是 Rust Runtime authority。目标不会在样本不足时推导已建立的预算，不会根据耗时自动决定 governance profile，也不会把本地耗时冒充 provider/enterprise assurance。缺失、过期或 identity 不匹配的测量保持 unknown 或 fail-closed。

性能和治理强度是两个维度。`VerificationTier` 与 `EvidenceAssurance` 不由耗时、缓存命中、worker 数或预算结果推导。即使报告指出超预算瓶颈，protected 和 policy-required nodes 仍然必须执行。

## 对象工程继承

adopter repository 可以使用相同的 identity-bound fixture 和 regression gate，但使用各自 repository 与 Runtime identity。共享 Runtime 不保存全局 budget 或 current project，一个 repository 的耗时不能授权另一个 repository 的 Work Item。

