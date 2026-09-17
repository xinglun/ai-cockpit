---
author: AI Cockpit maintainers
title: "Performance Baseline"
description: "Reproducible local performance evidence and its release limitations."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - performance_baseline
---

# Performance baseline (local evidence)

This baseline was captured with:

```text
command: cargo test -p cockpit-cli --test performance -- --nocapture
source base: 9177b119d3232bbc48dacca71c0beff31089e82b
host: aarch64-apple-darwin (Darwin arm64)
toolchain: rustc/cargo 1.94.1
profile: dev, incremental test fixture
date: 2026-08-21
```

The source tree was a dirty local candidate when measured. These numbers are a
machine-specific baseline, not release evidence; rerun them from the immutable
release candidate before publication.

| Surface | Fixture | Result |
| --- | --- | --- |
| `status` warm startup | 12 samples | median 23 ms |
| repository observation (incremental cache hit) | 200 generated files, 405 files read | 63 ms |
| knowledge unrelated query | 10,000 records | 0 historical records accessed |

The status target (<50 ms) and incremental observation target (<100 ms) are met
in this run. The first uncached scan is measured separately; the acceptance target
applies to the incremental cache-hit path. The raw command output must be retained
with the release candidate's acceptance records.

## Current paired capture (WI-876)

The current candidate was measured on the same aarch64-apple-darwin machine with
`rustc 1.98.1`/`cargo 1.98.1`, Runtime `0.2.93`, an external baseline binary,
and a separately built candidate binary. Seven isolated fixture shapes were paired:
small clean, many-file clean (ORG-X), many historical Work Items
(ai-investigation-orchestrator), single-file change, multi-file change,
large-file change, and an evidence-path change. Each operation has 100 valid warm samples; the raw samples,
identities, counters, and unavailable reasons are retained in
`.ai/evidence/WI-876-performance-current-proof/paired-current-seven-scenarios.json`.

The paired comparator used a 5 ms noise budget. The capture is **not a proven
improvement**: 23 operation comparisons were within noise, 2 improved, and 10
exceeded the provisional noise budget. This is measurement evidence, not a
release-performance pass. Diagnostics on/off overhead is reported separately in
`diagnostics-overhead-org-x.json`; internal counters remain unavailable where the
Runtime does not expose them. Development-cycle cost is separate: the only
currently observed stage is Contract→checkpoint at 63,000 ms (one sample); agent
operation and preflight-rejection counts, verification→finish, and post-merge
cleanup are explicitly unavailable for the active WI.

## Current object-repository capture (WI-889)

WI-889 replaces that historical candidate with a current `0.2.95` paired
capture on the same `aarch64-apple-darwin` host and Rust/Cargo `1.98.1` as the
`0.2.93` baseline. Goods-garden, sentinel, and
ai-investigation-orchestrator were observed only through temporary views. Seven
scenarios produced 38 operation comparisons, each with 100 valid warm samples;
all p50/p95 budget decisions were `within_noise` under a 5 ms noise budget.
p99 is also retained as a tail diagnostic. This establishes comparable current
evidence, not a proven speed improvement. The small-clean object scenario was
unavailable because every supplied object exceeded the harness's `<=100`
tracked-file threshold. Complete raw captures and counters are retained in the
WI-889 evidence archive.
