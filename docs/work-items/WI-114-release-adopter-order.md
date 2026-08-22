---
author: AI Cockpit maintainers
title: "WI-114 release adopter lifecycle ordering"
description: "Make public Release and N-1 acceptance follow the Runtime lifecycle contract."
audience:
  - maintainer
  - reviewer
  - release_operator
status: implemented
authority: canonical
lastVerifiedBy: v0.2.8-adopter-acceptance
capabilityClaims:
  - release_adopter_acceptance
  - fail_closed_lifecycle
---

# WI-114: Release adopter lifecycle ordering

## Goal

Repair the public adopter and N-1 acceptance harnesses so they record
`preflight` before `checkpoint`, as required by the installed Runtime's
fail-closed lifecycle contract.

## Why this Work Item exists

The immutable v0.2.8 Release exposed a harness defect: both acceptance scripts
ran `start → checkpoint → preflight`. Runtime v0.2.8 correctly rejected that
sequence. This Work Item changes only the acceptance harnesses and their
regression checks; it does not rewrite the published Release or its receipt.

## Acceptance

- Public adopter acceptance records `lifecycle-preflight` before
  `lifecycle-checkpoint`.
- N-1 acceptance records `old-preflight` before `old-checkpoint`.
- N-1 acceptance closes the legacy Work Item with the old Runtime, preserves
  its historical evidence, then starts a fresh post-migration Work Item and
  records `new-preflight` → `new-checkpoint` → `new-verify` with v0.2.8.
- Static tests fail if either order regresses.
- Both harnesses remain immutable-public-artifact-only and retain cleanup,
  isolation, checksum, and `first-adopter-smoke=not_ready` assertions.
- A rerun against public v0.2.8 passes without source or workspace fallback.
- The N-1 harness does not fabricate a new lifecycle state for an old summary;
  it closes the old lifecycle before migration and uses the new lifecycle for
  the post-migration Work Item.

## Verification

```text
bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance_test.sh
AI_COCKPIT_RUN_PUBLIC_ACCEPTANCE=1 AI_COCKPIT_ACCEPTANCE_TARGET=aarch64-apple-darwin \
  bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit --from-tag v0.2.7 --to-tag v0.2.8 \
  --target aarch64-apple-darwin --output ./release-adopter-upgrade-acceptance
```

The published v0.2.8 Release remains immutable. Any failed receipt is retained
as failed post-release evidence and cannot be used to claim Release success.
The corrected public adopter and N-1 runs both passed with cleanup receipts;
the N-1 run used the public v0.2.7 → v0.2.8 pair and preserved the old
evidence bytes before starting the post-migration Work Item.

## Outcome

Status: **Corrective harness change; release truth remains immutable.**
