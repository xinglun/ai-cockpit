---
author: AI Cockpit maintainers
title: "WI-557 — reference script comparison batch 41"
description: "Compare thirteen deferred reference scripts and record their Rust-native semantic boundaries."
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-557-reference-file-comparison-batch-41
lastVerifiedBy: WI-557-reference-file-comparison-batch-41
---

[简体中文](WI-557-reference-file-comparison-batch-41.zh-CN.md) · [日本語](WI-557-reference-file-comparison-batch-41.ja.md)

# WI-557 — reference script comparison batch 41

## Objective

Compare the thirteen named scripts in the pinned local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` one by one. Record the portable
responsibility, the Rust-native counterpart, and any deliberate boundary in the
append-only inventory. This is semantic conformance work, not source-code or
wire-format copying.

## Scope

`scripts/ai_issue_log.py`, `scripts/ai_linked_worktree_recovery.py`,
`scripts/ai_ownership.py`, `scripts/ai_performance_budget.py`,
`scripts/ai_project_profile.py`, `scripts/ai_purge.py`,
`scripts/ai_readiness_policy.py`, `scripts/ai_recovery_usability.py`,
`scripts/ai_review_readiness_policy.py`, `scripts/ai_risk_policy.py`,
`scripts/ai_rollback.py`, `scripts/ai_safety_gate.py`, and
`scripts/ai_schema_migration.py`, plus the target ledger, conformance checks,
and the three-language comparison/parity pages listed by the Contract.

## Boundary

The Python modules, their source tests, source registries, and source JSON wire
formats remain reference material. The shared Rust Runtime, repository-local
Protocol, external provider boundary, and object repositories are not changed
by this documentation/ledger slice. The source recovery-usability scenario
registry remains explicitly reference-only because the target has no equivalent
generic fixed scenario registry.

## Acceptance

- Every named source path has exactly one explicit `WI-557` inventory record.
- Every record has a non-empty classification, Rust counterpart set, and
  evidence-backed reason; no path remains deferred or a migrate gap.
- The ledger regression and tri-language comparison/parity pages agree on the
  exact thirteen paths and pinned source commit.
- No reference implementation bytes, object-repository files, or global Agent
  configuration are added or modified.
- Reference inventory, documentation, governance-integrity, and diff checks
  pass.

## Verification

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
