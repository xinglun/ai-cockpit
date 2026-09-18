---
author: AI Cockpit maintainers
workItemId: WI-865-four-direction-convergence
title: "Four-direction convergence acceptance"
description: "Evidence-led acceptance report for performance, Outcome, HCI, and architecture convergence."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-865-four-direction-convergence
terminalArchive: .ai/work-items/archive/WI-865-four-direction-convergence.contract.json
terminalVerification: .ai/evidence/WI-865-four-direction-convergence.verification.json
terminalDecision: .ai/decisions/WI-865-four-direction-convergence.close.json
---

# WI-865 — Four-direction convergence acceptance

## Scope and decision boundary

This report records the four bounded work packages from the WI-865 Contract.
It does not grant merge or release authorization. Publication remains a
separate, explicitly authorized Work Item after reviewed merge.

## Acceptance evidence

- **Performance (P):** release-grade statistics and the P0 comparator reject 99
  valid warm samples, accept 100, preserve raw samples, and expose unavailable
  counters as reasons. The portable collector covers Runtime inspect/status/
  Outcome/verification planning and diagnose; paired baseline/candidate JSON
  is linked from the Work Item evidence after collection.
- **Outcome (O):** lifecycle, CLI, MCP, summary, and full rendering consume the
  Runtime-bound observation assembly. Release facts are optional and require an
  explicit evidence-bound projection; changed paths remain audit detail.
- **HCI (C):** the tri-language first-work-item route starts with
  `start --prepare`, distinguishes reviewable, mergeable, and closed, and keeps
  provider/release detail in the detailed workflow reference.
- **Architecture (A):** observation, lifecycle, verification, projection, and
  adapter ownership is documented in the responsibility map and guarded by the
  focused Outcome/lifecycle tests.

## Current status

The final table below is filled only from captured Runtime receipts, hosted PR
state, and paired benchmark JSON. Unknown or unavailable fields remain stated
as unknown; no benefit or authorization is inferred from a green check alone.

| Area | Baseline → candidate | Direct evidence | Decision |
|---|---|---|---|
| Runtime and cycle cost | pending paired capture | `.ai/evidence/WI-865-four-direction-convergence/` | pending |
| Outcome consistency | lifecycle and projection tests | `.ai/evidence/WI-865-four-direction-convergence.verification.json` | pending |
| HCI/default path | tri-language semantic and documentation gates | `tests/docs/getting_started_semantic.sh` | pending |
| Architecture | responsibility map plus pure-renderer tests | `docs/reference/architecture-responsibility-map-2026-09.md` | pending |

## Remaining risks

Resident MCP and provider-side publication are external boundaries for this
Work Item. A later release Outcome may populate version, release link,
installation/upgrade acceptance, and cleanup only when those immutable records
are supplied with their own evidence references.
