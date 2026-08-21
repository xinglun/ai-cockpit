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
