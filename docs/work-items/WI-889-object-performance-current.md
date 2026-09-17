---
author: AI Cockpit maintainers
workItemId: WI-889-object-performance-current
title: Current object-repository performance evidence
description: Paired current-version Runtime and development-cycle measurements on selected object repositories.
audience: [adopter, contributor, maintainer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-889-object-performance-current
---

# WI-889 — Current object-repository performance evidence

This Work Item measures the current Runtime against a public baseline on the
same machine, toolchain, build mode, and scenario definitions. The selected
object repositories are observed through temporary isolated views; their main
branches and pre-existing working-tree state remain out of scope.

## Acceptance boundary

- Release-grade percentile comparison requires at least 100 valid warm samples;
  99 remains diagnostic-only and must be rejected by the gate.
- Runtime latency and development-cycle cost are reported separately with raw
  samples, p50/p95/p99, environment identity, operation counters, reuse counts,
  and explicit invalidation or unavailable reasons.
- Goods-garden, sentinel, and ai-investigation-orchestrator are measured only
  through temporary isolated views, which are cleaned after collection.
- The three-language Work Item projection and reference-parity rows remain
  synchronized before archive. Release publication is outside this Work Item.

## Verification plan

Use the existing performance harness with `CARGO_INCREMENTAL=0` and the shared
verification target directory. Run focused format, performance, and document
projection checks before any expensive workspace verification. Preserve the
raw evidence and report unknown metrics rather than substituting zero.
