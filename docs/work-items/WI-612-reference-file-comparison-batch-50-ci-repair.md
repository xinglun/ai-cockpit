---
title: "WI-612 — reference test parity batch 50"
description: "Compare the next twenty maintained reference test paths without copying source implementation or wire formats."
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
workItemId: WI-612-reference-file-comparison-batch-50-ci-repair
lastVerifiedBy: WI-612-reference-file-comparison-batch-50-ci-repair
terminalArchive: .ai/work-items/archive/WI-612-reference-file-comparison-batch-50-ci-repair.contract.json
terminalVerification: .ai/evidence/WI-612-reference-file-comparison-batch-50-ci-repair.verification.json
---

# WI-612 — reference test parity batch 50

[简体中文](WI-612-reference-file-comparison-batch-50-ci-repair.zh-CN.md) · [日本語](WI-612-reference-file-comparison-batch-50-ci-repair.ja.md)

## Intent and boundary

Re-read the next twenty maintained test paths from the pinned local reference
checkout, one file at a time. Carry portable governance responsibility to the
Rust Runtime, native tests, or documentation while keeping source/provider
fixtures and participant-study material bounded as `reference-only`.

This is semantic parity, not source command, Python module, or JSON-wire
compatibility. It does not modify the reference checkout, object repositories,
global Agent/MCP configuration, or immutable historical evidence.

## Bounded result

The complete path mapping is recorded in
`tests/conformance/reference_file_inventory.json` and the tri-language
comparison ledgers. Fifteen paths are `implemented-different-by-design`; five
are `reference-only`; no portable omission or `migrate-gap` was found. The
comparison metadata check also fails closed when its Runtime version lags the
single Cargo workspace version.

## Acceptance and verification

- Every selected path has one classification, counterpart set, and bounded reason.
- No source Python implementation or source wire format is copied.
- Inventory, metadata, tri-language ledgers, parity pages, and this record agree.
- `python3 tests/docs/reference_comparison_metadata_test.py`, documentation and
  parity checks, inventory check, and `cargo test --locked --workspace` pass.

The next comparison batch starts only after reviewed delivery, exact cleanup,
and a visible human Outcome. Attached object/adopter repositories inherit the
shared Runtime and repository-bound isolation; source Python/Make modules,
provider policy values, stack presets, and source wire formats do not cross
that boundary.
