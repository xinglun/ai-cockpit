---
author: AI Cockpit maintainers
title: ガバナンス cost metrics
description: 一つの repository-bound Work Item の実測実行コストだけを報告する境界。
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# ガバナンス cost metrics

Rust Runtime は measured cost を advisory telemetry として提供します。repository を明示し、必要なら Work Item を指定します。

```sh
ai-cockpit diagnose --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

JSON result は repository identity に bind され、snapshot の Git call、read/hash file 数、verification run、executed/reused node、elapsed time、bottleneck hint、evidence reference、unknown を報告します。Verification receipt には typed `VerificationCostEstimate` と `VerificationCostObservation` も含まれ、planned node、resource unit、parallelism、process count、execution time を記録します。

## Advisory boundary

Cost は authority ではありません。estimate/observation は `VerificationTier`、`EvidenceAssurance`、policy requirement、protected node、scope、最終 Outcome を変更しません。worker/resource budget の不明、identity 欠落、invalid cache observation は `unknown`/`partial` のままで、green governance result にはなりません。Physical execution reuse と Work Item ごとの identity-bound evidence receipt は分離されます。

Reference の JSONL phase/wait parser と source report wire shape は Rust protocol requirement ではありません。repository が提供しない provider wait、human wait、token usage、P95、lifecycle category を Runtime は生成しません。

## Object project への継承

すべての adopter repository が同じ command と advisory boundary を使います。Runtime は共有されますが、snapshot、Work Item、evidence、cost fact は request-scoped、repository-local です。ある repository の cost report が別 repository/Work Item の変更を認可することはありません。

