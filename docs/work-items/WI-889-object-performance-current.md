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

## Current paired result

The baseline (`0.2.93`) and candidate (`0.2.95`) were built with Rust/Cargo
`1.98.1` on `aarch64-apple-darwin`, using the same isolated object-repository
views and seven measured scenarios. Every compared operation has 100 valid warm
samples. The comparator reports p50, p95, and p99; the 5 ms noise decision is
kept on the established p50/p95 budget metrics, while p99 is retained as a
tail diagnostic. All 38 comparisons are `within_noise`; this is a valid
comparison but does not prove an improvement.

The measured views were goods-garden (current repository and file-change
scenarios), sentinel (many-files-clean), and
ai-investigation-orchestrator (many-historical-wi). A direct small-clean
object view was attempted but rejected by the harness because each supplied
object repository exceeded its `<=100` tracked-file threshold; it is reported
as unavailable rather than replaced with a synthetic repository. The raw
collector output, including process/resource counters and invalidation reasons,
is retained in [the compressed capture archive](../../.ai/evidence/WI-889-object-performance-current/raw/runtime-captures.tar.gz)
with [checksums](../../.ai/evidence/WI-889-object-performance-current/raw/SHA256SUMS).

Diagnostics overhead is a separate paired measurement. On the current
repository, diagnostics-on versus diagnostics-off added 5.134 ms p50 and
5.255 ms p95 to `verification-plan`, 0 ms p50 and 1.183 ms p95 to `status`,
and 0.674 ms p50 and 1.689 ms p95 to `work-item-outcome`.

Runtime latency and development-cycle cost are intentionally separate. The
The first Contract→reviewable-PR interval is now measured as 4,744,000 ms
(Contract creation `2026-09-17T19:23:54Z` to PR #865 creation
`2026-09-17T20:42:58Z`). The Runtime did not persist an agent-operation count
for this interval, so that dimension remains unavailable; the captured
preflight-rejection count is 0. Verification→finish and post-merge-cleanup
remain explicitly unavailable until their lifecycle timestamps exist in the
development-cycle report.
