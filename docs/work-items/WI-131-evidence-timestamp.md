---
author: AI Cockpit maintainers
workItemId: WI-131-evidence-timestamp
title: Fail-closed verification evidence timestamps
description: Reject malformed RFC3339 timestamps in verification and retention metadata before Outcome or lifecycle completion.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-133-docs-truth
---

# WI-131 — Fail-closed verification evidence timestamps

## Intent

Verification evidence is an auditable record. A syntactically present but
malformed timestamp must not be accepted as current evidence or allow a
lifecycle transition to look green.

## Boundaries

- Validate the v2 envelope `createdAt` and retention `createdAt`/`expiresAt` as
  RFC3339 timestamps.
- Reuse the existing evidence validator for Outcome, finish, archive, and close.
- Preserve historical bytes; legacy evidence remains historical yellow until a
  fresh verification creates a v2 record.
- Do not translate Contract source text or change retention policy semantics.

## Acceptance

- Valid v2 timestamps remain green when all other identity and digest checks pass.
- Missing, malformed, or semantically invalid timestamps never produce a green
  Outcome and block finish/archive/close.
- Repository and CLI regressions cover tampering, archived close, and legacy
  evidence behavior.
- English, Simplified Chinese, and Japanese Outcome documentation state the
  timestamp validation boundary.

## Verification

See the archived Contract, verification evidence, close decision, and Runtime
evidence for focused repository/CLI tests, workspace checks, and documentation
acceptance: `.ai/evidence/WI-131-evidence-timestamp.verification.json` and
`.ai/decisions/WI-131-evidence-timestamp.close.json`.
