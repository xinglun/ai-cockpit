---
author: AI Cockpit maintainers
title: "WI-322 — lifecycle entry safety"
workItemId: WI-322-lifecycle-entry-safety
description: "Fail closed before a new Work Item when repository closure or pre-start base conditions are unresolved."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-322-lifecycle-entry-safety
terminalArchive: .ai/work-items/archive/WI-322-lifecycle-entry-safety.contract.json
terminalVerification: .ai/evidence/WI-322-lifecycle-entry-safety.verification.json
terminalFinalization: .ai/decisions/WI-322-lifecycle-entry-safety.finalize.json
terminalDecision: .ai/decisions/WI-322-lifecycle-entry-safety.close.json
---

# WI-322 — lifecycle entry safety

## Intent and boundary

Prevent a new governed Work Item from starting when the repository still has
an archived Work Item without a valid close decision, pre-start non-governance
changes, a detached branch, or a known branch/base mismatch. Repository
metadata that cannot be determined remains `unknown`; it is never presented as
green readiness.

The checks are repository-scoped and do not create a process-global current
project. Explicit recovery continuations retain their existing recovery path.

## Scope and acceptance

- `work-item new` and `start` fail closed on unresolved archived closure and
  preserve immutable archive bytes.
- `status` exposes deterministic `readiness`/`readyOnBase` facts and blockers.
- Pre-start user changes are rejected while Runtime-owned `.ai` writes remain
  allowed.
- A discoverable remote default ref is checked without network access; missing
  metadata yields `unknown` readiness.
- Two repository contexts remain isolated, with tri-language command and agent
  workflow documentation.

## Verification

The locked workspace tests, lifecycle-entry regressions, documentation gates,
and hosted CI verify the behavior. All repository-bound Runtime commands use
an explicit `--repo` path.

[简体中文](WI-322-lifecycle-entry-safety.zh-CN.md) ·
[日本語](WI-322-lifecycle-entry-safety.ja.md)
