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

## Historical paired capture (WI-876)

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

## Historical object-repository capture (WI-889)

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

## Current v0.2.98 supplemental capture (WI-905)

WI-905 refreshes the public-version identity without rewriting the historical
WI-876 or WI-889 evidence. It pairs the public `v0.2.93` and `v0.2.98`
binaries on the same `aarch64-apple-darwin` host with Rust/Cargo `1.98.1` and
100 valid warm samples per compared operation. The selected clean object views
were goods-garden (396 tracked files) and ORG-X (1,240 tracked files); neither
main branch was changed or merged. The compact raw bundle, checksums and exact
environment identities are in
`.ai/evidence/WI-905-performance-v098-evidence/raw/`.

The first ten-operation batch used the existing 5 ms noise budget and reported
three provisional regression flags. A separate 100-sample observe repeat did
not reproduce the observe flag (goods-garden: -2.397 ms p50 / +2.148 ms p95;
ORG-X: -0.240 ms p50 / +1.314 ms p95), while the inspect tail was not repeated
and remains unresolved. Internal `git_snapshot` p95 improved from 30.687 to
24.813 ms on goods-garden and from 34.796 to 31.576 ms on ORG-X, but the
end-to-end CLI result is not a proven speed improvement. The candidate
`work-item-outcome --delivery --json` path measured p50 241.329 ms, p95
264.831 ms and p99 371.828 ms over 100 valid warm samples. Diagnostics-on
overhead is reported separately, and Runtime cache-invalidation events remain
explicitly unavailable. Development-cycle stages other than the captured
Contract-to-reviewable-PR sample remain unavailable; they are not treated as
zero.
