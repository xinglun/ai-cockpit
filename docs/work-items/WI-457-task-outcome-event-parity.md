---
author: AI Cockpit maintainers
title: "WI-457 — Task Outcome event semantic parity"
workItemId: WI-457-task-outcome-event-parity
description: "Add the Rust-native append-only Task Outcome event projection and finding/risk fingerprint validation."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-457-task-outcome-event-parity
terminalArchive: .ai/work-items/archive/WI-457-task-outcome-event-parity.contract.json
terminalVerification: .ai/evidence/WI-457-task-outcome-event-parity.verification.json
terminalFinalization: .ai/decisions/WI-457-task-outcome-event-parity.finalize.d2e8f8795a6a88fc3fcd8bf2633813d2e20d0443e4c48397b5bab254b0ba8a70.json
terminalDecision: .ai/decisions/WI-457-task-outcome-event-parity.close.json
---

# WI-457 — Task Outcome event semantic parity

WI-457 adds the repository-bound Rust projection for Task Outcome events. It
keeps the event stream append-only, validates identity and evidence references,
and records deterministic finding/risk fingerprints so a repeated finding is
not silently counted as new progress. The implementation is semantic parity
with the local reference source, not Python wire compatibility.

[简体中文](WI-457-task-outcome-event-parity.zh-CN.md) · [日本語](WI-457-task-outcome-event-parity.ja.md)

## Delivered boundary

- Strict `TaskOutcomeEvent` validation for the reference event families,
  correction/supersession ordering, repository and Work Item identity, safe
  evidence paths, and unknown-field rejection.
- Deterministic `findingFingerprint` values for finding/risk events with
  duplicate rejection unless an explicit correction/supersession is linked.
- Runtime generation of append-only events for Outcome report sections without
  inventing authority, approval, release, provider assurance, or user benefit.
- Byte-preserving archive binding and close-time validation of the event stream.
- Three-language reference and feature documentation describing semantic,
  privacy, localization, and non-wire boundaries.

## Verification evidence

The terminal verification is recorded in
`.ai/evidence/WI-457-task-outcome-event-parity.verification.json` and the
archive/close records bind the same repository, Contract, and Runtime identity.
The finalization history records the reviewed merge observation and exact
feature branch/worktree cleanup; immutable Runtime records are not rewritten.

## Related documentation

- [Task Outcome events](../reference/task-outcome-events.md)
- [Task Outcome report](../features/task-outcome-report.md)
- [Reference parity](../reference/reference-parity.md)
