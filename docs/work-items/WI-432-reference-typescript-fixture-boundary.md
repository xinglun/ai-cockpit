---
author: AI Cockpit maintainers
title: "WI-432 — TypeScript web fixture boundary"
workItemId: WI-432-reference-typescript-fixture-boundary
description: "Compare the pinned TypeScript web fixture one file at a time and record a Rust-native reference-only boundary without copying its Node toolchain."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-432-reference-typescript-fixture-boundary
terminalArchive: .ai/work-items/archive/WI-432-reference-typescript-fixture-boundary.contract.json
terminalVerification: .ai/evidence/WI-432-reference-typescript-fixture-boundary.verification.json
terminalFinalization: .ai/decisions/WI-432-reference-typescript-fixture-boundary.finalize.7ed22daac35a32d6f53289562f5fc955ba076854ff0483799f42c54a7a199eed.json
terminalDecision: .ai/decisions/WI-432-reference-typescript-fixture-boundary.close.json
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
---

# WI-432 — TypeScript web fixture boundary

## Intent and boundary

Read each of the eleven files under `examples/fixtures/typescript-web/` at
reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. They form an
executable TypeScript/npm sample in the reference repository. They are not Rust
Runtime code, Node or TypeScript toolchain support, portable governance policy,
or provider/enterprise evidence.

The target records each path as `reference-only` and explains its Rust-native
adopter boundary in [the adaptation guide](../reference/typescript-fixture-adaptation.md)
and [the file comparison ledger](../reference/reference-file-comparison.md).
No source fixture, npm dependency, installer, or Node lifecycle script is copied.

## Acceptance

- All eleven pinned paths are read and appear exactly once in the machine ledger.
- Every path has a non-empty reason and counterpart, is `reference-only`, and
  leaves no `deferred-next-batch` or `migrate-gap` record in this batch.
- English, Simplified Chinese, and Japanese adaptation, comparison, index, and
  parity routes agree on the source pin, file list, and non-copy boundary.
- Inventory and documentation gates pass without changing Runtime governance
  semantics, adopter toolchains, or global Agent/MCP configuration.

## Verification and non-claims

This is semantic/reference-boundary parity, not TypeScript toolchain support,
source-command compatibility, JSON-wire compatibility, or a second-stack
adopter acceptance. The machine ledger remains the source of per-file truth.

[简体中文](WI-432-reference-typescript-fixture-boundary.zh-CN.md) · [日本語](WI-432-reference-typescript-fixture-boundary.ja.md)
