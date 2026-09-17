# WI-889 raw Runtime captures

The paired report keeps the comparable warm samples and percentile results in
reviewable JSON. The complete collector output (including phase timings,
process/resource counters, cache-invalidation reasons, and per-operation
records) is retained in `runtime-captures.tar.gz` so the repository does not
grow by appending fourteen multi-megabyte JSON files.

The archive contains the seven baseline captures from Runtime `0.2.93`, the
seven candidate captures from Runtime `0.2.95`, and the candidate
`current-repository-diagnostics-on.json` capture used for the diagnostics
overhead comparison. Every capture requested 100 warm measurements; the
paired comparator accepted only captures with at least 100 valid warm samples.

Capture identity:

- host: `aarch64-apple-darwin`
- rustc: `1.98.1 (48a229cea 2026-09-01)`
- cargo: `1.98.1 (797e8a9bc 2026-08-05)`
- toolchain: `1.98.1-aarch64-apple-darwin`
- build mode: isolated baseline/candidate binaries, `CARGO_INCREMENTAL=0`
- verification target: `/Users/sei-rinn/.cache/ai-cockpit-verify-target`
- archive SHA-256: `f7a71bbb10ff62d0d68ceb37c74c39a0f57c533c79caa1f2136ac07613af2176`

The cache path above is an execution detail, not a required input to consume
the committed evidence. Object repositories were observed only through
temporary views; their main branches and pre-existing working trees were not
modified.
