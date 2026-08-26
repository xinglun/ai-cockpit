---
author: AI Cockpit maintainers
title: "WI-299 — release adopter finalization base binding"
workItemId: WI-299-release-adopter-finalize-binding
description: "Keep release adopter finalization receipts bound to the archived Work Item Contract base revision."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-299-release-adopter-finalize-binding
terminalArchive: .ai/work-items/archive/WI-299-release-adopter-finalize-binding.contract.json
terminalVerification: .ai/evidence/WI-299-release-adopter-finalize-binding.verification.json
terminalFinalization: .ai/decisions/WI-299-release-adopter-finalize-binding.finalize.json
terminalDecision: .ai/decisions/WI-299-release-adopter-finalize-binding.close.json
authority: canonical
---

# WI-299 — Release adopter finalization base binding

## Intent

The staged v0.2.32 adopter acceptance caught a real fail-closed mismatch: the
harness wrote the post-mutation HEAD into `pullRequest.baseRevision`, although
the Runtime requires that field to equal the archived Contract's base revision.

## Scope

Both release adopter harnesses now read and validate each Work Item Contract's
base revision before mutation. Finalization receipts keep the changed HEAD in
`headRevision` and bind `pullRequest.baseRevision` to the preserved Contract
revision. Static regression checks cover staged and N-1 upgrade paths.

## Boundary

This is a harness and regression-test correction. It does not change Runtime
lifecycle semantics, rewrite v0.2.32 historical bytes, or add another adopter
technology. Existing cleanup, isolation, immutable artifact, and structured
decision checks remain required.

## Verification

- Static adopter and upgrade acceptance tests pass.
- A candidate acceptance run must reach `finalize-verify` and structured close.
- The receipt must distinguish Contract `baseRevision` from mutation
  `headRevision`.
