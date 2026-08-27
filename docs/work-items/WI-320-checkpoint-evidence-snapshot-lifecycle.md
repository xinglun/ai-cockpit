---
author: AI Cockpit maintainers
title: "WI-320 — checkpoint evidence snapshot lifecycle"
workItemId: WI-320-checkpoint-evidence-snapshot-lifecycle
description: "Allow historical edit checkpoints while keeping terminal checkpoint evidence current and bound."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-320-checkpoint-evidence-snapshot-lifecycle
terminalArchive: .ai/work-items/archive/WI-320-checkpoint-evidence-snapshot-lifecycle.contract.json
terminalVerification: .ai/evidence/WI-320-checkpoint-evidence-snapshot-lifecycle.verification.json
terminalFinalization: .ai/decisions/WI-320-checkpoint-evidence-snapshot-lifecycle.finalize.json
terminalDecision: .ai/decisions/WI-320-checkpoint-evidence-snapshot-lifecycle.close.json
---

# WI-320 — checkpoint evidence snapshot lifecycle

## Intent and boundary

`before_edit` is an authorization boundary recorded before implementation. A
later edit and fresh preflight necessarily produce a newer repository snapshot;
that history must remain valid without weakening the `before_finish` boundary.
The terminal checkpoint must still bind the current Contract, repository, and
snapshot, and declared verification checks must correspond to real results.

## Scope and acceptance

- Historical `before_edit` and amendment records are accepted when their
  identity, shape, stage, and amendment chain are valid; they are not silently
  treated as current terminal evidence.
- `before_finish` remains current-snapshot bound and fails closed for stale,
  foreign, malformed, duplicate, or symlink-backed evidence.
- Required checkpoint checks are derived deterministically and cannot introduce
  a phantom verification name.
- Existing amendment, resume, lifecycle, and repository-isolation regressions
  remain green.
- English, Simplified Chinese, and Japanese documentation state this temporal
  evidence boundary and link the final Runtime receipts.

## Verification

Run the focused checkpoint and lifecycle tests, the locked workspace tests,
documentation/parity gates, and the hosted checks for the reviewed branch.
Every repository-bound Runtime command uses the explicit repository path.

## Out of scope

Planner and parallel execution, performance, CI/release/adopter harnesses,
global Agent or MCP configuration, and the later architecture split of the
large repository module are outside this bounded correction.

[简体中文](WI-320-checkpoint-evidence-snapshot-lifecycle.zh-CN.md) ·
[日本語](WI-320-checkpoint-evidence-snapshot-lifecycle.ja.md)
