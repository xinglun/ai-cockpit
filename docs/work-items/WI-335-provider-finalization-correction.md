---
author: AI Cockpit maintainers
title: "WI-335 — Provider finalization correction"
workItemId: WI-335-provider-finalization-correction
description: "Re-deliver WI-334's bounded evidence-parity documentation with the actual reviewed provider identity bound before verification."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-335-provider-finalization-correction
terminalArchive: .ai/work-items/archive/WI-335-provider-finalization-correction.contract.json
terminalVerification: .ai/evidence/WI-335-provider-finalization-correction.verification.json
terminalFinalization: .ai/decisions/WI-335-provider-finalization-correction.finalize.ba4e6148c90ef176ea251397fe70c446779d1a7612d9442fd87db79dc4dee90e.json
terminalDecision: .ai/decisions/WI-335-provider-finalization-correction.close.json
---

# WI-335 — Provider finalization correction

WI-334 is preserved as immutable history. Its archived Contract recorded a
placeholder PR URL before the actual PR identity was known. This successor
does not rewrite that predecessor or add Runtime behavior; it records the
recovery linkage and re-delivers the same bounded evidence-parity
documentation with the real provider PR bound before verification.

## Boundary

- Preserve all WI-334 archive, evidence, and recovery bytes.
- Record the WI-335 provider context only after the reviewed PR exists.
- Re-run the installed Runtime lifecycle and hosted checks.
- Finalize the exact branch/worktree, close with a structured human decision,
  and remove only the exact merged resources.

The Cursor adopter feedback remains external validation input. Stable stdout
JSON, replayable Outcome, lifecycle entry gates, and verification invalidation
are existing Runtime boundaries; automatic IDE chat posting remains the host
Adapter's responsibility.

## Acceptance

1. The WI-334 predecessor bytes and repository identity remain unchanged.
2. The tri-language parity ledger records this recovery without a guessed PR
   URL and links the actual provider PR after it is created.
3. The active Contract binds the actual PR before verification, and every
   finalization receipt matches the installed Runtime and repository.
4. Hosted checks and the complete lifecycle produce auditable evidence, after
   which the exact branch and worktree are cleaned.

[简体中文](WI-335-provider-finalization-correction.zh-CN.md) · [日本語](WI-335-provider-finalization-correction.ja.md)
