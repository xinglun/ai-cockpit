---
author: AI Cockpit maintainers
title: WI-647 — Benchmark cold/warm grouping correctness
description: Fix cold/warm sample misclassification in the development performance harness before any runtime optimization work.
workItemId: WI-647-benchmark-cold-warm-grouping
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-647-benchmark-cold-warm-grouping
terminalArchive: .ai/work-items/archive/WI-647-benchmark-cold-warm-grouping.contract.json
terminalVerification: .ai/evidence/WI-647-benchmark-cold-warm-grouping.verification.json
terminalFinalization: .ai/decisions/WI-647-benchmark-cold-warm-grouping.finalize.json
terminalDecision: .ai/decisions/WI-647-benchmark-cold-warm-grouping.close.json
---

# WI-647 — Benchmark cold/warm grouping correctness

This Work Item is P0 of the AI Cockpit performance initiative: it makes the
local development performance harness (`tests/performance/runtime_benchmark.sh`)
trustworthy before any runtime optimization is attempted. It changes only
measurement tooling; it does not change governance decisions, evidence
semantics, or the required verification graph.

## Defect found

`runtime_benchmark.sh` (introduced in commit `acb3c386`, WI-402) sorted every
collected timing sample and reported the minimum as the "cold" sample:

```python
values.sort()
warm = values[1:]
...
{"name": f"{name}.cold", "elapsedMs": round(values[0], 3), "iterations": 1}
```

Sorting before grouping silently swaps the first process call for whichever
later call happens to be fastest. On the fixed sequence `[120, 20, 22, 21]`
(first call slow, later calls fast) the old code reported `20` as cold and
`[21, 22, 120]` as warm — the true first call (`120`) was folded into the warm
population instead of being reported as cold. WI-402's own report treated this
script's output at face value and never questioned the sort order.

## Fix

- `tests/performance/runtime_benchmark_stats.py` (new): pure `summarize(name,
  raw_ms, prior_probe_calls)` function. `raw_ms[0]` is always reported as cold;
  `raw_ms[1:]` are the warm samples in original call order (`rawMs`); only a
  separately sorted copy of the warm samples feeds percentile computation.
- `tests/performance/runtime_benchmark_stats_test.py` (new): pins the fixed
  sequence `[120, 20, 22, 21]` (cold must be `120`) plus a reversed-outlier
  sequence, and exercises the reliability floors below.
- `p50Ms`/`p95Ms` are withheld with an explicit `insufficient_samples` reason
  below fixed reliability floors (`MIN_SAMPLES_FOR_P50=5`,
  `MIN_SAMPLES_FOR_P95=20`) instead of reporting a percentile computed from too
  few points. The reported `elapsedMs` falls back to the worst raw sample so a
  budget gate still fails closed even when no percentile claim is reliable.
- Each capture now records an `environment` block (hardware, OS, filesystem,
  repository head/branch/dirty state, tracked file count) and a
  `preMeasurementProcessInvocations` list. `--version`/`inspect`/`status`
  identity probes already run before any sample is measured, so a `.cold`
  sample is the first *measured* call, not a true OS-cold-cache call; the
  harness now says so explicitly instead of implying a pristine cache.
- `measurementModel` records that this harness measures independent CLI
  process latency only, not persistent MCP session latency.
- `tests/performance/regression_gate.sh` required `elapsedMs`/`maxElapsedMs` to
  be Python `int`. Real `runtime_benchmark.sh` output has always produced
  rounded floats, so every real capture failed `sample_malformed` against any
  budget file — the gate had never actually accepted real measurement
  evidence. Fixed to accept `int` or `float` (excluding `bool`) for both
  fields; `iterations` stays a strict `int`.

## Out of scope (deferred to later P0/P1 Work Items)

- The 8-scenario fixture matrix (small/large clean repo, single/multi/large
  file change, many historical Work Items, concurrent verification, resident
  MCP repeat queries).
- Phase-level breakdown (git query / file read+hash / evidence validation /
  scheduling+subprocess / outcome projection) and resource metrics (bytes
  read/hashed, git call count, spawned process count, cache-invalidation
  reasons, peak memory) surfaced on `status`/`doctor`/`observe` — today only
  `inspect` emits `filesRead`/`filesHashed`/`gitCalls`, and no command reports
  process count or peak memory.
- A dev-only comparator that tolerates explicitly bound baseline/candidate
  Runtime identity differences while still verifying each side's evidence
  integrity.
- Persistent MCP session latency measurement.
- Any P1–P3 runtime optimization (request-scoped dedup, `IncrementalMerkle`,
  `PhysicalSingleFlightCoordinator`, polling-wait changes, resident cache, PGO).

## Verification

`cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets
--all-features -- -D warnings`, and `cargo test --locked --workspace` pass
unchanged (no Rust source was touched). `python3
tests/performance/runtime_benchmark_stats_test.py` and `bash
tests/performance/regression_gate_test.sh` pass. `runtime_benchmark.sh` was run
against the installed v0.2.87 binary on this repository; its JSON output was
then fed through `regression_gate.sh` with a real (float) budget file, both for
a passing and a budget-exceeded case, confirming the gate now consumes real
measurement evidence while still failing closed on a genuine regression.

### Local measurement (advisory)

On 2026-09-07, macOS arm64 (Darwin 25.6.0, arm64, 10 logical CPUs), against
this repository (580 archived Work Items, 9614 tracked files, clean HEAD at
`1623ee5a`), a 6-iteration run recorded (old script's misleading grouping in
parentheses): `inspect.cold` 98.951 ms, `status.cold` 1887.792 ms,
`doctor.cold` 50.172 ms, `observe.cold` 119.701 ms. `status` is markedly more
expensive than `inspect`/`doctor`/`observe`; this is raw evidence for the next
P0 bottleneck-ranking Work Item, not a claim this Work Item optimizes. These
are local process-latency observations, not provider or enterprise guarantees.
