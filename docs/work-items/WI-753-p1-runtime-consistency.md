---
author: AI Cockpit maintainers
title: "WI-753 — P1 controlled Runtime consistency through interruption and resume"
description: "Adds an executable controlled-repository check for displayed state, selected option, and Runtime behavior, including safe interruption and resume."
audience: [maintainer, reviewer, adopter]
workItemId: WI-753-p1-runtime-consistency
status: in_progress
authority: authorized
lastVerifiedBy: WI-753-p1-runtime-consistency
---

[简体中文](WI-753-p1-runtime-consistency.zh-CN.md) · [日本語](WI-753-p1-runtime-consistency.ja.md)

# WI-753 — P1 controlled Runtime consistency through interruption and resume

## Intent

Deliver Section IV of the collaboration-language initiative: prove in a
temporary controlled repository that the Runtime's displayed state and
available option agree with the selected option and the transition the Runtime
actually permits, including an interrupted verification and a safe retry.

## Boundary

This Work Item changes one CLI integration test and the tri-language
collaboration reference pages. Production behavior, Outcome schema, external
participants, and the separately approved P2 onboarding/contribution track are
out of scope. The synthetic human choice is explicitly `TEST DATA ONLY` and is
written only under the temporary fixture; it cannot become an authorization
record in this repository.

## Acceptance

- `crates/cockpit-cli/tests/collaboration_consistency.rs` asserts the offered
  `confirm_review` option, rejection before selection, checkpoint after the
  selected fixture decision, no verification pass after process interruption,
  and current evidence after resume.
- A changed verification projection invalidates the earlier preflight receipt;
  the test records a fresh fixture decision before asserting the current
  preflight state is `human_decision_recorded`.
- The scenario matrix records this observed check as SCN-025 and the
  invariant-coverage pages update invariant 7 without claiming exhaustive
  state-space coverage.
- `cargo fmt --check`, the focused test, `cargo clippy --tests -- -D warnings`,
  and the repository documentation acceptance checks pass.

## Evidence

- archive: `.ai/work-items/archive/WI-753-p1-runtime-consistency.contract.json`
- verification: `.ai/evidence/WI-753-p1-runtime-consistency.verification.json`
- finalization: `.ai/decisions/WI-753-p1-runtime-consistency.finalize.json`
- close: `.ai/decisions/WI-753-p1-runtime-consistency.close.json`
