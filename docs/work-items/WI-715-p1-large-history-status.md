---
author: AI Cockpit maintainers
title: "WI-715 — P1 large-history status candidate decision"
description: "Measure a request-scoped large-history status deduplication candidate and retain an evidence-backed decline when the acceptance contract is not met."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-715-p1-large-history-status
lastVerifiedBy: WI-715-p1-large-history-status
---

[简体中文](WI-715-p1-large-history-status.zh-CN.md) · [日本語](WI-715-p1-large-history-status.ja.md)

# WI-715 — P1 large-history status candidate decision

## Intent

Test whether reusing a validated close receipt within one immutable `status`
observation removes repeated read/parse work on the measured
`many-historical-wi` path, while preserving Calibrated Human-Agent Trust,
evidence validity, repository isolation, authorization, and recovery behavior.

## Boundary and acceptance contract

The Work Item is limited to the request-scoped status observation, its focused
test, external benchmark evidence, and tri-language records. It does not add a
persistent cache, cross-repository or cross-Work-Item reuse, coordinator,
IncrementalMerkle reuse, large-file streaming, wait replacement, or P3
architecture change. The candidate was pre-registered with a required warm
status improvement of at least 5% at both p50 and p95; all inherited
non-target budgets and behavior checks also had to pass.

## Measurement and result

The paired development-binary runs used 20 warm samples and one OS-cache
warmup. The first measured call followed identity probes and is not called true
cold cache. p99, resident MCP, phase timing, internal Git/I/O counters, and
peak memory were unavailable where the harness or platform could not prove
them; they are recorded as unavailable rather than zero. The host could not
provide a trustworthy filesystem comparison key, so the regression gate failed
closed.

| Order | Baseline status warm p50/p95 | Candidate status warm p50/p95 | Candidate delta | Decision |
| --- | ---: | ---: | ---: | --- |
| baseline → candidate | 3483.095 / 4681.167 ms | 3334.552 / 3478.871 ms | -4.26% / -25.68% | threshold not met; gate failed closed |
| candidate → baseline | 3315.139 / 4260.963 ms | 3435.055 / 4453.557 ms | +3.62% / +4.52% | threshold not met; gate failed closed |

An additional unbudgeted run was adverse to the candidate (+1.35% p50,
+9.28% p95) and is retained only as supporting evidence. The candidate code and
test were removed; no production performance change was merged.

## Correctness and evidence

Baseline and candidate outputs matched on clean, changed, and malformed-close
fixtures across `inspect`, `status`, `doctor`, and `observe`: 12 comparisons,
zero exit-code mismatches, and zero normalized-output mismatches after removing
only runtime-digest fields. The raw and derived records are under
`.ai/evidence/external/WI-715-p1-large-history-status.*`; the gate result and
experiment summary are the authoritative decision records.

## Decision and follow-up

Decision: `declined`. This Work Item does not claim a latency, CPU, I/O, memory,
or resident-MCP benefit. Do not retry this micro-optimization as an accepted
change until a trustworthy environment comparator and stronger phase-level
measurement identify a bottleneck that justifies the complexity. The next
optimization Work Item must start from the latest reviewed default branch and
retain this decline as immutable evidence.
