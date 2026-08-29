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
[work-item-id] [budgets.json]` harness measures cold and warm process latency for
`inspect`, `status`, `doctor`, `observe`, and (when selected) Work Item status and
diagnosis. It requires an external executable regular file, records both the
Runtime-reported and file SHA-256 identities, writes atomically, and never builds
or runs a source fallback. Its output is measurement evidence only; use
`regression_gate.sh` with an explicitly reviewed budget file for a release gate.
