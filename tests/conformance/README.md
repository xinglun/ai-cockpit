# Conformance harness

Each case has a repository material fixture, a contract, evidence material,
an explicit governance input, and an expected semantic result. The normal Rust
test loads the files from disk and compares decision fields rather than
formatting; it is the fast offline regression and does not fetch or execute V1.

Gate B has a separate executable boundary. `v1-reference.lock` pins the exact
V1 reference commit. The dedicated CI job checks out that commit, sets
`AI_COCKPIT_V1_ROOT`, and runs
`cargo test -p cockpit-core --test v1_oracle -- --ignored`. The test verifies the
checkout identity before `v1_oracle.py` calls V1 governance primitives for all
fourteen fixtures and compares decision state, blockers, unknowns, safe actions,
required checks, authority, and outcome state. The adapter never reads
`expected.json`; a mismatch is therefore independent evidence, not a second
projection of the Rust result.

The external V1 runtime and Python are conformance-test dependencies only. They
are not linked into the Rust binary and are never attached to adopter
repositories.
