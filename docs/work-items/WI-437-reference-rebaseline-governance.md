---
author: AI Cockpit maintainers
title: "WI-437 — local-reference governance rebaseline delta"
workItemId: WI-437-reference-rebaseline-governance
description: "Re-read seven governance files changed in the maintained local reference checkout."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-437-reference-rebaseline-governance
terminalArchive: .ai/work-items/archive/WI-437-reference-rebaseline-governance.contract.json
terminalVerification: .ai/evidence/WI-437-reference-rebaseline-governance.verification.json
terminalFinalization: .ai/decisions/WI-437-reference-rebaseline-governance.finalize.json
terminalDecision: .ai/decisions/WI-437-reference-rebaseline-governance.close.json
---

# WI-437 — local-reference governance rebaseline delta

This documentation and conformance Work Item re-reads seven files whose source
bytes changed after the previous reference ledger. The maintained semantic
reference is the local checkout at
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`; the public reference
repository is not accessed. The task records semantic parity decisions only and
does not copy Python, Make, YAML, or source JSON artifacts into the Rust project.

[简体中文](WI-437-reference-rebaseline-governance.zh-CN.md) · [日本語](WI-437-reference-rebaseline-governance.ja.md)

## Scope

- Re-read `.ai/cockpit/README.md`, `.ai/cockpit/README.ja.md`,
  `.ai/cockpit/adoption.ja.md`, `.ai/guards/changed_critical_coverage_policy.json`,
  `.ai/guards/coverage_policy.yaml`, `.ai/quality/governance-routing.yaml`, and
  `.ai/schemas/task_outcome.schema.json` at the pinned local source commit.
- Record an explicit Rust-native counterpart or non-portability reason for each.
- Update the machine inventory, tri-language comparison/parity documentation, and
  regression assertions without changing Runtime behavior.

## File-level decision

All seven files are `implemented-different-by-design`. The source changes are
reference-side cleanup of Python/Make surfaces: removing an obsolete
`REPORT_LANGUAGE` argument, deleting Python-only coverage associations,
separating route selection from duplicated gate metadata, and simplifying the
Python Task Outcome schema. Rust keeps its own typed OutcomeV2/humanHandoff and
dynamic gate boundaries; source wire shapes are not compatibility requirements.

## Verification

The local source policy, inventory regression, documentation acceptance, parity
status, governance integrity gate, and Runtime verification must pass. The
inventory retains `previousBatch`, `previousClassification`, and
`sourceChangedSincePrevious` provenance while the seven current records leave
`deferred-next-batch`.
