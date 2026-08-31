---
author: AI Cockpit maintainers
title: "WI-441 — local-reference entrypoint and Agent parity"
workItemId: WI-441-reference-entrypoint-parity
description: "Re-read the local reference's Agent rules, reader routes, capability boundary, and Task Outcome entrypoint."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-441-reference-entrypoint-parity
terminalArchive: .ai/work-items/archive/WI-441-reference-entrypoint-parity.contract.json
terminalVerification: .ai/evidence/WI-441-reference-entrypoint-parity.verification.json
terminalFinalization: .ai/decisions/WI-441-reference-entrypoint-parity.finalize.json
terminalDecision: .ai/decisions/WI-441-reference-entrypoint-parity.close.json
---

# WI-441 — local-reference entrypoint and Agent parity

This Work Item re-reads nine files in the maintainer-owned local reference
checkout at `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`, pinned
to commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The public reference is
not accessed. The result is a file-level semantic decision for the Rust target;
it is not a request to copy source Agent files, Python/Make commands, or source
JSON wire formats.

[简体中文](WI-441-reference-entrypoint-parity.zh-CN.md) · [日本語](WI-441-reference-entrypoint-parity.ja.md)

## Scope

The following paths were read individually and registered in the generated
inventory:

| Local reference path | Classification | Rust counterpart or boundary |
| --- | --- | --- |
| `AGENTS.md` | `implemented-different-by-design` | `AGENTS.md`, `.ai/README.md`, `docs/reference/agent-workflow.md`, and typed lifecycle/adapter services preserve Contract-first work, latest-base discovery, human pause, closure, and cleanup. Source `make ai-*` commands are not target commands. |
| `GEMINI.md` | `implemented-different-by-design` | `.ai/README.md`, `crates/cockpit-agent`, its install tests, and the explicit Gemini adapter route provide the provider-facing guidance without committing a provider-specific file or changing global configuration. |
| `docs/README.md` | `implemented-different-by-design` | The target's current/getting-started/operations/reference map preserves the reader-first and goal-first route with Rust-specific boundaries. |
| `docs/README.zh-CN.md` | `implemented-different-by-design` | The Chinese reader route and cross-language links preserve the same intent and add explicit Runtime/adopter boundaries. |
| `docs/README.ja.md` | `implemented-different-by-design` | The Japanese reader route and cross-language links preserve the same intent and add explicit Runtime/adopter boundaries. |
| `docs/capabilities.md` | `implemented-different-by-design` | The target capability page keeps the Repository Governance Layer and external non-claims, then documents Rust-native CLI, MCP, scaffold, profile, knowledge, Outcome, and isolation paths. |
| `docs/capabilities.zh-CN.md` | `implemented-different-by-design` | The Chinese capability route preserves the source boundary and explains the repository-local Runtime/adopter inheritance without copying source statuses. |
| `docs/capabilities.ja.md` | `implemented-different-by-design` | The Japanese capability route preserves the source boundary and explains the repository-local Runtime/adopter inheritance without copying source statuses. |
| `docs/features/task-outcome-report.md` | `implemented-different-by-design` | `OutcomeV2`, the CLI/MCP human handoff, and immutable evidence preserve the report/status/PR separation; source report prose and Make commands are not wire requirements. |

## Boundary decision

The current reference is itself a specification corpus and is now private and
local-only. A source path being absent from this Rust repository is not an
omission when its portable responsibility is provided by the shared Runtime,
the repository-local `.ai/` route, or an explicit provider adapter. Conversely,
source-specific commands and generated records remain reference-only. The same
boundary is inherited by adopter repositories: one installed Runtime, an
explicit repository context, repository-local evidence, and provider-owned
Agent configuration only when an owner explicitly installs an adapter.

## Verification boundary

The inventory keeps `sourceChangedSincePrevious`, `previousBatch`, and
`previousClassification` for audit history while resolving these nine current
records. The local source policy requires a clean checkout at the pinned commit
and `network_access = false`; hosted CI consumes only the committed offline
corpus. Documentation, parity, governance-integrity, status-consistency, and
locked workspace tests are the declared verification evidence.
