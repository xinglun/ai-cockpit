---
workItemId: WI-395-runtime-performance-extreme
title: "Rust Runtime performance optimization"
author: AI Cockpit maintainers
description: "Measured Rust-native reduction of redundant snapshot and Work Item status work without weakening governance."
type: implementation
audience: [adopter, contributor, maintainer, reviewer]
authority: human-authorized
status: implemented
lastVerifiedBy: WI-395-runtime-performance-extreme
terminalArchive: .ai/work-items/archive/WI-395-runtime-performance-extreme.contract.json
terminalVerification: .ai/evidence/WI-395-runtime-performance-extreme.verification.json
terminalFinalization: .ai/decisions/WI-395-runtime-performance-extreme.finalize.json
terminalDecision: .ai/decisions/WI-395-runtime-performance-extreme.close.json
---

# WI-395 — Rust Runtime performance optimization

[简体中文](WI-395-runtime-performance-extreme.zh-CN.md) · [日本語](WI-395-runtime-performance-extreme.ja.md)

## Intent and installation boundary

Measure and reduce Rust Runtime cost for request-scoped status, observation,
and aggregate Work Item projections. The Runtime remains one externally
installed shared binary. Every adopter uses an explicit `--repo` and keeps an
independent `.ai/` state; this Work Item does not copy the reference installer,
SDK/toolchain setup, Make/Python runtime, or V1 wire behavior.

## Bounded optimization

- Reuse one identity-bound repository snapshot across aggregate Work Item
  status projections.
- Capture the source-tree digest during the existing Git index read and resolve
  remote default metadata with one bounded Git query.
- Avoid repeated recursive sorting during repository observation.
- Keep changes, unknown inputs, required checks, evidence binding, and
  fail-closed decisions unchanged.
- Measure before/after samples and report local cost facts rather than provider
  or enterprise assurance.

## Acceptance boundary

Performance targets are evaluated on the declared platform with captured
identity-bound evidence. A target miss remains a measured gap; it is never
“fixed” by skipping verification. Adopter acceptance repeats the same cold/warm
sequence with the installed or published Runtime and separate repository and
Runtime identities.
