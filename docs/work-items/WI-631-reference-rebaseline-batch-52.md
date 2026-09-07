---
author: AI Cockpit maintainers
title: WI-631 - Reference rebaseline batch 52
description: Re-read the next 60 source-changed reference paths without copying source implementation.
audience: [maintainer, reviewer, adopter]
workItemId: WI-631-reference-rebaseline-batch-52
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-631-reference-rebaseline-batch-52
terminalArchive: .ai/work-items/archive/WI-631-reference-rebaseline-batch-52.contract.json
terminalVerification: .ai/evidence/WI-631-reference-rebaseline-batch-52.verification.json
terminalFinalization: .ai/decisions/WI-631-reference-rebaseline-batch-52.finalize.json
terminalDecision: .ai/decisions/WI-631-reference-rebaseline-batch-52.close.json
---

# WI-631 - Reference rebaseline batch 52

This Work Item re-reads the next 60 non-history paths whose bytes changed at
the pinned local reference commit `a9224aed77b5c317b53c4551a9eec306d91ee330`.
Every path is classified in `tests/conformance/reference_file_inventory.json`;
the ledger is the machine-readable file-level record of its Rust counterparts,
classification, previous decision, and non-copy boundary.

## Result set

The explicit set is `WI631_REFERENCE_PATHS` in
`tests/conformance/reference_file_inventory.py` (60 paths). It contains 44
`implemented-different-by-design` paths and 16 `reference-only` response,
registry, or assessment records. No path is left `deferred-next-batch` or
`migrate-gap`. The 60 paths, in stable order, are:

```text
docs/reference/ai-cockpit-work-item-lifecycle.md
docs/reference/capability-truth-matrix.json
docs/reference/comprehension-validation-responses/peter_01.en.json
docs/reference/comprehension-validation-responses/peter_02.en.json
docs/reference/comprehension-validation-responses/tanaka_01.ja.json
docs/reference/comprehension-validation-responses/tanaka_02.ja.json
docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json
docs/reference/comprehension-validation-responses/xiaoli_02.zh-CN.json
docs/reference/comprehension-validation-results.json
docs/reference/comprehension-validation-results.md
docs/reference/content-bound-evidence-reuse.md
docs/reference/cross-wi-integration.md
docs/reference/deprecated-assets-registry.json
docs/reference/diff-bound-evidence-reuse.md
docs/reference/distribution.ja.md
docs/reference/distribution.md
docs/reference/documentation-context-registry.json
docs/reference/environment-bound-reuse.md
docs/reference/evidence-binding-foundation.md
docs/reference/governance-cost-metrics.md
docs/reference/governance-profiles.ja.md
docs/reference/governance-profiles.md
docs/reference/governance-profiles.zh-CN.md
docs/reference/implementation-knowledge.ja.md
docs/reference/implementation-knowledge.md
docs/reference/implementation-knowledge.zh-CN.md
docs/reference/japanese-capability-assessment.json
docs/reference/japanese-capability-assessment.md
docs/reference/performance-diagnosis.md
docs/reference/pre-release-documentation-alignment.json
docs/reference/pre-release-documentation-alignment.md
docs/reference/repository-workflow.ja.md
docs/reference/safe-parallel-verification.md
docs/reference/troubleshooting.md
docs/reference/verification-evidence-reuse-runtime.md
docs/reference/verification-evidence-reuse.md
docs/reference/work-item-lifecycle-closure.md
docs/reference/work-item-status-interface.md
docs/trust-layer.ja.md
docs/trust-layer.md
docs/trust-layer.zh-CN.md
docs/upgrade.ja.md
docs/upgrade.md
docs/upgrade.zh-CN.md
install.sh
next-release.json
pyproject.toml
release-state.json
release.json
requirements-dev.in
requirements-dev.lock
scripts/ai_adoption_reality_report.py
scripts/ai_archive_work_item.py
scripts/ai_check_backtrack.py
scripts/ai_check_knowledge_index.py
scripts/ai_check_pr.py
scripts/ai_check_reference_impact.py
scripts/ai_check_status_consistency.py
scripts/ai_check_summary.py
scripts/ai_check_task_outcome.py
```

`reference-only` means source study/provider material, not a missing Runtime
feature. The remaining paths are carried by existing typed Rust Runtime,
repository-native tests, CI/release boundaries, knowledge/evidence services,
or tri-language reader docs. Source Python, Make, provider decisions, and
source JSON wire formats are not copied. Attached object/adopter repositories
inherit shared Runtime semantics, explicit repository context, isolated
evidence/knowledge, fail-closed lifecycle, and visible human Outcome.

## Acceptance

Run the inventory check with the pinned source commit and historical target
anchor, documentation acceptance, parity status check, and the repository
quality gate. The next semantic batch begins only after this Work Item is
merged, closed, and post-close documentation promotion is current.

See also: [中文](WI-631-reference-rebaseline-batch-52.zh-CN.md) ·
[日本語](WI-631-reference-rebaseline-batch-52.ja.md).
