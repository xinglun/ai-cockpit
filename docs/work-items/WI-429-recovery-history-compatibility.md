---
author: AI Cockpit maintainers
title: "WI-429 — Historical recovery projection"
description: Resolve archived recovery residue without weakening fail-closed validation.
audience: [contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
workItemId: WI-429-recovery-history-compatibility
lastVerifiedBy: WI-429-recovery-history-compatibility
terminalArchive: .ai/work-items/archive/WI-429-recovery-history-compatibility.contract.json
terminalVerification: .ai/evidence/WI-429-recovery-history-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-429-recovery-history-compatibility.finalize.json
terminalDecision: .ai/decisions/WI-429-recovery-history-compatibility.close.json
---

# WI-429 — Historical recovery projection

## Intent and boundary

Archived recovery chains can contain an older successor attempt whose target
was never bound, followed by a valid supersede receipt. Runtime must project
the valid terminal decision without rewriting immutable history.

In scope:

- recognize only narrowly classified historical successor-binding residue;
- let a newer valid `supersede` win by recorded decision time;
- keep malformed, foreign, tampered, or newer-invalid records fail-closed;
- add Rust regression coverage and tri-language workflow/parity documentation.

Out of scope: rewriting historical governance bytes, broad recovery graph
redesign, release/CI routing, or global Agent/MCP configuration.

## Acceptance and evidence

The predecessor must be renderable and closeable when a valid supersede is the
latest trusted recovery decision. Without such a decision, the same residue
must remain a visible failure. Contract, Summary, Outcome, Events, Evidence,
and recovery receipt bytes are preserved exactly.

Verification and terminal receipts will be recorded under `.ai/evidence/` and
`.ai/decisions/` after the reviewed PR is merged.

[中文](WI-429-recovery-history-compatibility.zh-CN.md) · [日本語](WI-429-recovery-history-compatibility.ja.md)
