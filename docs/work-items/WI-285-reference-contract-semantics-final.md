---
author: AI Cockpit maintainers
title: "WI-285 — reference Contract semantics final recovery"
workItemId: WI-285-reference-contract-semantics-final
description: "Complete the bounded Rust Contract-semantics parity batch after prearchive documentation recovery."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-285-reference-contract-semantics-final
terminalArchive: .ai/work-items/archive/WI-285-reference-contract-semantics-final.contract.json
terminalVerification: .ai/evidence/WI-285-reference-contract-semantics-final.verification.json
terminalFinalization: .ai/decisions/WI-285-reference-contract-semantics-final.finalize.1086a11e42dc74baa147fc16496e4b294c5ffb4c59f44b01385ef6f7ec684aca.json
terminalDecision: .ai/decisions/WI-285-reference-contract-semantics-final.close.json
authority: canonical
---

# WI-285 — reference Contract semantics final recovery

WI-285 is the explicit successor of immutable WI-284. It preserves every
predecessor Contract, evidence, archive, and recovery byte. Hosted quality
found that historical WI-281 documentation promotion and predecessor status
updates were missing after WI-284 was archived; this successor completes the
same bounded batch with those promotions present before verification.

Acceptance requires the Rust Contract scenario implementation and tests, all
tri-language parity/documentation bindings, a full workspace verification on
the current default branch, a reviewed hosted PR, and immutable recovery
links. No unrelated CI, release, planner, or global adapter changes are in
scope.
