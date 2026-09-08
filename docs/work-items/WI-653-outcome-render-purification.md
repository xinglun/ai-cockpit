---
author: AI Cockpit maintainers
title: WI-653 — Purify the Outcome render layer
description: render_human_outcome no longer reads or validates repository facts; a new use-case assembles them once.
workItemId: WI-653-outcome-render-purification
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-653-outcome-render-purification
terminalArchive: .ai/work-items/archive/WI-653-outcome-render-purification.contract.json
terminalVerification: .ai/evidence/WI-653-outcome-render-purification.verification.json
terminalFinalization: .ai/decisions/WI-653-outcome-render-purification.finalize.json
terminalDecision: .ai/decisions/WI-653-outcome-render-purification.close.json
---

# WI-653 — Purify the Outcome render layer

This Work Item is P1-A of the AI Cockpit architecture initiative: it makes
`crates/cockpit-repository/src/outcome_render.rs` a pure presentation layer
over already-observed, already-validated facts.

## Problem found

`render_human_outcome(root: &Path, outcome: &OutcomeV2, language: &str)`
took a repository root and, inside the render path, performed its own
repository I/O and governance validation:

- It checked whether the archived Contract file exists and called
  `close_decision_is_valid_for_status` (a governance check reading
  `.ai/decisions/{id}.close.json`) to derive `archived_unclosed`.
- It called `load_human_decision(root, work_item_id)`, which reads
  `.ai/decisions/{id}.close.json` again and validates repository-id binding,
  record state, decision confirmation, and every structured-decision field —
  full governance validation logic embedded inside a text-formatting
  function.

All four existing call sites (`crates/cockpit-cli/src/main.rs`:
`WorkItemCommand::Outcome`, `print_lifecycle_result`,
`emit_blocked_lifecycle_handoff`; `crates/cockpit-mcp/src/lib.rs`:
`work_item_outcome`) already call `outcome_v2_with_runtime` immediately
before calling `render_human_outcome` — the redundant read was structural
across every caller, not incidental to one.

## Change

- New `OutcomeRenderInput { outcome: OutcomeV2, human_decision:
  HumanDecisionProjection, archived_unclosed: bool }` and two assembly
  functions, `outcome_render_input`/`outcome_render_input_with_runtime`,
  that call `outcome_v2`/`outcome_v2_with_runtime` plus the existing
  `close_decision_is_valid_for_status`/`load_human_decision` logic exactly
  once each.
- `render_human_outcome` now takes `&OutcomeRenderInput` and a language code
  only — no `root: &Path`, no filesystem access, no governance validation
  call anywhere in its body.
- `HumanDecisionProjection` (previously a private enum local to the render
  module) is now `pub`, since it is a field of the public
  `OutcomeRenderInput`.
- All four call sites migrated to `outcome_render_input(_with_runtime)` then
  `render_human_outcome(&input, language)`; the `--json`/machine-JSON
  branches use `input.outcome` (the identical `OutcomeV2` value) so JSON
  output is unaffected.
- Six existing tests across `cockpit-repository` and `cockpit-mcp` that
  called the old two-argument-plus-root signature were updated to the new
  assemble-then-render flow, keeping their original assertions on the
  resulting text and on `OutcomeV2` fields.

No wording, localization string, JSON schema, or governance decision logic
changed.

## Correctness evidence

- Five new unit tests (`crates/cockpit-repository/src/outcome_render.rs`,
  `#[cfg(test)] mod render_tests`) construct `OutcomeRenderInput` entirely in
  memory — no temp directory, no filesystem — covering: missing human
  decision, valid human decision, invalid/malformed human decision,
  archived-but-unclosed, and historical/superseded. This directly
  demonstrates the render function needs no repository access to test.
- All six updated integration tests (in `cockpit-repository`'s
  `recovery_decision.rs`, `archive_integrity.rs`, `evidence_assurance.rs`,
  `status_projection.rs`, `recovery_events.rs`, and `cockpit-mcp`'s
  `rpc.rs`) pass unchanged in their assertions on rendered text (e.g.
  `"Outcome: 🟡"` prefixes, `"provider finalization"` recovery text,
  `"决定: continue"` human-decision text) — proving byte-for-byte output
  parity between the old and new code paths for every case those tests
  exercise.
- `cargo test --locked --workspace` passes (120+ test result blocks across
  the workspace, 0 failures).

## Verification

`cargo fmt --all -- --check` and `cargo clippy --locked --workspace
--all-targets --all-features -- -D warnings` pass. No CLI/MCP JSON output,
exit code, or governance semantics changed — only the internal Rust
call path that produces the human-readable handoff text.

## Out of scope

P0 responsibility/dependency mapping (separate Work Item), P1-B observation
context, P2 lifecycle/storage extraction, P2-B state types, P2-C multi-file
consistency, and P3 physical-execution boundary are not addressed here.
