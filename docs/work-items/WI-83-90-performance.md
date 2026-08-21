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

# WI-83–WI-90: Performance and runtime efficiency

This slice improves repeated local work without changing governance authority.
Performance measurements are evidence only when they carry Runtime identity,
repository identity, capture time, samples, and explicit budgets. The portable
`tests/performance/regression_gate.sh` consumes two captured JSON records and
fails closed on missing fields, identity mismatch, malformed samples, and
budget regressions. It never builds a source fallback.

The repository layer exposes a request-scoped `RepositoryExecutionContext`.
It captures one immutable Git snapshot and memoizes the derived observation;
`RuntimeSession` can retain explicitly bound contexts for repeated requests,
but has no global current repository and requires an explicit path for bind,
refresh, and unbind. Two repositories therefore retain separate identities and
snapshots.

Git content identity is an incremental Merkle cache over declared relative
files. Unchanged metadata reuses a digest, content changes invalidate one
entry, deleted files are removed, and absolute or escaping paths fail closed.
Verification keeps its existing dependency DAG, protected-node execution, and
receipt binding while adding resource weights and an explicit resource budget.
Zero or over-budget commands are rejected before any process starts.

`SingleFlightCoordinator` coalesces concurrent requests only when repository,
Work Item, command, and Runtime identities all match. It is an ephemeral
optimization; the returned receipt still follows the ordinary evidence store
and is never treated as authority by the coordinator. Knowledge indexes record
the digest of archived source inputs and are rebuilt when those inputs change;
the index remains a noncanonical cache.

Focused evidence:

```text
cargo test -p cockpit-verification --test execution --test graph
cargo test -p cockpit-git --test snapshot
cargo test -p cockpit-repository --test repository_context --test knowledge_cache
tests/performance/regression_gate_test.sh
```

The timings remain platform-specific. A passing local gate is not hosted CI or
release evidence until the captured artifact is bound to the target release.
