---
author: AI Cockpit maintainers
title: "WI-480 — finalization context recovery guard"
description: "Reject ambiguous provisional resource contexts before terminal lifecycle steps."
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: human-authorized
lastVerifiedBy: WI-480-finalization-context-recovery
terminalArchive: .ai/work-items/archive/WI-480-finalization-context-recovery.contract.json
terminalVerification: .ai/evidence/WI-480-finalization-context-recovery.verification.json
terminalFinalization: .ai/decisions/WI-480-finalization-context-recovery.finalize.json
terminalDecision: .ai/decisions/WI-480-finalization-context-recovery.close.json
workItemId: WI-480-finalization-context-recovery
---

# WI-480 — finalization context recovery guard

This bounded Runtime change makes the bare `pending` provider sentinel
provisional, matching the existing `pending:<stable-reference>` rule. It keeps
finish and archive fail-closed until an explicit `finalize-plan` binds a real
reviewed resource. Immutable WI-479 records are recovered by the existing
append-only successor path and are not rewritten.

[简体中文](WI-480-finalization-context-recovery.zh-CN.md) · [日本語](WI-480-finalization-context-recovery.ja.md)

## Scope

- classify the exact `pending` sentinel as provisional;
- add protocol and lifecycle regressions for finish rejection and recoverability;
- document the explicit finalization boundary in three languages.

## Out of scope

Release versioning/publication, adopter repositories, foreign-runtime policy,
and rewriting any WI-479 Contract, evidence, archive, outcome, event, or
recovery bytes.

## Verification

- `cargo test --locked -p cockpit-protocol --test resource_finalization`
- `cargo test --locked -p cockpit-repository --test archive_integrity`
- `cargo test --locked --workspace`
- `cargo fmt --all -- --check`
