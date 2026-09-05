---
title: "WI-598 — reference test parity batch 48"
description: "Compare the next twenty maintained reference test paths without copying source implementation or wire formats."
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
workItemId: WI-598-reference-test-parity-batch-48
lastVerifiedBy: WI-598-reference-test-parity-batch-48
---

# WI-598 — reference test parity batch 48

[简体中文](WI-598-reference-test-parity-batch-48.zh-CN.md) · [日本語](WI-598-reference-test-parity-batch-48.ja.md)

## Intent and boundary

Compare the next twenty maintained files from the pinned local reference
checkout, one file at a time. Carry forward portable governance semantics to
the Rust Runtime or repository-native gates, while retaining stack-specific
toolchain and source-harness material as `reference-only`.

This is semantic parity, not source command, Python module, or JSON-wire
compatibility. It does not modify object repositories, global Agent/MCP
configuration, or immutable historical evidence.

## Bounded result

The twenty paths are recorded in
`tests/conformance/reference_file_inventory.json` under
`WI-598-reference-test-parity-batch-48`:

- 18 are `implemented-different-by-design`, backed by existing typed Git,
  repository, profile, evidence, CI, and release boundaries.
- 2 are `reference-only`: Java runtime selection and Bandit baseline data are
  provider/toolchain-specific and are not Runtime requirements.

No `migrate-gap` was found. The tri-language ledger and metadata sidecar are
updated together; the ledger remains append-only and source history is not
rewritten.

## Acceptance and verification

- Every path has exactly one classification, counterpart set, and bounded
  reason.
- Any confirmed portable omission is fixed within this Work Item rather than
  silently deferred.
- Inventory, regression scripts, metadata, tri-language comparison/parity
  pages, and this record agree.
- Conformance, documentation, governance-integrity, and locked workspace
  checks pass before finish.

The release following this batch must use immutable public artifacts and the
published adopter/N-1 acceptance harness. The next comparison batch starts
only after the reviewed release, exact cleanup, and visible human Outcome.
