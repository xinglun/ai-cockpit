# WI-1081 in-scope review corrections

The complete 13-node pre-review baseline remains immutable under its original source snapshot `sha256:6ffcd0aaadd80bdba91dc6d7b70dc3baf588d76be913ba8dc89c5123f55d2dcc` and receipt digest `sha256:a89dbf133da5282400166c3dc8ec8f67b414c243419c91370bec12b8c86fa864`. It does not verify the later corrections.

## Reproduced failures

Regression commit `a9d4b65a` added three meaningful negative cases before changing production validation. The Runtime v4 test target reported 3 passing and 2 failing tests: exchanging the source/sync selectors was incorrectly accepted, and a public Retained→Deleted record/verify flow still verified after the root's old remote observation was removed. The separate idempotent-observe test failed because a stored winner with `boundAt="not-a-date"` was returned successfully. The complete original RED Runtime outputs and exit codes are retained.

## Minimal corrections

Production correction commit `458f87505c0183b5d96dd78f7d6a78ef33c949e8` changes only the two existing in-scope repository source files:

- Preserve the complete selected observation or legacy receipt in a typed value, assign source/sync/remote values explicitly, and carry those values directly through digest, subject, assurance, time, PR-body, and remote-ref validation. Schema v4 never rematches the selected set by raw digest. Legacy v1/v2/v3 retain their unique raw-digest resolution.
- Validate the stored winner's `boundAt` with the same RFC3339 validator before idempotent observe returns it. Rejection preserves the existing record and raw bytes.
- For a v4 Deleted head, revalidate the exact v4 Retained root's delegated observations, including its old remote selector. This does not require the old local resources to remain present and does not change legacy-history interpretation.

The public v4 history regression records Retained, performs only fixture-owned branch/worktree deletion, appends a valid fresh Deleted observation/transition, and first verifies the intact head. It then checks missing root observation and changed root observation bytes (a syntactically valid replacement `boundAt`); both must fail. The canonical root receipt bytes must remain unchanged. The prior v3 fixture's validation/recording assertions remain in its shared setup and are still executed.

## Scoped GREEN evidence

All checks ran through the installed repository-bound Runtime, one build task, retained Cargo target, and the writable process-local HOME:

- `p2-v4-regressions-green`: 5/5 passing.
- `p2-observation-regressions-green`: 7/7 passing, including the malformed stored winner.
- `p2-v3-compatibility-green`: the complete existing v3 public-recording case passes.

These are targeted checks. Complete Contract verification of the corrected source is still pending at this checkpoint. Contract remains `sha256:8f28e50938c1c594db450768be4ca14db02e46f0866c7316893f86e4c5bbb33b`; corrected source snapshot is `sha256:1df671d690e6b41ba964633198718f2d06d36f63cced491d4af9ebb7aa85cdc1`. No observer production code, test assertion, system permission, historical Work Item, or prior receipt was weakened or rewritten.
