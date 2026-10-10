---
author: AI Cockpit maintainers
title: "Collaboration scenario matrix"
description: "A state/transition-derived scenario matrix for collaboration-language moments, with real observed and documented cases distinguished from designed ones."
audience: [adopter, contributor, maintainer, reviewer]
status: current
authority: canonical
lastVerifiedBy: WI-781-trust-diagnostics
---

# Collaboration scenario matrix

This page is the readable companion to
[`collaboration-scenario-matrix.json`](collaboration-scenario-matrix.json),
which is the structured source of truth for automated consumption. It extends
the current [collaboration language contract](collaboration-language-contract.md)
with concrete scenarios grounded in the repository's supported lifecycle,
evidence, authorization, composition, and operation semantics.

## How to read this matrix

Each scenario names:

- **sourceType** — `observed` (executed in this repository on 2026-09-08
  during WI-679/WI-680 delivery, cited with the exact command/output),
  `documented` (restates an existing canonical doc without new execution), or
  `designed` (synthetic, built to cover a required category with no
  available real record; never a claim about proven real-world behavior).
- **category** — lifecycle transition, evidence, authorization,
  verification, merge/close, historical query, Agent/session hand-off, or
  multi-language/entry-point consistency, composition execution/retry,
  process supervision, or legacy composition.
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
| Verification | SCN-013..015, SCN-025 | 3 | 1 | 0 |
| Merge/close | SCN-016..019 | 3 | 1 | 0 |
| Historical query | SCN-020 | 0 | 1 | 0 |
| Agent/session hand-off | SCN-021..022, SCN-026 | 2 | 1 | 0 |
| Multi-language/entry-point | SCN-023..024 | 0 | 2 | 0 |
| Finalization observation | SCN-027..033 | 6 | 0 | 0 |
| Composition execution | SCN-034 | 0 | 1 | 0 |
| Composition retry | SCN-035..036 | 0 | 2 | 0 |
| Process supervision | SCN-037 | 0 | 1 | 0 |
| Legacy composition | SCN-038 | 0 | 1 | 0 |
| Verification retry boundary | SCN-039 | 0 | 1 | 0 |

SCN-033 is unavailable and is reported separately; it is not included in the
observed, documented, or designed counts above.

SCN-034 through SCN-039 are `documented` cases based on Contract A12-A14 and
the current implementation. They are not claims that those outcomes were
observed in this cloud run. No new `designed` case is added.

## Full matrix

See `collaboration-scenario-matrix.json` for the complete, structured
39-scenario table (SCN-001 through SCN-039; IDs are stable and may be
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
| SCN-025 | verification | Displayed option, selected test-data decision, interruption, and resumed Runtime transition remain consistent | observed | consistent_with_safe_retry |
| SCN-026 | handoff | A fresh subprocess reconstructs the handoff from Runtime records without conversation history | observed | state_fully_recoverable_with_explicit_block |
| SCN-034 | composition_execution | Passed execution with deferred cleanup remains unknown and non-reusable | documented | nonterminal_unknown |
| SCN-036 | composition_retry | Only the bound Linux system no-op gets a fresh deferred-cleanup retry | documented | fresh_attempt_old_tree_preserved |
| SCN-037 | process_supervision | Linux ECHILD proof is not claimed by Unix/Windows process-group backends | documented | backend_specific |
| SCN-038 | legacy_composition | Legacy v1/v2 booleans do not establish reusable v3 evidence | documented | compatibility_only |
| SCN-039 | verification | A Cargo first run is not a supported deferred-cleanup repeat | documented | retry_blocked_before_spawn |

## Known limitations

This matrix remains a hand-curated semantic index rather than a complete
automated test suite. Its JSON source now includes an executable check registry:
each bound scenario names its input facts, expected semantics, and test entry
point. WI-781 adds direct human-report semantic parity, finalization-action
classification, bounded Outcome assembly retry, and staged Runtime diagnostics;
unsupported process counts remain explicitly unavailable. None of these checks
claims exhaustive Cartesian coverage of every invariant or state; future Work
Items may add bounded checks without renumbering scenarios.

The new composition cases are documented boundaries, not a hosted acceptance
receipt. A Cargo command may run as an admitted first execution, but the current
deferred-cleanup retry policy admits only a single protected system no-op
`true`; it does not establish retry support for Cargo or arbitrary tests.
The projection does not replace the Work Item `finish` gates.
