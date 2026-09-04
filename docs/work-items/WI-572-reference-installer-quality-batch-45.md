---
author: AI Cockpit maintainers
title: "WI-572 — installer and quality reference comparison batch 45"
description: "Compare twenty maintained reference paths and record bounded Rust semantic decisions."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-572-reference-installer-quality-batch-45
lastVerifiedBy: WI-572-reference-installer-quality-batch-45
terminalArchive: .ai/work-items/archive/WI-572-reference-installer-quality-batch-45.contract.json
terminalVerification: .ai/evidence/WI-572-reference-installer-quality-batch-45.verification.json
terminalFinalization: .ai/decisions/WI-572-reference-installer-quality-batch-45.finalize.ce778cafe4377bd38aad5238a5fd182cee9611e7017c91e83f40f0a1116cda6f.json
terminalDecision: .ai/decisions/WI-572-reference-installer-quality-batch-45.close.json
---

[简体中文](WI-572-reference-installer-quality-batch-45.zh-CN.md) · [日本語](WI-572-reference-installer-quality-batch-45.ja.md)

# WI-572 — Installer and quality reference comparison batch 45

## Objective

Read the next twenty maintained files in the pinned local reference checkout
`fde3380f81fea5fd2e288f7a8849f737dc074060`, one by one. Record either the
Rust counterpart or a bounded source/provider-only decision. This is semantic
comparison, not source implementation or JSON-wire migration.

## Compared paths and decisions

The complete path-by-path ledger is maintained in
`tests/conformance/reference_file_inventory.json` and the tri-language
comparison pages. Nineteen paths are `implemented-different-by-design`:

- `scripts/installer/{git_state,inspection,legacy,ownership,planning,presentation,rollback,transaction,upgrade}.py`
- `scripts/quality_{measurements,session_lock,test_manifest}.py`
- `scripts/release_archive.py`
- `scripts/run_quality_{gate,session}.py`
- `scripts/summarize_quality_gates.py`
- `scripts/sync_published_release_projection.py`
- `scripts/unsupported_claim_gate.py`
- `scripts/verify_quick_install_release.py`

`scripts/real_adopter_reference_validation.py` is `reference-only`: its
seven-project matrix is specific to the reference template and is not a
portable Rust Runtime contract.

## Boundaries and adopter inheritance

The shared Rust Runtime, explicit `--repo` context, typed Agent/release/
verification services, dynamic quality route, isolated Contract/evidence/
knowledge, and human Outcome handoff are the target capabilities. Source
Python modules, Make/provider orchestration, source wire formats, and
template-specific stack matrices are not copied. Attached object/adopter
repositories inherit the same Runtime capabilities and boundaries, not the
source implementation.

The batch also hardens lifecycle recovery: when a human-authorized Contract
amendment invalidates a predecessor verification receipt, preflight classifies
that receipt as stale (not tampered) so a fresh verification can replace it;
malformed or foreign evidence remains contradictory and fail-closed. A fresh
replacement verification consumes only the active retry marker projection;
the append-only recovery receipt remains historical evidence.

## Verification

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report /tmp/ai-cockpit-governance-integrity.json`
- `git diff --check`
