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
`regression_gate.sh <baseline.json> <candidate.json>` gate to reject missing
samples, zero-iteration samples, identity mismatches, and budget regressions.
The gate consumes captured evidence only; it never builds a source fallback.

Verification scheduling also supports per-command resource weights and an
explicit resource budget. A command whose weight is zero or exceeds the budget
fails closed, while dependency order, protected nodes, and receipt reuse keep
their existing semantics. Repository contexts and runtime sessions are
request-scoped; they do not create a process-global current repository.
