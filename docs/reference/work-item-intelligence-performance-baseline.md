---
author: AI Cockpit maintainers
title: Work Item Intelligence performance baseline boundary
description: How local intelligence performance observations are collected without becoming governance authority.
audience: [adopter, maintainer, reviewer]
status: implemented
authority: supporting
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Work Item Intelligence performance baseline boundary

[简体中文](work-item-intelligence-performance-baseline.zh-CN.md) · [日本語](work-item-intelligence-performance-baseline.ja.md)

Performance measurements are reproducible local observations, not a budget,
SLO, assurance claim, or permission to weaken verification. Use an isolated
temporary fixture and record the Runtime, repository, profile, toolchain,
input, and filesystem identities with the report.

## Measurement guidance

Vary Work Item count, fact count, reader concurrency, and cold/warm reader
state. Record sample count, p50/p95/p99 latency, timeouts, lock/resource wait,
and fixture bytes. Cold and warm results must query the same explicitly built
projection when the goal is read performance; projection rebuild cost is a
separate observation.

The Rust `diagnose` and cost-observation routes report bounded execution,
reuse, worker, and timing facts. They do not claim the reference Python
benchmark numbers, provider wait, human wait, or universal throughput. A
future performance Work Item must compare like-for-like identities and retain
the generated report with its evidence.
