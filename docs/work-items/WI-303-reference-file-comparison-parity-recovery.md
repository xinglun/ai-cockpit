---
author: AI Cockpit maintainers
title: "WI-303 — reference-file comparison parity recovery"
workItemId: WI-303-reference-file-comparison-parity-recovery
description: "Recover the missing three-language parity registration for the immutable WI-302 comparison without rewriting predecessor records."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-303-reference-file-comparison-parity-recovery
terminalArchive: .ai/work-items/archive/WI-303-reference-file-comparison-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-303-reference-file-comparison-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-303-reference-file-comparison-parity-recovery.finalize.json
terminalDecision: .ai/decisions/WI-303-reference-file-comparison-parity-recovery.close.json
authority: canonical
---

# WI-303 — reference-file comparison parity recovery

## Intent

WI-302 is an immutable merged comparison delivery whose pending parity bridge
could not be promoted after merge without violating the finalization append
boundary. This successor records the recovery decision, promotes WI-302 to a
truthful recovered row in all three parity projections, and removes the stale
pending registry entry.

## Scope and boundary

The change is limited to `docs/reference/reference-parity*`, the typed pending
parity registry, and these three readable Work Item projections. All WI-302
archive, verification, finalization, recovery, and merge-observation bytes stay
unchanged. No Runtime, CLI, CI, release, adopter, or global Agent/MCP behavior
is changed.

## Acceptance and verification

- The three parity documents contain one recovered WI-302 row with immutable
  predecessor and recovery evidence, and one pre-archive WI-303 row.
- The pending registry is empty after the atomic recovery projection.
- The installed Runtime lifecycle and documentation/governance integrity gates
  pass with a fresh repository-bound verification receipt.
- Hosted checks pass before merge; finalization, exact cleanup, and close bind
  the reviewed successor.
