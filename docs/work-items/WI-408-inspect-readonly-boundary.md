---
author: AI Cockpit maintainers
title: WI-408 — Read-only Work Item inspect boundary
description: Keep work-item inspect read-only while preserving explicit approach materialization.
workItemId: WI-408-inspect-readonly-boundary
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-408-inspect-readonly-boundary
terminalArchive: .ai/work-items/archive/WI-408-inspect-readonly-boundary.contract.json
terminalVerification: .ai/evidence/WI-408-inspect-readonly-boundary.verification.json
terminalFinalization: .ai/decisions/WI-408-inspect-readonly-boundary.finalize.json
terminalDecision: .ai/decisions/WI-408-inspect-readonly-boundary.close.json
---

# WI-408 — Read-only Work Item inspect boundary

## Intent

Make `work-item inspect` a truthful read-only projection. The command must
derive compatibility, implementation approach, and parallel-slot information
without silently materializing repository files. The explicit `work-item
approach` command remains the intentional write boundary.

## Scope

- Add a request-scoped, non-persisting implementation-approach path for inspect.
- Keep explicit `work-item approach` persistence and archive consumption unchanged.
- Add CLI and repository regressions proving repeated inspect calls preserve
  authoritative and derived bytes, including a freshly attached adopter.
- Document the boundary in English, Simplified Chinese, and Japanese and add a
  static CI guard against contradictory implementations or claims.

## Out of scope

Knowledge materialization, lifecycle state transitions, Agent provider/global
configuration, release/adopter harness implementation, and the explicit
`work-item approach` write semantics are unchanged.

## Acceptance

1. `work-item inspect --repo <path> --id <id>` returns its projection without
   creating or refreshing `.ai/work-items/active/<id>.approach.json`.
2. Explicit `work-item approach` still creates that repository-local artifact.
3. Repeated CLI and repository projections leave repository bytes unchanged.
4. Tri-language documentation and the static CI gate describe the same boundary.
5. A fresh attached adopter observes the same explicit `--repo` isolation.

## Evidence

Verification, repository-bound regression, and documentation-integrity evidence
are recorded by the Runtime lifecycle and linked after reviewed merge.
