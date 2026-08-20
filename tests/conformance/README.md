# Conformance harness

Each case has a repository material fixture, a contract, evidence material,
an explicit governance input, and an expected semantic result. The Rust test
loads the files from disk and compares decision fields rather than formatting.
`v1-reference.lock` records the V1 reference commit used to author the corpus;
normal builds do not fetch or execute V1.
