---
author: AI Cockpit maintainers
workItemId: WI-130-status-closed-projection
title: Closed Work Item status projection
description: Project a valid repository-bound close decision as the terminal status without rewriting archive truth.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-133-docs-truth
---

# WI-130 — Closed Work Item status projection

## Intent

The Runtime writes a structured close decision, but the read-only status
projection previously read only the archived Summary and could continue to show
`finish_ready` after a successful close. This Work Item makes `archived` and
`closed` distinct, using the close decision only as a validated projection.

## Boundaries

- Preserve archived Contract, Summary, Outcome, and manifest bytes.
- Require Work Item identity, closed state, confirmed decision state, and a
  strict structured human decision before projecting `closed`.
- Keep invalid or missing decisions visible as unknowns; never infer closure
  from file existence.

## Acceptance

- `work-item status` and the repository projection report `archived` after
  archive and `closed` only after a valid close decision.
- CLI and repository regression tests cover valid, missing, malformed, foreign,
  and invalid close records.
- English, Simplified Chinese, and Japanese Outcome documentation describe the
  terminal projection boundary.

## Verification

See the archived Contract, verification evidence, close decision, and Runtime
evidence for the focused repository/CLI tests, workspace checks, and
documentation acceptance: `.ai/evidence/WI-130-status-closed-projection.verification.json`
and `.ai/decisions/WI-130-status-closed-projection.close.json`.
