# Performance acceptance fixtures

These benchmarks are intentionally reproducible and architecture-oriented. Run
`cargo test -p cockpit-cli --test performance -- --nocapture` to measure warm
`status` startup and a medium repository observation. The test output records
sample count, median startup, files read, and elapsed observation time.

The knowledge crate also contains the 10,000-record unrelated-dependency query:
it asserts `historical records accessed = 0`. The bounded verification receipt
records `nodesPlanned`, `nodesExecuted`, `nodesReused`, `gitCalls`, `filesRead`,
`filesHashed`, `processesSpawned`, and `elapsedMs`.

The <50 ms status and <100 ms incremental-observation numbers are release
targets, not prose claims. A release gate must attach the captured benchmark
output to its evidence bundle on the target platform.

The runtime exposes identity-bound `PerformanceBaseline` records for local
fixtures. Each record requires `runtimeVersion`, `runtimeDigest`,
`repositoryId`, capture time, samples, and explicit budgets. Run the portable
`regression_gate.sh <baseline.json> <candidate.json>` gate for the legacy
schema 1 fixtures; it rejects missing samples, zero-iteration samples, identity
mismatches, and budget regressions. The gate consumes captured evidence only;
it never builds a source fallback. Schema 2 is the P0 evidence contract and is
checked separately by `p0_regression_gate.sh`: baseline and candidate Runtime
versions/digests may differ, but repository identity, repository snapshot,
evidence completeness, and a comparable environment must bind. A budget must
name an explicit metric (`warm.p50Ms`, `warm.p95Ms`, or `warm.p99Ms`); an
unreliable percentile is a gate failure and cannot fall back to `elapsedMs` or
another percentile.

Verification scheduling also supports per-command resource weights and an
explicit resource budget. A command whose weight is zero or exceeds the budget
fails closed, while dependency order, protected nodes, and receipt reuse keep
their existing semantics. Repository contexts and runtime sessions are
request-scoped; they do not create a process-global current repository.

For a Work Item, the detected Cargo/npm command is dynamically eligible for
the same profile-authorized receipt reuse as standalone auto-detected
verification. The Runtime compares the complete identity context before every
reuse decision. Changed or unknown impact executes the declared command and
records the reason, while explicit custom commands remain fresh. Adopter
acceptance must run the identical cold/warm sequence with the published binary
and retain both repository and Runtime identities; local source builds are not
valid acceptance evidence.

WI-395's Rust-native optimization removes duplicate snapshot capture from
aggregate Work Item status, captures the source-tree digest during the
existing Git index read, resolves remote default metadata with one bounded
query, and avoids repeated recursive sorting during observation. The
optimization is request-scoped and identity-bound: it does
not create a global repository cache or copy the reference install flow.

The portable `runtime_benchmark.sh <binary> <repo> <output.json> [iterations]
[work-item-id] [budgets.json]` harness emits schema 2 evidence for end-to-end
wall-clock process latency. It preserves the acquisition order, records the
first measured process separately from independent CLI processes after a
bounded OS-cache warmup, and explicitly marks resident MCP as not measured.
Each record retains raw samples, warmup count, sample count, quantile method,
Runtime/repository identity, repository state, data scale, scenario-matrix
status, phase boundaries, cache-invalidation reasons, and resource metrics.
Internal read bytes, hash bytes, Git calls, child-process counts, and peak
memory are marked unavailable when the Runtime or platform cannot provide
them; they are never represented as zero. It requires an external executable
regular file, records both the Runtime-reported and file SHA-256 identities,
writes atomically, and never builds or runs a source fallback. A one-iteration
sanity run must not be used for p95/p99 claims. Use `p0_regression_gate.sh`
with an explicitly reviewed budget file for schema 2 release evidence; use
`regression_gate.sh` only for the legacy schema 1 fixtures.

The P0 scenario matrix names small and many-file clean repositories, single-,
multi-, and large-file changes, many historical Work Items, concurrent
validation requests, and resident MCP repeat queries. A single invocation is
bound to its supplied repository and reports unselected scenarios as
`not_measured`; it does not synthesize their results. A requested scenario is
measured only when captured facts prove its shape: `small-clean` is clean with
at most 100 tracked files, `many-files-clean` is clean with at least 1,000,
`single-file-change`/`multi-file-change` have exactly one/at least two changed
paths, `large-file-change` has a changed file of at least 1 MiB, and
`many-historical-wi` has at least 100 archived Work Items. The portable harness
does not execute concurrent requests or resident MCP transport, so those
scenarios remain explicitly `not_measured`.
