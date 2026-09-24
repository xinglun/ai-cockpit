# WI-1031 implementation plan

Implement serially in one Work Item, branch, and worktree. Keep changes split into reviewable commits. Do not release or mutate tags.

## 0. Runtime preparation

- Create the dedicated WI/worktree first (done at base `b57561d`).
- Keep this specification and plan in that worktree.
- Amend the Runtime-owned Contract only through `ai-cockpit work-item amend`; run preflight and record the already-granted, scope-bounded human review through Runtime controls. Do not hand-edit generated Contract/Summary/receipt records and do not add a second start-confirmation gate.
- Re-run inspect/status/preflight after each Contract or repository snapshot change. Implement only when Runtime admits the action.

## 1. Protocol and storage consistency

- Add regression tests first for registration/event interruption and retry.
- Reorder or journal writes so an identity-change event cannot be lost while a new registration becomes authoritative; an inconsistent intermediate state must block consumers.
- Add recovery/reconciliation tests while preserving immutable prior events.
- Bind published evidence digests into append-only `OutcomePublished` events; require current-generation, outcome-specific publication before a verification-required dependency can pass.
- Add one repository publication service and matching CLI/MCP `publish-outcome` writes; validate the current registration and any consumer-required verification receipt before appending the event.
- Commit boundary: protocol/storage and focused tests.

## 2. Dependency and coordination admission

- Add tests for same-WI outcome-specific admission and a three-level impact chain before changing projection logic.
- Filter invalidation blockers by each action's consumed outcome IDs; refresh current dependency state before action admission.
- Filter every outcome-scoped fact blocker, including head and merge-fact blockers, so an unselected outcome cannot block a selected action.
- Propagate invalidation through dependency closure; retain generation-bound, append-only recovery semantics.
- Make `MergedTarget` and verification-required dependencies depend on factual receipts/Git state, not caller assertions or file existence.
- Commit boundary: dependency/coordination and focused tests.

## 3. Composition validation and reuse

- Add negative tests first for stale JSON after environment/toolchain change, omitted required checks, and feature-worktree-to-main target resolution.
- Compute or verify source, executable/toolchain, command, and environment identity at the Runtime boundary. Disable reuse when an input is unknown.
- Derive required checks and participants from Contract/registration facts; reject missing coverage before process spawn.
- Scope node identity to direct inputs and upstream receipts so a local change reruns only affected nodes and dependency descendants.
- Keep bounded execution, durable attempts, timeout/interruption records, and cleanup reporting from WI-1030.
- Add a compatibility regression proving an ordinary single-WI path with no collaboration records still runs one-at-a-time through its existing lifecycle and canonical gate.
- Commit boundary: composition/validation/reuse and targeted tests.

## 4. Outcome projection and documentation

- Add projection negatives for temporary-but-unmerged, moved target/stale history, and safely paused current generation.
- Separate composition success, actual target merge, cleanup, current applicability, and reusable checks. Use the same admission decision as execution.
- Update user-facing docs and gate manifest only where required to bind the new tests to canonical CI.
- Commit boundary: projection/docs/gate mapping and tests.

## 5. Verification and integration

- Run focused package tests after each boundary, including CLI and MCP publication-route parity, then `cargo fmt --all -- --check`, `cargo test --locked --workspace`, `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`, process acceptance, and manifest validation.
- Exercise multiple real concurrent processes and linked worktrees; verify an actual second CLI/MCP invocation starts zero processes only when observed inputs match, and changed inputs rerun only affected nodes.
- Run the canonical Runtime/Cargo/CI gate on the exact merge candidate. Resolve CI and review findings, merge and clean up under the authorized scope, and stop before release. Deliver a human-visible Outcome with evidence, unknowns, and the remaining release decision clearly separated.

## Execution rule

Use test-first development for each defect: make the specific negative test fail, implement the narrow correction, rerun the focused test, then the applicable package and canonical gates. Preserve failed/stale evidence and unrelated cleanup obligations.
