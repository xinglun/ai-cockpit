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

## 动态验证选择

检测到的 Work Item 命令与独立的自动检测验证使用同一 profile-authorized reuse 路径。只有 repository、snapshot、profile、Runtime、command、scope、stage、runner、base、toolchain、dependency 和 policy identity 全部匹配时，规划器才会复用结果。任何不匹配或影响未知都会执行声明的命令并记录原因，不会静默扩大复用范围或降低必需检查。显式自定义命令保持 fresh；未来若要复用，必须由操作者明确声明新的自定义命令复用 Contract。

## 对象工程继承

adopter repository 可以使用相同的 identity-bound fixture 和 regression gate，但使用各自 repository 与 Runtime identity。共享 Runtime 不保存全局 budget 或 current project，一个 repository 的耗时不能授权另一个 repository 的 Work Item。

Runtime 升级后，adopter repository 也继承同一动态规则：冷验证建立 receipt，未变化的热重复只能在该 adopter 自身 repository context 内复用。adopter acceptance receipt 必须记录冷/热耗时、执行/复用节点、选择原因、Runtime identity 和 repository identity。
