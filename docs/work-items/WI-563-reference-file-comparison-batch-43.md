---
author: AI Cockpit maintainers
title: "WI-563 — reference file comparison batch 43"
description: "Compare twenty maintained reference scripts and record bounded Rust semantic decisions."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-563-reference-file-comparison-batch-43
lastVerifiedBy: WI-563-reference-file-comparison-batch-43
terminalArchive: .ai/work-items/archive/WI-563-reference-file-comparison-batch-43.contract.json
terminalVerification: .ai/evidence/WI-563-reference-file-comparison-batch-43.verification.json
terminalFinalization: .ai/decisions/WI-563-reference-file-comparison-batch-43.finalize.json
terminalDecision: .ai/decisions/WI-563-reference-file-comparison-batch-43.close.json
---

[简体中文](WI-563-reference-file-comparison-batch-43.zh-CN.md) · [日本語](WI-563-reference-file-comparison-batch-43.ja.md)

# WI-563 — Reference file comparison batch 43

## Objective

Read the next twenty maintained files in the pinned local reference checkout
`fde3380f81fea5fd2e288f7a8849f737dc074060`, one by one, and record an explicit
Rust counterpart or a bounded source/provider-only decision. This is semantic
comparison, not source implementation or JSON-wire migration.

## Scope and boundary

The scoped paths are the wizard I/O/localization helpers, Work Item
intelligence and benchmark/status helpers, Bootstrap repository/wizard/write
boundaries, and the CI, documentation, governance, absurd-case, and release
checkers listed in the three comparison pages. The machine ledger,
tri-language comparison/parity pages, and this Work Item page are updated.

No Python, Shell, Make, source locale, provider credential, generated history,
or source JSON schema is copied. Runtime behavior, object repositories, and
global Agent/MCP configuration are out of scope. Any source-specific wizard,
Bandit/coverage floor, deprecated-asset registry, benchmark report, or provider
distribution behavior remains explicitly bounded rather than silently claimed
as a Rust capability.

## Comparison result

The twenty paths are classified as fourteen `implemented-different-by-design`,
five `reference-only`, and one `not-applicable`. The ledger, source pin,
counterpart lists, and English/Chinese/Japanese pages use the same path set.
No `migrate-gap` or portable implementation omission was found. Attached object
repositories inherit the shared Runtime, explicit repository binding,
isolated Contract/evidence/knowledge, trust/lifecycle gates, and visible human
Outcome; they do not inherit source Python modules, provider policy values, or
source wire formats.

## Verification

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
