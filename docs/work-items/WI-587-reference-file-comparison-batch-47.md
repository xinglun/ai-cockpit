---
author: AI Cockpit maintainers
title: "WI-587 — reference test and fixture comparison batch 47"
description: "Compare the next twenty maintained reference test and fixture paths without copying source implementation or wire data."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-587-reference-file-comparison-batch-47
lastVerifiedBy: WI-587-reference-file-comparison-batch-47
terminalArchive: .ai/work-items/archive/WI-587-reference-file-comparison-batch-47.contract.json
terminalVerification: .ai/evidence/WI-587-reference-file-comparison-batch-47.verification.json
terminalFinalization: .ai/decisions/WI-587-reference-file-comparison-batch-47.finalize.77d894a5f84cb71e7a2270802132f00a78b05cc2a8aa2b6e51131daeec4a3782.json
terminalDecision: .ai/decisions/WI-587-reference-file-comparison-batch-47.close.json
---

[简体中文](WI-587-reference-file-comparison-batch-47.zh-CN.md) · [日本語](WI-587-reference-file-comparison-batch-47.ja.md)

# WI-587 — Reference test and fixture comparison batch 47

## Objective

Read the next twenty maintained paths in the pinned local reference checkout
`fde3380f81fea5fd2e288f7a8849f737dc074060`, one by one, and record an
evidence-backed semantic decision. This is a comparison of responsibilities,
not a source implementation, Python module, or JSON-wire migration.

## Compared paths and decisions

Twelve paths are `implemented-different-by-design` because the portable
responsibility is already provided by typed Rust Runtime services, native
tests, or immutable release/adopter harnesses:

- `tests/repository_fixture.py`
- `tests/test_absurd_capability_truth.py`
- `tests/test_adoption_e2e.py`
- `tests/test_adoption_evidence.py`
- `tests/test_adoption_ready.py`
- `tests/test_ai_archive_work_item.py`
- `tests/test_ai_check_serial_order.py`
- `tests/test_ai_check_summary.py`
- `tests/test_ai_check_work_item.py`
- `tests/test_ai_external_handoff.py`
- `tests/test_ai_onboard.py`
- `tests/test_ai_post_archive_recovery.py`

The ledger classifies the eight source-owned inputs as `reference-only`:

- `tests/conftest.py`
- `tests/fixtures/japanese-capability-corpus.json`
- `tests/fixtures/wizard/android.json`
- `tests/fixtures/wizard/ios.json`
- `tests/fixtures/wizard/monorepo.json`
- `tests/snapshots/wizard/kotlin.json`
- `tests/snapshots/wizard/mixed.json`
- `tests/snapshots/wizard/swift.json`

The complete path-by-path mapping, Rust counterparts, and non-overclaiming
reasons are in [`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json)
and in the tri-language [reference file comparison](../reference/reference-file-comparison.md).

## Boundaries and adopter inheritance

The Rust target preserves the portable semantics: explicit repository
contexts, adversarial fail-closed behavior, immutable release/adopter
acceptance, archive/recovery integrity, serial lifecycle gates, typed Summary
and Contract validation, external handoff identity, and explicit onboarding.
Attached object/adopter repositories inherit these shared Runtime boundaries
and repository isolation. They do not inherit source pytest fixtures, sample
participant/capability corpora, stack toolchain presets, Python evaluators,
source installer workflows, or source JSON wire shapes.

No `migrate-gap` was found in this batch. The source-only fixtures remain
available as conformance reference material and are not silently promoted to
Runtime capability claims.

## Verification

- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `cargo test --locked --workspace --all-targets --all-features -- --test-threads=1`
- `git diff --check`
