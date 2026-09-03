---
author: AI Cockpit maintainers
title: "WI-521 — Reference guard and adoption-check comparison batch 35"
description: "Compare the next bounded reference scripts one by one and record Rust-native boundaries without copying source tooling."
audience:
  - maintainer
  - reviewer
  - adopter
status: completed
authority: canonical
workItemId: WI-521-reference-file-comparison-batch-35
lastVerifiedBy: WI-521-reference-file-comparison-batch-35
---

# WI-521 — Reference guard and adoption-check comparison batch 35

## Objective

Compare the next bounded set of reference files at pinned commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`, then record an evidence-backed
classification for every current path. The goal is semantic parity and an
explicit adopter boundary, not source Python/Make compatibility.

## File-level result

| Reference path | Decision |
| --- | --- |
| `scripts/ai_check_adoption_ready.py` | `reference-only`: source-specific adoption completeness; Rust onboarding and status/doctor facts keep the checklist external. |
| `scripts/ai_check_archive_recovery.py` | `implemented-different-by-design`: append-only archive and predecessor-bound recovery protect immutable ownership. |
| `scripts/ai_check_backtrack.py` | `implemented-different-by-design`: Rust derives test/coverage weakening and input-trust signals; source report-only deletion warnings remain maintenance projection. |
| `scripts/ai_check_budget_impact.py` | `implemented-different-by-design`: typed identity-bound performance/cost budgets are advisory and never replace required verification. |
| `scripts/ai_check_capability_claims.py` | `reference-only`: source lexical claim/matrix validation is not Runtime authority; Rust capability truth is observed and repository-bound. |
| `scripts/ai_check_coverage_guard.py` | `implemented-different-by-design`: Rust detects weakening and binds declared verification; source association reports remain adopter policy. |
| `scripts/ai_check_dependabot_intake.py` | `not-applicable`: bot event identity and automatic merge are provider-specific. |
| `scripts/ai_check_diff_ownership.py` | `reference-only`: Rust lifecycle scope and archive ownership are authoritative; source cross-Work-Item preview is not copied. |
| `scripts/ai_check_guard_calibration.py` | `implemented-different-by-design`: Rust validates Project Profile and explicit calibration facts. |
| `scripts/ai_check_guards.py` | `implemented-different-by-design`: typed Contract, authority, trust, lifecycle, and isolation boundaries replace source YAML manifests. |
| `tests/test_ai_check_archive_recovery.py` | `implemented-different-by-design`: native archive/finalization tests cover the immutable ownership boundary. |
| `tests/test_ai_check_budget_impact.py` | `implemented-different-by-design`: native verification/performance tests cover typed budget and exact-reuse semantics. |

The retired `tests/test_ai_check_backtrack.py` path was not treated as a
current source file; its historical record remains in the append-only ledger.

## Acceptance

- Every selected current path was read from the pinned local checkout and is
  classified in `tests/conformance/reference_file_inventory.json`.
- The inventory regression asserts all twelve records have a non-empty reason,
  counterpart or explicit boundary, and no selected record remains deferred.
- No source Python, Make, YAML guard manifest, provider configuration, or object
  repository file was copied or modified.
- Tri-language comparison pages report the same counts and the same semantic
  boundary.

## Object/adopter inheritance

Every attached project inherits the shared Runtime, explicit `--repo` context,
repository-local Contract/evidence/knowledge, fail-closed lifecycle checks,
and human Outcome presentation. It does not inherit source stack commands,
Dependabot events, CODEOWNERS/SECURITY values, Python reports, or sample policy
decisions. Adopter/provider facts remain explicit external evidence.

## Verification

The machine inventory check and documentation/conformance gates are required
before Finish. This Work Item introduces no new Runtime code or governance
decision; any future portable extension requires a new bounded Contract.
