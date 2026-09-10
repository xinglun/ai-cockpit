---
author: AI Cockpit maintainers
title: "WI-692 — P0 concurrent verification measurement"
description: "Measure the real concurrent verification request path before considering PhysicalSingleFlightCoordinator integration."
audience: [maintainer, reviewer, adopter]
workItemId: WI-692-p0-concurrent-verification-measurement
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-692-p0-concurrent-verification-measurement
terminalArchive: .ai/work-items/archive/WI-692-p0-concurrent-verification-measurement.contract.json
terminalVerification: .ai/evidence/WI-692-p0-concurrent-verification-measurement.verification.json
terminalFinalization: .ai/decisions/WI-692-p0-concurrent-verification-measurement.finalize.json
terminalDecision: .ai/decisions/WI-692-p0-concurrent-verification-measurement.close.json
---

[简体中文](WI-692-p0-concurrent-verification-measurement.zh-CN.md) · [日本語](WI-692-p0-concurrent-verification-measurement.ja.md)

# WI-692 — P0 concurrent verification measurement

## Intent

Measure whether concurrent verification requests for the same repository and
command duplicate physical execution, and decide whether a future
`PhysicalSingleFlightCoordinator` integration is justified. The North Star is
Calibrated Human-Agent Trust: evidence, repository isolation, authorization,
fail-closed behavior, and recovery remain more important than an inferred
throughput gain.

## Scope and validity boundary

This Work Item adds only a reproducible external measurement harness and its
evidence. The harness launches independent public `ai-cockpit verify` CLI
processes against one clean fixture. It preserves request order, raw results,
exit codes, command identity material, receipt metrics, round wall time, and
environment identity. It does not call the coordinator directly and does not
modify `crates/**`, verification semantics, a gate, or a production caller.

The observed four-process execution count is therefore an independent-process
baseline. It is not proof that an in-process MCP/service request path can share
a physical execution. A future integration is valid only after that real path
demonstrates same-identity duplication and a material cost, while retaining
per-request authorization, evidence binding, failure, cancellation, and
resource boundaries.

## Bottleneck evidence and hypothesis

The hypothesis was that concurrent same-identity requests might repeat the
physical command and create a measurable target for single-flight coordination.
The harness used the release-built Runtime `0.2.87`, file digest
`sha256:7610b70b38dca6520ee8ee8cc15b838bb3c1d5d7f4b350234b02f055334fb3a6`,
against a clean detached fixture at `99f7d2323ffb59b1e3edd6c1c833b00f50fb9698`.
The fixture had 9,955 tracked files, no working-tree changes, and the expected
repository identity.

The production call-graph inventory at that revision found the coordinator
definition and implementation, but zero production callers. References were
limited to `crates/cockpit-verification/tests/physical_execution.rs`.
Consequently, the measured duplication is real for independent CLI processes,
but it is not a valid basis for wiring the currently unused in-process
coordinator into production.

## Measurement contract and raw result

The harness records raw per-request samples in round order and request-index
order; it sorts only a copy when calculating nearest-rank quantiles. It records
the sample count, raw samples, warm/identity-probe ordering, Runtime and
repository identity, command identity, process count, and every unavailable
metric with a reason. It does not claim a true cold cache: the Runtime was
invoked for identity before the measurement, and `coldCacheClaim` is `false`.
The initial run exposed a harness classification defect (`returnCode == 0` was
treated as success even when the governance receipt said `passed: false`). The
flawed raw run is retained at
`.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.raw-initial-measurement-bug.json`;
the harness was corrected to require both exit code 0 and `governancePassed ==
true`, and the final evidence was rerun rather than silently replacing the
initial record.

The final round wall-clock results are:

| Scenario | Rounds × concurrency | Successful / failed requests | Physical executions per round | p50 / p95 round wall ms |
| --- | ---: | ---: | ---: | ---: |
| Same identity | 20 × 4 | 80 / 0 | 4 on every round | 720.006 / 1094.730 |
| Distinct command identities | 20 × 4 | 80 / 0 | 4 on every round | 503.963 / 726.116 |
| Failure propagation | 20 × 4 | 0 / 80 | 4 on every round | 473.366 / 964.666 |
| Resource contention | 20 × 4 | 80 / 0 | 4 on every round | 674.100 / 993.346 |
| Cancellation | 1 × 1 | 0 / 1 | unavailable: no governance receipt | 106.182 / unreliable |

The 20-round p50 and p95 round summaries meet the harness reliability floor;
p99 is not a reliable claim because 20 rounds and 80 request samples are below
the declared 100-sample floor. Cancellation deliberately has one sample, so no
reliable percentile claim is made. The full raw values and derived summaries
are retained in:

- `.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.raw.json`
- `.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.same-identity.json`
- `.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.distinct-identity.json`
- `.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.failure.json`
- `.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.cancellation.json`
- `.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.resource-contention.json`
- `.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.measurement-summary.json`
- `.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.call-graph.json`

Queue wait, CPU time, peak memory, read bytes, and hashed bytes are explicitly
unavailable because this public CLI path does not expose them or the platform
collector was not enabled. They are not represented as zero. The harness does
record observed request concurrency and receipt process counts. Nested Cargo
resource behavior, resident MCP latency, and in-process service queueing were
not measured.

## Decision

The future coordinator integration is **declined for now**. Four independent
CLI processes did execute four physical commands per round, but no production
caller exists and the tested path cannot establish the identity and evidence
boundary required for safe sharing. The successor prerequisites are:

1. Measure a real in-process MCP or service request path with the same
   repository, Work Item, command, Runtime, and toolchain identity.
2. Bind each waiter's authorization and evidence receipt before sharing any
   physical result.
3. Measure failure, timeout, cancellation, waiter exit, and nested Cargo
   resource behavior on that path.

## Correctness, isolation, and governance

The focused harness tests preserve sample order, refuse to encode unavailable
metrics as zero, and preserve command identity arguments. The call-graph
evidence distinguishes definitions, test references, and production callers.
The measurement is repository-bound, uses an external executable, keeps the
fixture clean, and makes no Rust production change. Existing governance gates,
authorization, evidence semantics, and recovery behavior are not weakened.

This Work Item is `closed`. Runtime verification, reviewed PR delivery, archive,
close, and documentation promotion are recorded by the terminal paths in the
front matter. No production performance benefit is claimed: the coordinator
integration is declined because the measured path had no production caller,
and the decline plus all unknowns remain bound in the measurement summary.
