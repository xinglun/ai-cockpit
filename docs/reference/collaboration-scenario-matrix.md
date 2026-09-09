---
author: AI Cockpit maintainers
title: "Collaboration scenario matrix"
description: "A state/transition-derived scenario matrix for collaboration-language moments, with real observed and documented cases distinguished from designed ones."
audience: [adopter, contributor, maintainer, reviewer]
status: current
authority: canonical
lastVerifiedBy: WI-680-p1-collaboration-scenario-matrix
---

# Collaboration scenario matrix

This page is the readable companion to
[`collaboration-scenario-matrix.json`](collaboration-scenario-matrix.json),
which is the structured source of truth for automated consumption. It
extends the collaboration language contract delivered by
[WI-679](../work-items/WI-679-p0-collaboration-language-contract.md) (its
`docs/reference/collaboration-language-contract.md` page is not yet on the
default branch as of this Work Item; link here once WI-679 merges)
with concrete scenarios generated from the repository's actual supported
lifecycle, evidence states, authorization states, and operation types, per
the collaboration-language initiative's P1 scope.

## How to read this matrix

Each scenario names:

- **sourceType** — `observed` (executed in this repository on 2026-09-08
  during WI-679/WI-680 delivery, cited with the exact command/output),
  `documented` (restates an existing canonical doc without new execution), or
  `designed` (synthetic, built to cover a required category with no
  available real record; never a claim about proven real-world behavior).
- **category** — lifecycle transition, evidence, authorization,
  verification, merge/close, historical query, Agent/session hand-off, or
  multi-language/entry-point consistency.
- **expected result** and the **semantic invariant(s)** (from the
  collaboration language contract's ten invariants) it exercises.

This matrix prioritizes key boundaries and easily-confused combinations
(per the initiative's own scoping direction) rather than an exhaustive
Cartesian enumeration of every state times every operation.

## Coverage summary

| Category | Scenarios | Observed | Documented | Designed |
| --- | --- | --- | --- | --- |
| Lifecycle transition | SCN-001..005 | 4 | 0 | 0 (1 mixed: SCN-005 is an operational gotcha, not a governance rejection) |
| Evidence | SCN-006..008 | 1 | 2 | 0 |
| Authorization | SCN-009..012 | 1 | 3 | 0 |
| Verification | SCN-013..015 | 2 | 1 | 0 |
| Merge/close | SCN-016..019 | 3 | 1 | 0 |
| Historical query | SCN-020 | 0 | 1 | 0 |
| Agent/session hand-off | SCN-021..022 | 1 | 1 | 0 |
| Multi-language/entry-point | SCN-023..024 | 0 | 2 | 0 |

Most scenarios in this delivery are `observed` or `documented`; none are
`designed`, because the repository's own canonical docs and this delivery's
own real Runtime interactions already covered every required category.

## Full matrix

See `collaboration-scenario-matrix.json` for the complete, structured
25-scenario table (SCN-001 through SCN-024; IDs are stable and may be
extended, never renumbered, by future Work Items). A representative sample:

| ID | Category | Title | Source | Result |
| --- | --- | --- | --- | --- |
| SCN-003 | lifecycle_transition | New Work Item rejected while a predecessor lacks a valid close decision | observed | rejected |
| SCN-006 | evidence | `finish` rejected: verification evidence invalidated after a post-verify Contract change | observed | rejected |
| SCN-013 | verification | `finish` blocked without `acceptanceEvidence`/`intentAlignment` | observed | rejected |
| SCN-014 | verification | `finish` succeeds green while explicitly not authorizing merge | observed | accepted |
| SCN-017 | merge_close | Merge confirmation blocked by a platform (not Runtime) permission boundary | observed | rejected |
| SCN-019 | merge_close | A pre-merge documentation gap is fixed by a fast-follow Work Item, not history rewriting | observed | deferred_to_successor |
| SCN-022 | handoff | One Agent's incomplete predecessor closure structurally blocks another Agent's fresh start | observed | rejected_until_predecessor_closed |

## Known limitations

This matrix is a docs-only, hand-curated artifact. It is not itself an
automated test suite: turning each scenario into an executable check against
a controlled test repository, with positive/negative/boundary cases per the
ten semantic invariants, is explicitly out of scope here and is the next
Work Item in this initiative.
