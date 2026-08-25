---
author: AI Cockpit maintainers
title: "WI-280 — reference Contract semantics successor"
workItemId: WI-280-reference-contract-semantics-successor
description: "reference parity fields の strict Rust Contract validation と fail-closed preflight review。"
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

WI-280 は不変の WI-279 predecessor を新しい snapshot で継続します。
scenario coverage、acceptance criteria、concurrency boundary を strict typed projection
として検証し、malformed declaration は fail-closed にします。Rust mapping の文書も
三言語で同期します。
