---
author: AI Cockpit maintainers
title: WI-657 — Concurrent finish_work_item fault injection
description: A controlled two-thread fault-injection test finds and fixes a real atomic_write collision, and documents a second, deeper issue left deliberately unfixed.
workItemId: WI-657-lifecycle-concurrency-fault-injection
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-657-lifecycle-concurrency-fault-injection
terminalArchive: .ai/work-items/archive/WI-657-lifecycle-concurrency-fault-injection.contract.json
terminalVerification: .ai/evidence/WI-657-lifecycle-concurrency-fault-injection.verification.json
terminalFinalization: .ai/decisions/WI-657-lifecycle-concurrency-fault-injection.finalize.json
terminalDecision: .ai/decisions/WI-657-lifecycle-concurrency-fault-injection.close.json
---

# WI-657 — Concurrent finish_work_item fault injection

This Work Item is P2-C of the architecture initiative: verify multi-file
consistency and recovery under concurrency by controlled fault injection,
investigating the existing mechanism first rather than presuming a defect
exists.

## What was tested, and why this call

`finish_work_item_internal` is the largest read+decide+persist function
identified by the P0 map (`docs/reference/architecture-responsibility-map-2026-09.md`)
that had never been exercised by two callers racing the same Work Item. The
existing suite already covers sequential retry-after-failure recovery
(`recovery_decision.rs`) and single-caller partial-write rollback, but nothing
forced two threads to call `finish_work_item` for the same Work Item at the
same instant. A new test,
`crates/cockpit-repository/tests/lifecycle_concurrency.rs`, does exactly
that: two threads synchronized on a `std::sync::Barrier` both call
`finish_work_item` for one Work Item once it has reached `checkpointed`.

## Finding 1 (fixed): atomic_write temp-filename collision

`atomic_write` (`crates/cockpit-repository/src/lib.rs`) derived its temp file
name from `std::process::id()` alone. Two same-process threads writing the
same destination path therefore raced for the *identical* temp file path:
whichever thread called `fs::rename` second found its own temp file already
consumed by the other thread's rename, and failed with a misleading
filesystem "not found" on the destination path rather than a business-level
rejection. This is a genuine defect, confirmed empirically (the failure
reproduced across repeated runs before the fix, and stopped reproducing
after it).

The fix pairs the pid with the existing `NEXT_ATOMIC_WRITE_ID` atomic
sequence counter, the same pattern already used elsewhere in this file by
`write_cap_immutable` and the parallel-slot lease writers — not a new
mechanism, an existing precedented one applied to a fourth call site. This
is scoped to `atomic_write` only; no other function changed.

## Finding 2 (deliberately deferred, not fixed): rollback can clobber a concurrent winner

Even after the temp-filename fix, a losing thread can legitimately fail with
a business rejection (for example, a duplicate completion-event check), and
that losing thread's hand-rolled rollback in `finish_work_item_internal`
(the `atomic_json(&summary_path, &original_summary)` calls) restores *its
own* stale pre-attempt snapshot unconditionally. It does not check whether
the current on-disk state was meanwhile advanced by the concurrently
successful sibling call. This can revert a legitimate concurrent success
back to `checkpointed`, silently discarding a successful `finish`.

This is a real correctness gap, not a hypothetical one. Fixing it properly
requires a mutual-exclusion boundary (a lock, or a compare-and-swap check
against the summary's own digest/state before rollback writes) around
`finish`/`archive`/`close`, which is a larger, higher-risk change than this
Work Item's scope, and per this initiative's own risk discipline (see
WI-654's precedent of declining an insufficiently verified fix) it is not
attempted here. It is recorded here as a known, deferred limitation rather
than left undocumented.

## Test design

The new test asserts only what the fix guarantees, not the still-open
second issue:

- At least one of the two concurrent `finish_work_item` calls succeeds.
- No result — success or failure — contains a raw filesystem race artifact
  ("No such file or directory" / "os error 2").
- `summary.json` and `outcome.json` remain valid, parseable JSON regardless
  of which attempt's write ends up on disk last.

It deliberately does **not** assert that the Work Item ends in
`finish_ready`, and does not call `archive_work_item` afterward — both would
depend on Finding 2 being fixed, and asserting them would either mask the
gap or make the test flaky pending a larger redesign.

## Correctness evidence

`cargo test -p cockpit-repository --test lifecycle_concurrency` passed
consistently across 6 repeated runs after the fix (single-threaded runner,
to keep the fault-injection barrier deterministic per run). `cargo test
--locked --workspace`, `cargo fmt --all -- --check`, and `cargo clippy
--locked --workspace --all-targets --all-features -- -D warnings` all pass.

## Out of scope / follow-up

Redesigning `finish_work_item_internal`'s rollback to be safe under
concurrency (Finding 2) is out of scope here. `archive_work_item` and
`close_work_item_with_structured_decision_internal` have similar hand-rolled
rollback code and were not fault-injected in this Work Item; a future P2-C
follow-up should extend the same barrier-based technique to them before any
attempt to fix Finding 2, since a real mutual-exclusion fix likely needs to
cover all three functions together.
