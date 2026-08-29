---
author: AI Cockpit maintainers
title: WI-406 — Closed documentation promotion alignment
description: Align closed Work Item documentation promotion with Runtime finalReport evidence bindings.
workItemId: WI-406-close-promotion-alignment
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-406-close-promotion-alignment
terminalArchive: .ai/work-items/archive/WI-406-close-promotion-alignment.contract.json
terminalVerification: .ai/evidence/WI-406-close-promotion-alignment.verification.json
terminalFinalization: .ai/decisions/WI-406-close-promotion-alignment.finalize.json
terminalDecision: .ai/decisions/WI-406-close-promotion-alignment.close.json
---

# WI-406 — Closed documentation promotion alignment

## Intent

Align the closed Work Item documentation promoter with the Runtime's valid
finalReport evidence binding, while keeping malformed or incomplete close
records fail-closed.

## Scope

- Accept a verification reference bound by `finalReport.bindings`.
- Keep structured human-decision references non-empty and auditable.
- Preserve tri-language documentation and parity registration for this Work Item.

## Evidence

- Archive Contract: `.ai/work-items/archive/WI-406-close-promotion-alignment.contract.json`
- Verification: `.ai/evidence/WI-406-close-promotion-alignment.verification.json`
- Pull request: [#371](https://github.com/xinglun/ai-cockpit/pull/371)

## Boundary

This Work Item does not rewrite historical Runtime evidence or change Runtime
lifecycle semantics. Terminal documentation is promoted only after reviewed
merge and close evidence are available.
