# WI-905 current v0.2.98 capture bundle

This compact archive contains the raw JSON captures used by WI-905. It is a
supplement to the immutable WI-876/WI-889 evidence; those historical captures
are not rewritten.

## Identity

- Baseline: public `v0.2.93` binary.
- Candidate: public `v0.2.98` binary, Runtime digest
  `sha256:7a49278b2fed775668b4d42222702b66cefc10d57ec36fb476d35d2d4d7f0eeb`.
- Host: `aarch64-apple-darwin`, macOS 27 arm64.
- Toolchain: `rustc/cargo 1.98.1`, rustup toolchain
  `1.98.1-aarch64-apple-darwin`.
- Object views: goods-garden (396 tracked files, current-repository) and
  ORG-X (1,240 tracked files, many-files-clean). Both views were clean and
  were not modified or merged.
- Every persisted comparison contains 100 valid warm samples. The archive
  retains the source captures, paired report, diagnostic on/off captures,
  Outcome query samples, internal phase samples, and the repeat observe run.

## Interpretation

- The first 10-operation paired capture used a 5 ms noise budget and flagged
  three provisional regressions. A second 100-sample observe-only run reduced
  the goods-garden delta to `-2.397 ms` p50 / `+2.148 ms` p95 and the ORG-X
  delta to `-0.240 ms` p50 / `+1.314 ms` p95; the observe flag was therefore
  not stable. Inspect was not repeated, so the first-run inspect tail remains
  an unresolved diagnostic.
- Internal diagnosis showed a p95 `git_snapshot` reduction from 30.687 to
  24.813 ms on goods-garden and 34.796 to 31.576 ms on ORG-X. This is an
  internal phase observation, not proof of a faster end-to-end CLI path.
- Candidate v0.2.98 `work-item-outcome --delivery --json` measured 100 valid
  warm samples: p50 241.329 ms, p95 264.831 ms, p99 371.828 ms.
- Diagnostics-on overhead was measured separately; `observe` added 25.890 ms
  p50 and 35.951 ms p95 on the current repository. Diagnostic fields are not
  part of business evidence identity.
- Cache invalidation events are unavailable from the Runtime and are retained
  as an explicit unknown; no zero value is substituted.
