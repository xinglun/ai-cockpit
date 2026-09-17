# WI-880 recovery scope gate

## Boundary

This Runtime-only repair addresses Issue #851. It does not change Sentinel,
archived bytes, verification execution, release behavior, or ordinary Work Item
scope gates.

## Rule

An active successor must name its predecessor and use the unique,
repository-bound recovery decision that selects it. That decision can authorize
the successor's relation to the predecessor. For a different,
manifest-verified historical archive, an otherwise unknown scope relation is
skippable only when every candidate/archive pattern pair has a deterministic
literal filename-suffix proof of disjointness. Missing or invalid historical
evidence, an ambiguous recovery edge, overlap, and all other unknown relations
remain fail-closed.

## Evidence

- `crates/cockpit-repository/src/lib.rs` is the single implementation path for
  start and preflight scope checks.
- `crates/cockpit-repository/tests/lifecycle_entry.rs` preserves ordinary,
  malformed, foreign, and tampered-history boundaries.
- `crates/cockpit-repository/tests/status_projection.rs` covers a valid recovery
  edge, a suffix-proven disjoint unknown scope, an ordinary Work Item, and a
  mismatched recovery edge.
- The three `repository-workflow` and `capabilities` pages describe the same
  narrow semantics.

## Verification

Run focused repository tests first, then the Runtime workspace verification
with `CARGO_INCREMENTAL=0` and the shared verification target. Record any
precondition rejection before an expensive verification attempt; no historical
archive bytes may change.
