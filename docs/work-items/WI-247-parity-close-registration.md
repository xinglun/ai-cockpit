---
author: AI Cockpit maintainers
title: "WI-247 — WI-246 close parity registration"
workItemId: WI-247-parity-close-registration
description: "Project the immutable WI-246 terminal decision chain into the tri-language parity ledger."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-247-parity-close-registration
authority: canonical
---

# WI-247 — WI-246 close parity registration

WI-246 correctly closed after PR #197 merged, its governance append was
observed, and the exact branch and feature worktree were removed. Persisting
the authoritative close receipt exposed a ledger-ordering gap: all three
parity rows still described WI-246 as `In progress` and named only its
canonical pre-merge receipt. The gate therefore reported the expected
`missing_parity_decision` and `stale_parity_status` findings.

## Recovery boundary

The Runtime-generated recovery receipt binds the exact WI-246 Contract,
Summary, Outcome, Events, finalization chain, and close identities. Those
records, PR #197, and merge commit `98d6575` are immutable. WI-247 changes only
the English, Simplified Chinese, and Japanese parity/Work Item projections; it
does not change Runtime, CI, release, tests, crates, or WI-241.

## Acceptance and verification

Each WI-246 parity row becomes `Implemented` and retains the canonical
pre-merge receipt while adding the sequence-1 merge observation, sequence-2
cleanup, close, and recovery paths. Focused parity, governance, manifest, and
documentation checks plus the canonical strict repository runner must pass.
The real draft PR #198 is bound before verification, and Runtime lifecycle
records remain separate from the documentation change.
