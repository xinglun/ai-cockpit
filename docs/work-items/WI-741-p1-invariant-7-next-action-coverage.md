---
author: AI Cockpit maintainers
title: "WI-741 — P1 invariant 7 (next-action correctness) coverage"
description: "Closes the invariant-7 gap named by docs/reference/collaboration-invariant-coverage.md with a bounded test tied to a subset of the collaboration scenario matrix's observed scenarios."
audience: [maintainer, reviewer, adopter]
workItemId: WI-741-p1-invariant-7-next-action-coverage
status: in_progress
authority: authorized
lastVerifiedBy: WI-741-p1-invariant-7-next-action-coverage
---

[简体中文](WI-741-p1-invariant-7-next-action-coverage.zh-CN.md) · [日本語](WI-741-p1-invariant-7-next-action-coverage.ja.md)

# WI-741 — P1 invariant 7 (next-action correctness) coverage

## Intent

Close the sole remaining gap named by
`docs/reference/collaboration-invariant-coverage.md`: invariant 7,
"displayed next step matches current Runtime state/policy," had no test
that asserted the rendered recovery/next-action text against an
independently recorded expected value. A general oracle over every Runtime
state is intractable, so this Work Item takes the bounded approach that
document already recommended: assert the exact next-action text for a
subset of `docs/reference/collaboration-scenario-matrix.json`'s
`sourceType: "observed"` scenarios (SCN-001, SCN-002, SCN-016) against each
scenario's recorded `expected.keyMessage`, per explicit repository-owner
delegation to continue the AI Cockpit collaboration-language initiative.

## Boundary

This is a test-only Work Item. It adds exactly one new file,
`crates/cockpit-repository/tests/scenario_matrix_next_action.rs`. It does
not modify any production source file, any existing test file, or the
collaboration-invariant-coverage/scenario-matrix documents themselves
(updating invariant 7's row to reflect this new coverage is an explicit,
separate follow-on, not part of this delivery, so this Work Item cannot be
read as silently rewriting its own predecessor's claims).

## Acceptance and lifecycle

- `crates/cockpit-repository/tests/scenario_matrix_next_action.rs` asserts
  the exact (SCN-001, SCN-016) or substring (SCN-002) next-action text
  produced by direct `cockpit_repository` library calls matches each
  scenario's recorded `expected.keyMessage` in
  `docs/reference/collaboration-scenario-matrix.json`.
- A fourth guard test in the same file confirms SCN-001, SCN-002, and
  SCN-016 remain declared `sourceType: "observed"` and continue to exercise
  invariant 7, so this test cannot silently drift away from the document it
  is meant to keep honest.
- `cargo test -p cockpit-repository --test scenario_matrix_next_action`,
  `cargo fmt --check`, and `cargo clippy --tests -- -D warnings` all pass.
- `start → preflight → checkpoint → verify → finish → archive → close` is the
  governed route; `user_visible_benefit_not_declared` remains explicit.

## Evidence

- archive: `.ai/work-items/archive/WI-741-p1-invariant-7-next-action-coverage.contract.json`
- verification: `.ai/evidence/WI-741-p1-invariant-7-next-action-coverage.verification.json`
- finalization: `.ai/decisions/WI-741-p1-invariant-7-next-action-coverage.finalize.json` (pending merge)
- close: `.ai/decisions/WI-741-p1-invariant-7-next-action-coverage.close.json` (pending merge)
