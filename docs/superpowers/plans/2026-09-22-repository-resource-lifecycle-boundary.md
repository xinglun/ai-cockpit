# Repository resource lifecycle boundary implementation plan

> **Execution note:** Follow the repository Work Item lifecycle and use the
> approved WI-985 worktree. Do not edit generated `.ai` records by hand.

**Goal:** Move resource finalization, ordinary cleanup, and close-time resource
validation into `cockpit-repository::resource_lifecycle` while preserving the
public API, records, errors, write order, historical reads, and recovery
behavior.

**Design:** Keep the new module in the existing crate. Re-export the current
public functions from the root, expose only narrow `pub(crate)` helper seams
where required, and replace any root-glob dependency with explicit imports.
No protocol, CLI/MCP, release-script, or legacy compatibility behavior is
changed.

## Step 1: Establish the failing focused regressions

Files:

- Add `crates/cockpit-repository/tests/resource_finalization.rs`.
- Add `crates/cockpit-repository/tests/ordinary_cleanup.rs`.

Tests:

1. Add a small boundary assertion that `src/resource_lifecycle.rs` exists and
   contains no `use super::*`; this is intentionally red before the module is
   created and protects the explicit-import requirement.
2. Exercise a valid resource-finalization plan/record/verify path through the
   existing public root exports and assert receipt identity, decision path,
   and observable state.
3. Exercise duplicate/replay and foreign/stale finalization inputs and assert
   the existing fail-closed error behavior.
4. Exercise exact ordinary cleanup and assert the identity-bound receipt,
   sequence, predecessor, and observed branch/worktree facts.
5. Exercise missing, dirty, and foreign cleanup resources and assert that the
   result remains failed/unknown rather than verified.
6. Exercise archive/close and historical/recovery compatibility through the
   public root API where the focused fixtures can do so; leave broader
   existing integration coverage intact.

Run the smallest focused test targets and confirm the boundary assertion is
red because the new module is not yet present. The behavioral tests may serve
as characterization tests and pass against the old implementation; they must
not be weakened or changed to manufacture a failure.

## Step 2: Extract the complete resource lifecycle module

Files:

- Add `crates/cockpit-repository/src/resource_lifecycle.rs`.
- Edit `crates/cockpit-repository/src/lib.rs`.

Actions:

1. Move the resource-finalization planning, receipt and transition readers,
   binding/replay validators, finalization verification, and resource-bound
   decision helpers as one complete use case.
2. Move ordinary-cleanup binding/receipt validation, exact branch/worktree
   observation, receipt recording, and cleanup completion calculation.
3. Move only close helpers whose sole purpose is resource finalization or
   ordinary cleanup preconditions. Keep general archive, historical migration,
   and close orchestration owned by their current boundary.
4. Declare the module in `lib.rs` and explicitly `pub use` the existing public
   entry points. Preserve names, signatures, visibility, and root call paths.
5. Replace any `super::*` dependency with explicit imports. Promote a helper
   to `pub(crate)` only when it is a real shared primitive; otherwise move the
   helper with the use case. Do not create a broad prelude or duplicate a
   validator.
6. Preserve every receipt path, field, digest input, filename sequence, write
   order, error construction, lock boundary, and historical/recovery branch.

Run `cargo fmt --all -- --check` and the two new focused test targets while
iterating. Keep the diff limited to the declared source/test/docs paths.

## Step 3: Update responsibility and Work Item documentation

Files:

- `docs/reference/architecture-responsibility-map-2026-09.md`
- `docs/reference/architecture-responsibility-map-2026-09.zh-CN.md`
- `docs/reference/architecture-responsibility-map-2026-09.ja.md`
- `docs/work-items/WI-985-repository-resource-lifecycle.md`
- `docs/work-items/WI-985-repository-resource-lifecycle.zh-CN.md`
- `docs/work-items/WI-985-repository-resource-lifecycle.ja.md`
- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`

Document that resource finalization and ordinary cleanup are owned by
`resource_lifecycle.rs`, while archive/history and generic lifecycle
coordination remain separate. State that this is a maintenance boundary and
make no performance claim. Keep the three language projections semantically
equivalent and retain explicit pending/verification language until Runtime
evidence is complete.

## Step 4: Verify behavior and governance evidence

Run, in order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked -p cockpit-repository --test resource_finalization`
3. `cargo test --locked -p cockpit-repository --test ordinary_cleanup`
4. `cargo test --locked -p cockpit-repository --lib`
5. `cargo test --locked -p cockpit-repository --tests`
6. `cargo test --locked -p cockpit-cli --test mcp --test outcome_handoff`

Then inspect the diff and public symbol list, run the repository-bound Runtime
verification, and bind each required scenario to current evidence. Confirm
that no generated receipt or historical archive was hand-edited, no unrelated
worktree/resource was touched, and no performance benefit is reported.

## Step 5: Lifecycle completion

After verification is green, run the Runtime finish/archive path, create the
reviewed PR, wait for hosted required checks, merge only the reviewed PR, and
follow WI-985's exact resource route. Synchronize the remote default branch,
remove only the exact WI-985 branch/worktree, record ordinary cleanup when
applicable, run documentation promotion `--check-all`, close the Work Item,
and deliver a visible Outcome with status, evidence, unknowns, resolved
issues, risks, verification, impact, and next action.
