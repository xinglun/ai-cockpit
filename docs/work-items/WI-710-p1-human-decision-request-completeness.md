---
author: AI Cockpit maintainers
title: "WI-710 — P1 human decision request completeness test"
description: "Closes the invariant-9 gap named by WI-681: extends two existing preflight tests to assert every HumanDecisionRequest field is non-empty."
audience: [maintainer, reviewer, adopter]
workItemId: WI-710-p1-human-decision-request-completeness
status: implemented
authority: authorized
lastVerifiedBy: WI-710-p1-human-decision-request-completeness
terminalArchive: .ai/work-items/archive/WI-710-p1-human-decision-request-completeness.contract.json
terminalVerification: .ai/evidence/WI-710-p1-human-decision-request-completeness.verification.json
terminalFinalization: .ai/decisions/WI-710-p1-human-decision-request-completeness.finalize.json
terminalDecision: .ai/decisions/WI-710-p1-human-decision-request-completeness.close.json
---

[简体中文](WI-710-p1-human-decision-request-completeness.zh-CN.md) · [日本語](WI-710-p1-human-decision-request-completeness.ja.md)

# WI-710 — P1 human decision request completeness test

## Intent

Close the invariant-9 gap named in WI-681's coverage mapping
(`docs/reference/collaboration-invariant-coverage.md`, pending merge):
"every question that requires a human decision must name the decision
subject, its impact, and its recovery/resume condition" had no dedicated
assertion covering every `HumanDecisionRequest` field. This Work Item
extends the two existing tests in
`crates/cockpit-repository/tests/contract_preflight.rs` that already reach
`needs_human_confirmation` (an empty scaffold, and a high-risk
scenario-coverage gate), adding one shared assertion helper rather than a
new test file or new production code, per explicit repository-owner
delegation to continue the AI Cockpit collaboration-language initiative.

## Boundary

This is a test-only Work Item. It modifies exactly one existing file,
`crates/cockpit-repository/tests/contract_preflight.rs`, adding one helper
function and two call sites. It does not modify any production source file.

## Acceptance and lifecycle

- The new helper asserts `what_happened`, `why_it_matters`, `question`,
  `resume_condition`, `options`, `recommended_option`, and
  `recommendation_reason` are non-empty, that `recommended_option` names one
  of the offered options, and that every option's `id`/`label`/`effect` are
  non-empty.
- It is exercised by both existing `needs_human_confirmation` tests,
  covering two independently-triggered real scenarios.
- `cargo test --locked -p cockpit-repository --test contract_preflight`
  (7/7), `cargo fmt --check`, and `cargo clippy --tests -- -D warnings` all
  pass.
- `start → preflight → checkpoint → verify → finish → archive → close` is the
  governed route; `user_visible_benefit_not_declared` remains explicit.

## Evidence

- archive: `.ai/work-items/archive/WI-710-p1-human-decision-request-completeness.contract.json`
- verification: `.ai/evidence/WI-710-p1-human-decision-request-completeness.verification.json`
- finalization: `.ai/decisions/WI-710-p1-human-decision-request-completeness.finalize.json` (pending merge)
- close: `.ai/decisions/WI-710-p1-human-decision-request-completeness.close.json` (pending merge)
