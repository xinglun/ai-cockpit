---
author: AI Cockpit maintainers
title: "WI-280 — reference Contract semantics successor"
workItemId: WI-280-reference-contract-semantics-successor
description: "Strict Rust Contract validation for reference-parity fields and fail-closed preflight review."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-280-reference-contract-semantics-successor
terminalArchive: .ai/work-items/archive/WI-280-reference-contract-semantics-successor.archive.json
terminalVerification: .ai/evidence/WI-280-reference-contract-semantics-successor.verification.json
terminalFinalization: .ai/decisions/WI-280-reference-contract-semantics-successor.finalize.json
terminalDecision: pending-reviewed-merge-close
authority: canonical
---

# WI-280 — reference Contract semantics successor

WI-280 continues the immutable WI-279 predecessor through a fresh snapshot.
It validates scenario coverage, acceptance criteria, and concurrency boundaries
with strict typed projections, keeps malformed declarations fail-closed, and
documents the Rust mapping in all three supported languages.
