---
author: AI Cockpit maintainers
title: "WI-696 — P0 scenario measurement"
description: "Run the existing P0 benchmark against the real scenario matrix and bind a bottleneck order and guarded follow-up budget."
audience: [maintainer, reviewer, adopter]
workItemId: WI-696-p0-scenario-measurement
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-696-p0-scenario-measurement
terminalArchive: .ai/work-items/archive/WI-696-p0-scenario-measurement.contract.json
terminalVerification: .ai/evidence/WI-696-p0-scenario-measurement.verification.json
terminalFinalization: .ai/decisions/WI-696-p0-scenario-measurement.finalize.json
terminalDecision: .ai/decisions/WI-696-p0-scenario-measurement.close.json
---

[简体中文](WI-696-p0-scenario-measurement.zh-CN.md) · [日本語](WI-696-p0-scenario-measurement.ja.md)

# WI-696 — P0 scenario measurement

## Intent

Run the existing order-preserving schema-2 benchmark on reproducible clean and
changed repositories before selecting another optimization. The North Star is
Calibrated Human-Agent Trust: this Work Item records evidence and a guarded
budget only; it does not change Runtime governance behavior, authorization,
repository isolation, or recovery semantics.

## Boundary

The measured set uses the current `runtime_benchmark.sh` and
`concurrent_verification_benchmark.py` without changing their implementation.
The development release binary was built from `origin/main` at
`513e7e72523097893c658ec76c25706fd5b266b8`, copied to an external temporary
path, and identified as Runtime `0.2.87` with file digest
`sha256:faacc6a3368f56bc9274264c111500157fb79354260dba60898a3351b5c3e4dd`.
Published release acceptance remains a separate requirement.

## Measurement contract

Every portable scenario has one first measurement after three identity probes,
one OS-cache warmup process, and 20 independent warm processes. Raw order,
warmup count, sample count, nearest-rank method, Runtime identity, repository
identity, snapshot, OS/machine, data scale, metadata Git calls, direct CLI
processes, unavailable metrics, and the scenario-matrix classification remain
in the raw JSON. The first measurement is not called true cold cache. p95 is
reliable at 20 warm samples; p99 is explicitly unavailable below 100 samples.

## Scenario results

The final comparable development-binary set is under
`.ai/evidence/external/WI-696-p0-scenario-measurement.*.budgeted2.dev-faacc6a3.json`.
The derived record is
`.ai/evidence/external/WI-696-p0-scenario-measurement-summary.json`.

| Scenario | Shape | status warm p50/p95 ms | observe warm p50/p95 ms | Primary facts |
| --- | --- | ---: | ---: | --- |
| small-clean | 15 files, 1,913 bytes, clean | 82.696 / 85.517 | 59.106 / 60.704 | isolated attached fixture |
| single-file-change | 8 files, one changed path | 93.616 / 103.280 | 71.838 / 72.722 | isolated attached fixture |
| multi-file-change | 8 files, three changed paths | 95.430 / 99.833 | 72.077 / 73.432 | isolated attached fixture |
| large-file-change | 6 files, one 1 MiB changed file | 99.895 / 101.456 | 75.675 / 77.739 | large-file shape proven by metadata |
| many-files-clean | 10,044 files, 75,325,549 bytes | 3,095.188 / 3,126.662 | 144.646 / 148.613 | clean `origin/main` at 513e7e72 |
| many-historical-wi | 10,044 files, 604 archived Work Items | 3,103.234 / 3,588.843 | 145.194 / 148.943 | same clean snapshot and scale |

Inspect and doctor remain below the status and observe paths in every measured
shape. The dominant current bottleneck is `status` on the large repository,
especially the many-history case. This is a measured ordering, not a claim
that the implementation should be optimized without a successor Contract.

## Resource and phase boundary

The portable harness observed 91 direct external CLI processes and four Git
calls for its own metadata collection per scenario. Runtime-internal Git calls,
read bytes, hashed bytes, child-process counts, peak memory, phase timing, and
cache-invalidation reasons are unavailable with explicit reasons; no unavailable
metric is encoded as zero. Darwin filesystem type was also not accepted as a
trustworthy comparator key, so a real cross-runtime p0 gate comparison fails
closed on this host until a trustworthy environment key is available.

## Concurrency and resident MCP

The latest development-binary rerun of the independent public CLI harness is
`.ai/evidence/external/WI-696-p0-scenario-measurement.concurrent-validation-requests.dev-faacc6a3.json`.
Same-identity requests performed four physical executions per round (20 rounds,
concurrency four; round p50/p95 441.652/454.629 ms). Failure propagation,
resource contention, and cancellation are retained with their raw samples and
explicit reliability boundaries. The current call graph at `origin/main`
(`.ai/evidence/external/WI-696-p0-scenario-measurement.current-call-graph.json`)
has zero production callers of `PhysicalSingleFlightCoordinator`; wiring it into
production is therefore declined for now. Resident MCP repeat queries remain
`not_measured` because the portable harness has no resident MCP transport.

## Optimization budget and next action

Scenario budgets are recorded in
`.ai/evidence/external/WI-696-p0-scenario-measurement-budgets/`. They are
regression ceilings derived from repeated development measurements plus a
bounded noise allowance, not claimed improvements. A successor optimization
must pre-register its target path and improvement threshold using paired
baseline/candidate evidence, preserve all non-target budgets, and pass
`p0_regression_gate.sh` with complete comparable environment records. The first
candidate investigation should target request-scoped status work on the
large-history path; no large-file streaming, polling replacement, parallel
read/hash, coordinator integration, trusted incremental Merkle reuse, resident
MCP cache, in-process Git, or PGO change is accepted by this Work Item.

## Evidence boundary

This Work Item reports a baseline and measured unknowns. It makes no claim of a
latency, CPU, I/O, memory, or user-visible improvement. Existing WI-685 status
fact reuse and WI-692 independent-CLI concurrency evidence remain separately
bound and are not silently folded into this measurement.

## Historical validation limitation

An earlier repository-wide documentation acceptance, status-consistency, and
`closed-work-item --check-all` run failed closed on the separate WI-694
documentation promotion. The exact historical output is retained in
`.ai/evidence/external/WI-696-p0-scenario-measurement.validation-limitation.json`.
WI-697 subsequently promoted the WI-694 documentation projection; WI-699
reruns the same repository-wide checks for the WI-696 projection. This Work
Item does not modify WI-694 or WI-696 historical evidence and does not claim a
performance benefit.
