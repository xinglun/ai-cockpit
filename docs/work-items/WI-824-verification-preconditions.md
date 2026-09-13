---
author: AI Cockpit maintainers
title: "WI-824 — verification preconditions and durable attempts"
description: "Reject invalid verification inputs before spawning work and preserve bounded execution attempts for safe recovery."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-824-verification-preconditions
lastVerifiedBy: WI-824-verification-preconditions
terminalArchive: .ai/work-items/archive/WI-824-verification-preconditions.contract.json
terminalVerification: .ai/evidence/WI-824-verification-preconditions.verification.json
terminalDecision: .ai/decisions/WI-824-verification-preconditions.close.json
---

[简体中文](WI-824-verification-preconditions.zh-CN.md) · [日本語](WI-824-verification-preconditions.ja.md)

# WI-824 — verification preconditions and durable attempts

## Intent and boundary

This Work Item makes verification execution fail early when a current
preflight, Contract, or repository identity is not usable. It also preserves
each executed node's bounded output, exit code, timeout state, elapsed time,
command identity, Runtime identity, and source snapshot independently from the
formal completion receipt. Historical evidence is not rewritten.

## Recovery behavior

A successful attempt is reusable only when the source snapshot, command and
dependency inputs, Runtime digest, repository identity, and Contract execution
scope still match. Governance-only correction can therefore avoid rerunning an
unchanged command, while source, command, Runtime, or relevant dependency
changes invalidate reuse. Precondition rejection records zero spawned project
processes and a structured diagnostic.

## Acceptance evidence

- Archive: `.ai/work-items/archive/WI-824-verification-preconditions.contract.json`
- Formal verification: `.ai/evidence/WI-824-verification-preconditions.verification.json`
- Attempt records: `.ai/evidence/WI-824-verification-preconditions.verification-attempt.*.json`
- Close decision: `.ai/decisions/WI-824-verification-preconditions.close.json`
