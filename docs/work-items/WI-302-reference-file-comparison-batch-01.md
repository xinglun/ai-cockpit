---
author: AI Cockpit maintainers
title: "WI-302 — first deferred reference-file comparison batch"
workItemId: WI-302-reference-file-comparison-batch-01
description: "Compare the first ten deferred reference-source files with the Rust target and record bounded semantic conclusions."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-302-reference-file-comparison-batch-01
terminalArchive: .ai/work-items/archive/WI-302-reference-file-comparison-batch-01.contract.json
terminalVerification: .ai/evidence/WI-302-reference-file-comparison-batch-01.verification.json
terminalFinalization: .ai/decisions/WI-302-reference-file-comparison-batch-01.finalize.json
terminalDecision: .ai/decisions/WI-302-reference-file-comparison-batch-01.close.json
authority: canonical
---

# WI-302 — first deferred reference-file comparison batch

## Intent

Compare the first ten deferred reference-source records in lexical order against
the pinned source commit `e5acb677`, preserving the boundary between portable
governance semantics and source-language or provider-specific implementation.

## Scope and result

The batch covers `.ai/cockpit/bandit_low_risk_baseline.json`, `.gitattributes`,
the three selected GitHub metadata/workflow files, `.gitignore`, `LICENSE`, and
`Makefile`. The inventory records each source responsibility, Rust counterpart
or absence, classification, and reason. Compatibility and smoke workflow
matrices remain explicitly deferred because they require a separate multi-stack
and second-adopter comparison.

The synchronized ledger and tri-language report are:

- `tests/conformance/reference_file_inventory.json`
- `docs/reference/reference-file-comparison.md`
- `docs/reference/reference-file-comparison.zh-CN.md`
- `docs/reference/reference-file-comparison.ja.md`

## Evidence boundary

The target baseline is `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`; verification
was executed by installed Runtime `0.2.33` with binary digest
`sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`.
The archive and verification records are authoritative for lifecycle state;
this document is a readable projection and does not introduce source-language
runtime or provider ownership policy.
