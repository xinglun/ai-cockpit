# WI-1033 Cross-WI Review Corrections Recovery Plan

## Goal

Complete Runtime-bound acceptance and lifecycle closure for the already implemented WI-1032 review fixes, preserve the original recovery history, and stop before release. Implementation is serial in this linked worktree; the independent PR review remains separate.

## Constraints

- Keep exactly one active Work Item, branch, worktree, and repository context.
- Do not rewrite WI-1031 or WI-1032 records. WI-1032 is archived as replaced/not_verified; WI-1031 verification is historically passed but its human close is still pending.
- Do not change Sentinel. Use the candidate release binary only for read-only compatibility inspection and prove protected state is byte-identical.
- Keep installed Runtime 0.2.113 fixed. Do not infer that an older Runtime understands or enforces candidate collaboration fields.
- Use Runtime CLI + Cargo/CI canonical gates; this Rust repository has no supported Make gate.
- Keep query routes read-only and durable changes behind explicit Runtime write commands.
- Do not release, publish, create or move tags, or upgrade the Runtime.
- Task 8 (Runtime snapshot binding across commits) is a separate serial Work Item after this one is integrated and cleaned up.

## Remaining work

### 1. Establish the recovery Contract before implementation admission

- Add this specification and plan in the WI-1033 worktree.
- Activate the Runtime-generated recovery scaffold with the same bounded intent and source scope.
- Declare all nineteen acceptance criteria, the fifteen required scenarios with both expected results and verification plans, required evidence classes, and the correct canonical docs command.
- Cover the retirement-parity invariant: every required locale row binds the retirement receipt and not_verified state; any missing locale fails closed without inventing successful verification evidence.
- Run Runtime preflight and checkpoint only when the current Runtime admits them. Preserve any rejection without retrying unchanged inputs.

### 2. Revalidate existing implementation, repairing only demonstrated gaps

- Trace each independent-review finding to the implementation and a regression test.
- Include Contract-bound required-check coverage labels, no-follow evidence reads at registration/publication/inspection, and execution-repository/CoordinationStore common-directory identity as explicit review findings with pre-spawn negative tests.
- Add failing gate regressions for a valid active recovery successor and a replaced/not_verified archive; distinguish intermediate lineage from terminal close, retirement evidence from successful verification evidence, require complete locale parity, reject contradictory verification-receipt claims, and cover each missing-row case.
- Run focused negative and positive tests for trust identity, complete required-check coverage, crash recovery/live lock safety, evidence containment, provider/outcome selection, actual reuse, MCP identity, and query/write persistence.
- If a gap is found, add the failing regression first, implement the narrow repair, commit it separately, then re-query Runtime because the repository snapshot changed.
- Do not restate classification-function tests as proof of process reuse; count real spawned processes.
- Add a failing provider-registration regression where an old output is removed at a newer generation; prove the persisted event retains the old output ID and invalidation reaches direct and multi-level consumers.
- Add failing coordination-store regressions for traversal/mismatched request identities, including the persisted target Work Item ID and registration-to-target binding, plus non-Requested creation states; assert no target bytes or request records change on rejection.
- Implement only after observing each regression fail for the expected reason; rerun the focused `collaboration_admission` and `coordination_store` integration suites after each fix.

### 3. Run canonical and object-project compatibility evidence

Run, in order, the Contract-declared checks:

- cargo fmt --all -- --check
- cargo test --locked --workspace
- cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
- python3 tests/ci/repository_gate_manifest_test.py
- cargo build --locked --release -p cockpit-cli --bin ai-cockpit
- python3 tests/acceptance/cross_wi_coordination_processes.py --binary target/release/ai-cockpit
- bash tests/docs/documentation_acceptance.sh
- bash tests/ci/governance_integrity_gate_test.sh

Use the candidate binary for Sentinel inspection. Record exact before/after Git tree, Contract/evidence digests, lifecycle Runtime identity, and coordination-store bytes; stop if any protected state changes.

### 4. Runtime handoff, review, merge, and cleanup

- Re-query inspect, status, doctor, and the Work Item Outcome before each lifecycle boundary.
- Run Runtime-bound verification only against the final committed snapshot. Keep every attempt and rejection receipt.
- Deliver an explicit Outcome that separates implementation, verification, independent review, merge, local cleanup, historical lineage close, and release status.
- Push the authorized branch, create/update the PR, wait for hosted checks, obtain independent review, merge, and perform exact branch/worktree cleanup only when Runtime admits each action.
- Resolve the WI-1031 → WI-1032 → WI-1033 historical chain with the Runtime's selected-successor-lineage mechanism only after all required archived-node facts are available. Do not manufacture close evidence.
- Stop before release. Once this WI and exact cleanup are complete, create a distinct serial Work Item for Task 8.
