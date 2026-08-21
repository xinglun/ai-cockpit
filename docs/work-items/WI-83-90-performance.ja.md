---
author: AI Cockpit maintainers
title: "WI-83–WI-90 Performance and Runtime Efficiency"
description: "Identity-bound performance evidence, bounded scheduling, repository context reuse, and noncanonical caches."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: performance-focused-tests
capabilityClaims:
  - performance_baseline
  - repository_context_isolation
  - resource_aware_verification
  - single_flight_execution
  - incremental_knowledge_cache
---

# WI-83–WI-90：Performance と Runtime 効率

このスライスは governance authority を変えずに、繰り返し行う local operation を効率化します。
性能測定は Runtime identity、repository identity、取得時刻、sample、明示的な budget を持つ場合だけ
evidence です。`tests/performance/regression_gate.sh` は 2 つの JSON を読み、必須 field 欠落、identity
不一致、壊れた sample、budget regression を fail-closed で拒否します。source fallback は build しません。

Repository layer の `RepositoryExecutionContext` は request-scoped で、1 つの immutable Git snapshot と
派生 observation を memoize します。`RuntimeSession` は明示的に bind した context を保持できますが、
global current repository は持たず、bind、refresh、unbind には常に path が必要です。A/B repository の
identity と snapshot は分離されます。

Git content identity は宣言された相対 file の incremental Merkle cache です。変更されていない metadata
では digest を reuse し、content 変更では対象 entry を invalidate し、削除 file は除去します。absolute または
repository 外の path は fail-closed です。Verification は dependency DAG、protected node 実行、receipt
binding を維持したまま、resource weight と明示的な resource budget をサポートします。zero または budget
超過の command は process 起動前に拒否されます。

`SingleFlightCoordinator` は repository、Work Item、command、Runtime identity がすべて一致する場合だけ
concurrent request を coalesce します。一時的な最適化であり、receipt は通常の evidence store を通し、
coordinator 自体は authority になりません。Knowledge index は archive source digest を記録し、source
変更時に再生成します。index は noncanonical cache のままです。

Focused evidence：

```text
cargo test -p cockpit-verification --test execution --test graph
cargo test -p cockpit-git --test snapshot
cargo test -p cockpit-repository --test repository_context --test knowledge_cache
tests/performance/regression_gate_test.sh
```

local timing は platform-specific です。target Release に artifact を bind して初めて release evidence になります。
