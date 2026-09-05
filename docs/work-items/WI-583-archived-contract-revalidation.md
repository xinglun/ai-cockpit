---
author: AI Cockpit maintainers
title: "WI-583 — Archived Contract revalidation and successor close"
description: "Add an append-only, evidence-bound recovery path for archived Work Items whose Contract changed after historical verification."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-583-archived-contract-revalidation
lastVerifiedBy: WI-583-archived-contract-revalidation
terminalArchive: .ai/work-items/archive/WI-583-archived-contract-revalidation.contract.json
terminalVerification: .ai/evidence/WI-583-archived-contract-revalidation.verification.json
terminalFinalization: .ai/decisions/WI-583-archived-contract-revalidation.finalize.json
terminalDecision: .ai/decisions/WI-583-archived-contract-revalidation.close.json
---

[简体中文](WI-583-archived-contract-revalidation.zh-CN.md) · [日本語](WI-583-archived-contract-revalidation.ja.md)

# WI-583 — Archived Contract revalidation and successor close

## Objective

Provide a supported recovery path when an archived Work Item's Contract was
legitimately amended after its historical verification. The path preserves the
original archive and evidence bytes, records current revalidation as a
successor, and permits an explicit human-authorized close without inventing a
provider result.

## Boundary

The Runtime and its repository-bound CLI are in scope. The object repository
`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` is an
external read-only adopter and is not modified by this Work Item. Source
template wire formats, release publication, CI policy redesign, and provider
operations remain out of scope.

## Design

`work-item revalidate-archived --repo <repository> --id <predecessor> --successor <successor>`
creates an append-only recovery decision after validating the archived
Contract, archive manifest, and historical verification evidence. The Runtime
binds the current Contract digest, predecessor evidence digest, repository
identity, archive manifest, and human authority. It then scaffolds the
successor; the successor must complete the normal lifecycle and fresh
verification before the predecessor can be closed.

Historical evidence is never rewritten or promoted to current green evidence.
The final predecessor close records both identities and the human decision.
Missing, malformed, stale, foreign, symlinked, or contradictory evidence
remains fail-closed.

## Acceptance

1. A regression fixture covers a Contract amended after archive while the
   original verification evidence remains immutable.
2. A supported command creates and verifies a successor revalidation record
   while the predecessor is still pending close.
3. The successor completes `start → preflight → checkpoint → verify → finish →
   archive → finalize → finalize-verify → close` with explicit human
   authorization.
4. The predecessor closes only after the successor is valid and records the
   historical/current evidence distinction and lineage.
5. Tampered, missing, malformed, foreign, stale, and symlinked evidence is
   rejected without repository writes.
6. English, Simplified Chinese, and Japanese command/workflow documentation
   describe the append-only and historical-evidence boundaries.

## Verification

The Contract declares the focused Rust protocol/repository/CLI tests, complete
workspace tests, formatting, clippy, and the repository's documented quality
gates. Evidence is generated only by the installed Runtime with an explicit
repository context.
