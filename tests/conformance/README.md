# Conformance harness

Each case has a repository material fixture, a contract, evidence material,
an explicit governance input, and an expected semantic result. The normal Rust
test loads the files from disk and compares decision fields rather than
formatting; it is the fast offline regression and does not fetch or execute V1.

Gate B has two explicit boundaries. Hosted CI runs the committed offline
semantic corpus and never accesses a reference repository. The current source
for future file-by-file comparison is the local Git checkout named by
`AI_COCKPIT_REFERENCE_ROOT`, pinned in `reference-source.lock`; the checkout
must be clean and its HEAD must match the lock. `reference_source_policy.py`
performs this check without cloning or fetching. The legacy executable oracle
remains a maintainer-local, test-only operation using `v1-reference.lock` and
an exact local checkout; it is never a hosted CI dependency.

For a local comparison, set `AI_COCKPIT_REFERENCE_ROOT` to the maintained
checkout and run:

```bash
python3 tests/conformance/reference_source_policy.py \
  --lock tests/conformance/reference-source.lock \
  --reference "$AI_COCKPIT_REFERENCE_ROOT"
```

The legacy V1 runtime and Python are optional local conformance dependencies
only. They are not linked into the Rust binary and are never attached to
adopter repositories.
