---
author: AI Cockpit maintainers
title: "WI-277 — capability profile parity recovery"
workItemId: WI-277-capability-profile-parity-recovery
description: "Restore hosted parity registration and prove capability-profile inheritance for adopter repositories after the WI-276 recovery."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-277-capability-profile-parity-recovery
terminalArchive: .ai/work-items/archive/WI-277-capability-profile-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-277-capability-profile-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-277-capability-profile-parity-recovery.finalize.26a5046378afcc467b75e703bb6b7dd83d53f665d76605695f7f28a6b9b8f564.json
terminalDecision: .ai/decisions/WI-277-capability-profile-parity-recovery.close.json
authority: canonical
---

# WI-277 — capability profile parity recovery

## Intent

Restore the tri-language reference-parity registration omitted from the
predecessor delivery and prove that strict capability/profile declarations are
available through repository-bound CLI and MCP projections.

## Scope

- Preserve the immutable WI-276 recovery linkage.
- Register the English, Japanese, and Chinese parity rows before verification.
- Verify two-repository isolation, malformed/stale declaration rejection, and
  read-only adopter projections.
- Bind the reviewed PR, merge observation, exact cleanup, and close decision.

## Boundary

This Work Item does not rewrite WI-276 archive or evidence bytes, add new
capability semantics, change global Agent/MCP configuration, or perform the
later architecture cleanup.

## Acceptance and verification

- Rust workspace quality, documentation, conformance, and governance gates
  pass in one bounded Runtime execution.
- Hosted quality, Windows Runtime, and V1 behavioral-oracle checks pass.
- The merged PR and exact branch/worktree cleanup are recorded by the Runtime.

