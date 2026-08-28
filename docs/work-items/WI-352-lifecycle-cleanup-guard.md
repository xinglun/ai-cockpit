---
author: AI Cockpit maintainers
title: "WI-352 — Lifecycle cleanup guard"
workItemId: WI-352-lifecycle-cleanup-guard
description: "Make incomplete lifecycle cleanup visible and fail closed for repositories and release-adopter runs."
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-352-lifecycle-cleanup-guard
terminalArchive: .ai/work-items/archive/WI-352-lifecycle-cleanup-guard.contract.json
terminalVerification: .ai/evidence/WI-352-lifecycle-cleanup-guard.verification.json
terminalFinalization: .ai/decisions/WI-352-lifecycle-cleanup-guard.finalize.json
terminalDecision: .ai/decisions/WI-352-lifecycle-cleanup-guard.close.json
capabilityClaims: [lifecycle_governance, cleanup_handoff]
---

# WI-352 — Lifecycle cleanup guard

[简体中文](WI-352-lifecycle-cleanup-guard.zh-CN.md) · [日本語](WI-352-lifecycle-cleanup-guard.ja.md)

## Intent and boundary

Make an archived Work Item with missing or invalid close evidence visibly
non-terminal. The Runtime must expose the exact cleanup/finalization/close
next actions in status and human Outcome, while preserving repository-local
state and the shared Runtime boundary. Release-adopter harnesses must remove
their isolated run roots on both success and failure after writing their
receipts; cleanup must never rewrite acceptance truth.

## Verification

- Archived-but-unclosed state is blocking, yellow, and actionable; it cannot
  be reported as green or permit the next Work Item.
- Valid finalization and close remain bound to the reviewed PR, branch,
  worktree, repository, and Runtime identity.
- Harness and wrapper cleanup is tested for success and failure paths, with
  HOME/XDG_CONFIG_HOME forbidden-write roots kept isolated from runtime-write
  TMPDIR/CARGO_HOME roots.
- English, Simplified Chinese, and Japanese documentation carry the same
  boundary and preserve the immutable archive/evidence record.

## Delivery status

The implementation and verification evidence are archived. The reviewed PR
still requires provider finalization, exact resource cleanup, and a structured
close decision before this Work Item becomes terminal.
