---
author: AI Cockpit maintainers
title: "WI-756 — P1 Runtime-only handoff reconstruction"
description: "Adds a bounded controlled-repository check that a fresh Agent or session can reconstruct the collaboration handoff without conversation history."
audience: [maintainer, reviewer, adopter]
workItemId: WI-756-p1-handoff-reconstruction
status: recovered
authority: authorized
lastVerifiedBy: WI-756-p1-handoff-reconstruction-revalidation
terminalArchive: .ai/work-items/archive/WI-756-p1-handoff-reconstruction.contract.json
terminalVerification: .ai/evidence/WI-756-p1-handoff-reconstruction.verification.json
terminalFinalization: .ai/decisions/WI-756-p1-handoff-reconstruction.finalize.json
terminalDecision: .ai/decisions/WI-756-p1-handoff-reconstruction.close.json
---

[简体中文](WI-756-p1-handoff-reconstruction.zh-CN.md) · [日本語](WI-756-p1-handoff-reconstruction.ja.md)

# WI-756 — P1 Runtime-only handoff reconstruction

## Intent

Deliver Section V of the collaboration-language initiative: prove in a
temporary controlled repository that a fresh subprocess can reconstruct the
goal, scope, completion state, current evidence bindings, authorization, and
persisted blocking reason without conversation history.

## Boundary

This Work Item changes one CLI integration test and the tri-language
collaboration reference pages. Production behavior, Outcome schema, external
participant recruitment/interviews/evaluation, and the separately approved P2
onboarding/contribution track are out of scope. The test uses only temporary
controlled-repository records and does not create an authorization record in
this source repository.

## Acceptance

- `crates/cockpit-cli/tests/collaboration_handoff.rs::new_agent_reconstructs_handoff_from_runtime_records_without_conversation_history`
  reads Contract, Summary, status, the persisted active Outcome, and current
  verification evidence from a fresh subprocess boundary.
- The test reconstructs goal/scope, done versus pending state, current
  evidence bindings and freshness, authorized authority, an explicit
  `finish.governance` recovery condition, and the unknown user-visible benefit
  without inferring a completion or benefit.
- The scenario matrix records this observed check as SCN-026; the invariant
  coverage and Contract pages describe it as bounded evidence, without
  external participant validation.
- `cargo fmt --check`, the focused test, `cargo clippy --tests -- -D warnings`,
  and the repository documentation acceptance checks pass.

## Evidence

- archive: `.ai/work-items/archive/WI-756-p1-handoff-reconstruction.contract.json`
- verification: `.ai/evidence/WI-756-p1-handoff-reconstruction.verification.json`
- finalization: `.ai/decisions/WI-756-p1-handoff-reconstruction.finalize.json`
- close: `.ai/decisions/WI-756-p1-handoff-reconstruction.close.json`
