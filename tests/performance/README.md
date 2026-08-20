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
