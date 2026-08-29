---
workItemId: WI-396-status-fast-path-budget
title: "Status fast path and strict performance budget"
author: AI Cockpit maintainers
description: "Remove a measured clean-snapshot subprocess while keeping performance claims identity-bound and fail-closed."
type: implementation
audience: [adopter, contributor, maintainer, reviewer]
authority: human-authorized
status: implemented
lastVerifiedBy: WI-396-status-fast-path-budget
terminalArchive: .ai/work-items/archive/WI-396-status-fast-path-budget.contract.json
terminalVerification: .ai/evidence/WI-396-status-fast-path-budget.verification.json
terminalFinalization: .ai/decisions/WI-396-status-fast-path-budget.finalize.json
terminalDecision: .ai/decisions/WI-396-status-fast-path-budget.close.json
---

# WI-396 — Status fast path and strict performance budget

[简体中文](WI-396-status-fast-path-budget.zh-CN.md) · [日本語](WI-396-status-fast-path-budget.ja.md)

## Intent

Continue the Rust performance convergence after WI-395. A clean repository
snapshot already proves that the equivalent diff is empty, so the Runtime may
skip that redundant Git subprocess. Dirty or uncertain input must retain full
patch inspection and identical governance facts.

## Boundary

The benchmark boundary is the release/installed Runtime on a declared local
platform. Status `<50 ms` and medium observation `<100 ms` remain explicit
targets; a miss is recorded as a bounded gap or failed budget, never hidden by
weakening verification. Runtime and repository identity are always recorded.

The Runtime remains one shared external binary. Adopters bind it with an
explicit `--repo`, and each repository retains isolated `.ai/` state. No global
cache, current repository, provider/enterprise performance claim, or reference
installer/Make/Python/V1 copy is introduced.

## Verification

Run the locked workspace tests, Git snapshot regression tests, performance
fixture, identity-bound regression gate, documentation gates, and `git diff
--check`. The final Work Item evidence must include the measured command,
sample/median, `gitCalls`, Runtime digest, and repository identity.
