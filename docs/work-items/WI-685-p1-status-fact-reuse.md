---
author: AI Cockpit maintainers
title: "WI-685 — P1 status fact reuse"
description: "Reuse immutable finalization transition facts within one status observation on repositories with many historical Work Items."
audience: [maintainer, reviewer, adopter]
workItemId: WI-685-p1-status-fact-reuse
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-685-p1-status-fact-reuse
terminalArchive: .ai/work-items/archive/WI-685-p1-status-fact-reuse.contract.json
terminalVerification: .ai/evidence/WI-685-p1-status-fact-reuse.verification.json
terminalFinalization: .ai/decisions/WI-685-p1-status-fact-reuse.finalize.json
terminalDecision: .ai/decisions/WI-685-p1-status-fact-reuse.close.json
---

[简体中文](WI-685-p1-status-fact-reuse.zh-CN.md) · [日本語](WI-685-p1-status-fact-reuse.ja.md)

# WI-685 — P1 status fact reuse

## Intent

Reduce repeated finalization-transition directory scans and parsing on the
`status` path while preserving Calibrated Human-Agent Trust, repository and
Work Item isolation, evidence binding, authorization boundaries, and recovery.

## Boundary

The candidate is request-scoped only. `status_with_runtime` builds one immutable,
repository-bound `FinalizationTransitionIndex` for the observation and passes it
to historical finalization projection. It stores each transition's path,
content digest, parsed value, and fail-closed parse/digest error. It does not
introduce a cross-request cache, alter the snapshot boundary, mutate evidence,
or change the Runtime, MCP, doctor, scheduler, IncrementalMerkle, large-file,
parallel-read, or P3 paths.

The shared resolver reads the canonical receipt before consuming indexed
candidates. This preserves the existing canonical error precedence; malformed,
missing, forked, stale, digest-mismatched, symlinked, and path-anomalous inputs
remain fail-closed.

## Hypothesis and bottleneck evidence

The many-history repository contains 395 canonical finalization receipts and
156 transition files. Before this change, the historical inventory scanned the
decisions directory once per old-runtime canonical receipt, yielding 396
directory enumerations per status observation. The candidate enumerates it twice:
once for the canonical inventory and once to build the transition index. The
candidate path remains immutable and bounded to the current repository and
observation.

The source-path count evidence is recorded in
`.ai/evidence/external/WI-685-p1-status-fact-reuse.observation-counts.json`.

## Measurement contract and result

Contract thresholds were amended and revalidated before acceptance: at least
20 warm independent CLI samples; `status` p50 improvement of at least 10% and
p95 improvement of at least 5%; non-target p95 regression below both 25% and
20ms. The benchmark records first measurement, one OS-cache warmup, warm sample
count, percentile reliability, environment, raw samples, and unavailable
metrics. It does not claim true cold-cache behavior or resident MCP behavior.

| Paired run | status baseline p50/p95 ms | status candidate p50/p95 ms | delta |
| --- | ---: | ---: | ---: |
| baseline → candidate | 2058.544 / 2165.179 | 1565.100 / 1596.771 | -23.98% / -26.25% |
| candidate → baseline | 2051.134 / 2104.867 | 1559.946 / 1653.997 | -23.94% / -21.43% |

In these same-window runs, inspect, doctor, and observe stayed within the
non-target budget. A separate round40 candidate measurement became much slower
while unrelated paths also slowed; because the harness cannot expose host load,
that temporally unpaired pre-box candidate run is retained as adverse evidence
and not used as an acceptance claim. It is not deleted or replaced.

Raw evidence and the derived summary are under
`.ai/evidence/external/WI-685-p1-status-fact-reuse.*.json`, especially
`measurement-summary.json`.

## Correctness and isolation

- The focused transition suite passed 29 tests and the status projection suite
  passed 11 tests.
- A TDD regression test first failed for canonical-error precedence, then passed
  after the resolver was refactored to observe the canonical receipt first.
- Clean, single-file-changed, and malformed-transition CLI status outputs had
  equal normalized SHA-256 digests and equal exit code 0 between baseline and
  candidate. Only the permitted `.compatibility.runtimeDigest` field was
  normalized; all governance fields and evidence bindings were retained.
- Two independently built indexes in the unit test each parse the fixture once;
  no index is retained after the status observation and no repository, Work Item,
  or request can share it.

See `output-parity.json`, the raw status JSON files, and
`preflight-negative-tests.json` for the bound evidence.

## Limits and governance state

The resident MCP scenario and the other matrix scenarios were not measured in
this Work Item. Runtime phase timing, actual read bytes, hashed bytes, internal
Git calls, runtime child-process counts, cache invalidation events, and peak
memory are explicitly unavailable on this platform; none are represented as
zero. The current benchmark also cannot measure host CPU contention, which is
why the adverse round40 run remains separately classified.

This Work Item is `in_progress` pending Runtime verification, hosted PR review,
merge, archive, close, and documentation promotion. No PR or final governance
decision is claimed yet.
