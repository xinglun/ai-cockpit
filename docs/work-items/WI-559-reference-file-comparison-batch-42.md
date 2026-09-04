---
author: AI Cockpit maintainers
title: "WI-559 — reference file comparison batch 42"
description: "Compare twenty deferred reference scripts and record their Rust-native semantic boundaries."
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-559-reference-file-comparison-batch-42
lastVerifiedBy: WI-559-reference-file-comparison-batch-42
---

[简体中文](WI-559-reference-file-comparison-batch-42.zh-CN.md) · [日本語](WI-559-reference-file-comparison-batch-42.ja.md)

# WI-559 — Reference file comparison batch 42

## Goal

Compare the next twenty maintained scripts in the pinned local reference
checkout with the Rust Runtime, one file at a time, and record an explicit
semantic counterpart or bounded reference-only decision.

## Scope and boundary

The machine ledger and tri-language comparison/parity pages are updated for:
`ai_onboard`, `ai_prepare_hosted_verification`, `ai_project_doctor`,
`ai_projection_lease`, `ai_provider_merge_state_recovery`,
`ai_quality_architecture`, `ai_resume_work_item`, `ai_start`,
`ai_start_receipt`, `ai_task_event_log`, `ai_terminology`, `ai_trust_guards`,
`ai_trust_schema`, `ai_uninstall_facts`, `ai_uninstall_proposal`,
`ai_unknown_confirmation`, `ai_validate_java_runtime`,
`ai_verification_context`, `ai_verification_policy`, and `ai_verify`.

No Python or shell implementation is copied. Hosted-snapshot preparation,
Python AST architecture auditing, and Java runtime selection remain
source/provider or adopter-specific. No Runtime behavior, object repository,
or global Agent/MCP configuration is changed.

## Result

Seventeen paths are `implemented-different-by-design`; three are
`reference-only`. The ledger, source pin, Rust counterpart list, and all three
language pages agree. Attached object repositories inherit the shared Runtime,
explicit repository binding, isolated Contract/evidence/knowledge, trust and
lifecycle gates, and human Outcome handoff.

## Verification

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
