---
title: Repository resource lifecycle boundary
status: approved-for-plan
workItemId: WI-985-repository-resource-lifecycle
baseRevision: 6d4b14eaa5744ec449588d3211970feb66ccd651
---

# Repository resource lifecycle boundary

## Purpose

Extract the resource-bound lifecycle implementation from
`crates/cockpit-repository/src/lib.rs` into a same-crate
`resource_lifecycle` module. The change is a maintenance boundary: it reduces
the number of unrelated responsibilities in the root module while preserving
the current public API and all governance behavior.

This is not a performance change and does not introduce a new crate, a
workflow engine, a second governance implementation, or a new protocol.

## Evidence and current boundary

The current `lib.rs` contains complete resource lifecycle use cases alongside
repository observation, archive/history handling, knowledge projection, and
status projection. The relevant contiguous areas include:

- `plan_resource_finalization` and its resource-finalization validation and
  receipt helpers (around lines 7,836–9,977).
- `OrdinaryCleanupObservation`, `OrdinaryCleanupResult`,
  `OrdinaryCleanupReceipt`, observation, receipt-chain validation, and
  `record_ordinary_cleanup_with_runtime` (around lines 9,996–10,715).
- `resource_cleanup_completion_state` and the close entry points and
  close-before-resource validation (from around line 10,715 onward).

The repository already exposes same-crate module boundaries for lifecycle,
evidence storage, execution context, historical compatibility, knowledge,
status projection, and Outcome rendering. The architecture responsibility map
identifies the next safe step as tightening one complete use case at a time,
with public APIs, JSON/file layout, errors, historical reads, and recovery
behavior unchanged.

## Design

### Module ownership

Create `crates/cockpit-repository/src/resource_lifecycle.rs` and move into it
the implementation whose primary responsibility is resource identity and
resource cleanup:

1. Resource-finalization planning, receipt/transition loading, replay and
   binding validation, finalization verification, and the resource-bound
   finalization decision path.
2. Ordinary-cleanup binding, exact branch/worktree observation, append-only
   cleanup-receipt validation, and cleanup receipt recording.
3. The close-time resource-cleanup completion calculation and the close
   precondition helpers that exist only to validate resource finalization or
   ordinary cleanup state.

The root module retains stable public entry points through explicit
re-exports. Root-only operations such as general archive construction,
historical migration, status projection, and generic atomic file primitives
remain owned by their current modules or by the root until a later bounded
Work Item proves a separate boundary.

### Dependency direction

`resource_lifecycle` may consume typed protocol facts and narrow `pub(crate)`
repository primitives, but it must not use `use super::*`. Imports must name
the exact types, constants, and helpers used. Any root helper required by the
extracted code is exposed as a narrow `pub(crate)` helper or moved with the
complete use case; no broad root prelude is introduced.

The module must not call CLI/MCP code, alter protocol serialization, or grant
authorization. It validates and records the same facts and delegates generic
observation, locks, JSON parsing, and atomic writes to the existing helpers.

### Compatibility contract

The following are invariants, not implementation suggestions:

- Existing public function names, argument types, return types, and root
  re-export paths remain available.
- Existing receipt and decision paths, JSON field names, schema versions,
  digest inputs, filename sequencing, predecessor links, and write order are
  unchanged.
- Existing `ObserverError` variants/messages and fail-closed behavior remain
  unchanged unless a test demonstrates that the extraction changed them; no
  error normalization is part of this Work Item.
- Resource identity remains bound to repository, Work Item, Contract,
  branch/ref, head revision, worktree identity, and runtime facts exactly as
  before.
- Duplicate, replayed, stale, foreign, dirty, missing, and indeterminate
  resources remain rejected or reported with their current state; no cleanup
  path may turn unknown into verified.
- Historical archive reads, recovery decisions, and legacy compatibility
  paths continue to use their existing validators. The extraction must not
  delete or rewrite historical evidence.

### Test strategy

Use the existing focused repository tests if present; otherwise add the
smallest tests under the existing test organization without changing test
fixtures outside this Work Item. Cover:

- a valid resource-finalization path and its receipt/decision bindings;
- duplicate/replay and invalid or foreign finalization inputs;
- exact ordinary-cleanup identity and identity-bound receipt creation;
- missing, dirty, foreign, or otherwise unresolved cleanup resources;
- archive/close and historical/recovery regression paths through the stable
  root API.

Run the Contract-declared formatting, repository library/integration, and
CLI MCP/Outcome regression commands. The acceptance evidence must show the
same observable records and states before and after the extraction. No timing
or runtime-speed claim is an acceptance criterion.

### Documentation

Update the English, Simplified Chinese, and Japanese architecture
responsibility maps to state that resource finalization and ordinary cleanup
are owned by `resource_lifecycle.rs`, while archive/history and generic
lifecycle coordination retain their existing ownership. Add the three
Work Item documents with the exact scope, compatibility constraints, and
verification evidence. Documentation must describe a maintenance boundary,
not claim that moving code accelerates execution.

## Out of scope

- New crates, protocol or wire changes, CLI/MCP behavior changes, release
  script migration, or legacy compatibility deletion.
- Changes to knowledge caching, query indexing, Outcome delivery semantics,
  observation-context design, or repository-wide performance.
- Reordering writes, changing receipt schemas, changing error categories, or
  broad root-module cleanup unrelated to this resource-lifecycle boundary.
- Provider deletion, branch/worktree deletion, publishing, CI dispatch, or
  release of this Work Item as part of implementation.

## Rollout and acceptance

Implement only after the Runtime checkpoint and a written plan. Start with a
failing focused regression, then extract the smallest complete use-case
boundary, restore all declared tests, and inspect the diff for public API and
record-layout stability. Finish only when the Runtime verification evidence,
scenario coverage, documentation promotion, reviewed PR, merge, exact
Work Item cleanup, and terminal Outcome are all current.
