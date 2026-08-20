# Fixture layout

Conformance cases contain `input.json`, `contract.json`, `repository/`,
`evidence/`, and `expected.json`. The Rust harness loads the input and expected
semantics from disk. V1 provenance is pinned in `../conformance/v1-reference.lock`;
the V1 runtime is not invoked during normal tests.
