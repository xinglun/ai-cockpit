---
author: AI Cockpit maintainers
title: ガバナンス performance budget
description: 必須 verification を弱めない identity-bound な local performance measurement。
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# ガバナンス performance budget

Performance measurement は local engineering evidence であり、必須 check を省略する許可でも hosted provider evidence でもありません。Rust verification crate は typed `PerformanceBaseline`、`PerformanceSample`、`PerformanceBudget`、`PerformanceAssessment` を提供します。baseline には Runtime version/digest、repository identity、capture time、sample、明示的な最大 elapsed-time budget が必要です。

Portable regression gate は baseline と candidate の JSON を読みます。

```sh
tests/performance/regression_gate.sh baseline.json candidate.json
```

missing/zero-iteration sample、invalid identity、budget regression を拒否します。source fallback を build せず、Contract の required verification graph も変更しません。Verification command の resource weight と明示的な resource budget も execution 前に fail-closed です。

## Measurement boundary

Reference の profile P95 report は Rust Runtime authority ではありません。sample が足りないのに established budget を推測せず、timing から governance profile を自動決定せず、local timing を provider/enterprise assurance として表示しません。欠落・stale・identity mismatch は unknown または fail-closed です。

Performance と governance strength は別の次元です。`VerificationTier` と `EvidenceAssurance` は elapsed time、cache hit、worker 数、budget result から導出されません。over-budget report が bottleneck を示しても、protected node と policy-required node は必須のままです。

## Object project への継承

Adopter repository は同じ identity-bound fixture と regression gate を使えますが、repository/Runtime identity はそれぞれ固有です。Shared Runtime は global budget/current project を保存せず、ある repository の timing が別 repository の Work Item を認可することもありません。

