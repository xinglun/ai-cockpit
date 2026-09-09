---
author: AI Cockpit maintainers
title: "WI-763 — WI-752 P1 fact-reuse redelivery"
description: "Revalidate P1 fact reuse from the latest main with raw measurements and a fail-closed optimization decision."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human:repository-owner
workItemId: WI-763-wi752-fact-reuse-redelivery
lastVerifiedBy: WI-763-wi752-fact-reuse-redelivery
terminalArchive: .ai/work-items/archive/WI-763-wi752-fact-reuse-redelivery.contract.json
terminalVerification: .ai/evidence/WI-763-wi752-fact-reuse-redelivery.verification.json
terminalFinalization: .ai/decisions/WI-763-wi752-fact-reuse-redelivery.finalize.2a7bdf19a5543e89aaf37a5df5118da6f793735afbdce643093f9b69bbd489ab.json
terminalDecision: .ai/decisions/WI-763-wi752-fact-reuse-redelivery.close.json
---

# WI-763 — WI-752 P1 fact-reuse redelivery

## Contract and boundary

This Work Item is a fresh measurement-only redelivery from the latest remote
`main` (`a8fa804057cad8792f56b580f665046d2d3fc52d`). It exists because the
immutable WI-752 PR #733 delivery was rejected by hosted quality with
`docs_governance_integrity: invalid_premerge_finalize`; that PR, branch,
worktree, and evidence remain historical and are not rewritten or deleted.

The scope is limited to raw measurement evidence, this tri-language report,
and the three reference-parity projections. Production Runtime behavior,
IncrementalMerkle, release publication, and the historical WI-752 resources
are out of scope.

## Measurement and decision

Two identical-runtime retests used `tests/performance/runtime_benchmark.sh`
with one warmup and eight warm samples for `inspect`, `status`, `doctor`,
`observe`, `work-item status`, and `diagnose`. The first sample is retained as
the first independent CLI process after identity probes, not a true cold-cache
claim. Warm p95/p99 are unavailable because 8 samples are below the declared
reliability thresholds.

The raw fixture is
`tests/performance/fixtures/WI-763-fact-reuse-measurement.json`. Both runs use
Runtime `0.2.87`, the same Runtime digest, repository head, machine, and
unavailable filesystem comparison key. Warm p50 changed by approximately
`+0.4%` for `status`, `+2.1%` for `observe`, `+2.2%` for `inspect`, `-0.2%`
for `doctor`, `+0.5%` for Work Item status, and `+0.3%` for diagnose. These
same-runtime retests do not establish a candidate optimization or a reliable
benefit; no production optimization is accepted and the candidate is declined.

Runtime-reported internal read bytes, hash bytes, Git call counts, child
processes, peak memory, and cache invalidation events are explicitly marked
unavailable rather than filled with zero. Resident MCP and concurrency are not
measured by this portable harness.

## Correctness and validity

`cargo test -p cockpit-repository --test repository_context -- --nocapture`
passed 7/7. The checks cover one-snapshot memoization, repository isolation,
explicit RuntimeSession binding, fail-closed invalidation after source or
configuration changes, fresh phase boundaries, and governance consumption of
validated observation context. Repeated `status` and `doctor` output was
identical across independent invocations.

The pre-archive parity row is intentionally registered as
`In progress → Implemented after verified close` with terminal paths before
fresh verification evidence. After close, the Runtime promotion check may
project this row and the three language pages to their terminal status.

## Terminal evidence

Expected terminal bindings are archive
`.ai/work-items/archive/WI-763-wi752-fact-reuse-redelivery.contract.json`,
verification
`.ai/evidence/WI-763-wi752-fact-reuse-redelivery.verification.json`,
finalization
`.ai/decisions/WI-763-wi752-fact-reuse-redelivery.finalize.json`, and close
`.ai/decisions/WI-763-wi752-fact-reuse-redelivery.close.json`.
