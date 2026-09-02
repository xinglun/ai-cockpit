---
author: AI Cockpit maintainers
title: "WI-494 — capability, comprehension, and deprecated-assets reference rebaseline"
description: "Re-read seven changed local reference records and preserve explicit Rust-native boundaries."
audience:
  - maintainer
  - reviewer
workItemId: WI-494-reference-file-comparison-batch-27
status: implemented
authority: canonical
lastVerifiedBy: WI-494-reference-file-comparison-batch-27
terminalArchive: .ai/work-items/archive/WI-494-reference-file-comparison-batch-27.contract.json
terminalVerification: .ai/evidence/WI-494-reference-file-comparison-batch-27.verification.json
terminalFinalization: .ai/decisions/WI-494-reference-file-comparison-batch-27.finalize.json
terminalDecision: .ai/decisions/WI-494-reference-file-comparison-batch-27.close.json
---

# WI-494 — capability, comprehension, and deprecated-assets reference rebaseline

## Goal

Re-read the seven local reference paths whose bytes changed after their prior
`reference-only` decisions. Record a bounded, evidence-backed decision for each
path without copying source study data, Python/Make implementation, or source
cleanup tooling into the Rust repository.

## Scope and boundary

The seven source paths are:

- `docs/reference/capability-truth-matrix.json`
- `docs/reference/comprehension-validation-responses/peter_01.en.json`
- `docs/reference/comprehension-validation-responses/tanaka_01.ja.json`
- `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json`
- `docs/reference/comprehension-validation-results.json`
- `docs/reference/comprehension-validation-results.md`
- `docs/reference/deprecated-assets-registry.json`

All seven remain `reference-only`. The capability matrix is a source-owned
claim/freshness projection; the participant responses and comprehension report
are revision-bound study evidence; and the deprecated-assets registry is a
source-specific cleanup aid. Rust preserves the applicable boundaries through
typed request-scoped capability views, reader-facing Outcome documentation,
immutable Work Item history, and reviewed resource finalization. None of the
source bytes becomes Runtime authority or adopter evidence.

The inventory application and regression test retain each path's prior
classification and `sourceChangedSincePrevious` provenance. The tri-language
comparison and parity routes document the same no-copy decision.

## Acceptance

- The seven paths are re-read one by one and recorded in the inventory as
  `reference-only`, with non-empty Rust counterparts and reasons.
- No participant, comprehension, source capability-claim, or source cleanup
  registry bytes are copied into Runtime or adopter state.
- Inventory validation, the conformance regression, documentation acceptance,
  parity status checks, and the repository's declared Runtime verification pass.
- The branch is delivered through the reviewed PR lifecycle and exact cleanup;
  no global Agent/MCP configuration or object/adopter repository is changed.

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

The reference checkout is local and pinned by
`tests/conformance/reference-source.lock`; source implementation and JSON-wire
compatibility are not asserted.
