---
author: AI Cockpit maintainers
title: WI-417 — deterministic Cargo verification scaffold selection
description: Select an executable default Cargo verification command from repository facts.
workItemId: WI-417-cargo-verification-lock-strategy
audience: [adopter, contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-417-cargo-verification-lock-strategy
terminalArchive: .ai/work-items/archive/WI-417-cargo-verification-lock-strategy.contract.json
terminalVerification: .ai/evidence/WI-417-cargo-verification-lock-strategy.verification.json
terminalFinalization: .ai/decisions/WI-417-cargo-verification-lock-strategy.finalize.json
terminalDecision: .ai/decisions/WI-417-cargo-verification-lock-strategy.close.json
---

# WI-417 — deterministic Cargo verification scaffold selection

[简体中文](WI-417-cargo-verification-lock-strategy.zh-CN.md) · [日本語](WI-417-cargo-verification-lock-strategy.ja.md)

## Intent

Make the default verification command emitted when a Cargo Work Item scaffold is
activated executable for the repository. A tracked `Cargo.lock` selects
`cargo test --locked --workspace`; a Cargo repository without a lockfile selects
`cargo test --workspace`; non-Cargo repositories receive no invented Cargo command.

## Scope and boundary

The same deterministic rule is used by `start` and recovery scaffold activation.
This Work Item changes command selection and its reference documentation only;
it does not change verification semantics, release/adopter harnesses, Sentinel
source, or global Agent/MCP configuration.

## Evidence

- Archive: `.ai/work-items/archive/WI-417-cargo-verification-lock-strategy.contract.json`
- Verification: `.ai/evidence/WI-417-cargo-verification-lock-strategy.verification.json`
- Finalization: `.ai/decisions/WI-417-cargo-verification-lock-strategy.finalize.json`
- Close: `.ai/decisions/WI-417-cargo-verification-lock-strategy.close.json`
- Reviewed PR: [#382](https://github.com/xinglun/ai-cockpit/pull/382)

Targeted lockfile, lockless, and non-Cargo tests passed, followed by the full
locked workspace test suite under Runtime v0.2.43.
