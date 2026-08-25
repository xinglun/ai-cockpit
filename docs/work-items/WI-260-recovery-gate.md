---
author: AI Cockpit maintainers
title: "WI-260 — Recovery-aware governance gate"
workItemId: WI-260-recovery-gate
description: "Make immutable predecessor recovery converge in governance inventory and documentation promotion."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-260-recovery-gate
authority: canonical
---

# WI-260 — Recovery-aware governance gate

## Intent

Ensure an immutable predecessor with a valid recovery receipt is projected as
`recovered`, even when its historical close is non-canonical, and ensure normal
documentation promotion remains reserved for an approved close.

## Scope

This Work Item updates the governance-integrity inventory, the closed-Work-Item
documentation promotion helper, and their regressions. It adds no Runtime
lifecycle behavior and never rewrites WI-258's historical close bytes.

## Acceptance

- A valid recovery plus an invalid historical close projects `recovered` without
  an `invalid_terminal_decision` finding.
- A valid approved close still takes precedence over an older recovery receipt.
- Promotion skips only a valid recovered predecessor and fails closed for an
  invalid recovery receipt.
- A retry recovery may omit `successorWorkItemId`; successor/supersede decisions
  still require an explicit successor binding.
- An unambiguous abbreviated Git revision is resolved to one exact commit for
  finalization binding; ambiguous or invalid revisions remain fail-closed.
- Regression tests cover both gate and promotion behavior.
- The tri-language Work Item and parity rows bind the corrective evidence.

## Evidence boundary

Recovery is a historical terminal projection, not a green completion claim.
The successor Work Item owns any future implementation promotion; the
predecessor's original bytes remain immutable and auditable.
