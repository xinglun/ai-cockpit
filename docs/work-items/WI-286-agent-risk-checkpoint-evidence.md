---
author: AI Cockpit maintainers
title: "WI-286 — Rust Agent Risk and checkpoint evidence boundary"
workItemId: WI-286-agent-risk-checkpoint-evidence
description: "Migrate the reference Agent Risk and checkpoint controls into one typed, request-scoped Rust lifecycle boundary."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-286-agent-risk-checkpoint-evidence
authority: canonical
---

# WI-286 — Rust Agent Risk and checkpoint evidence boundary

This bounded parity batch brings the reference source's Agent Risk and
checkpoint semantics into the Rust Runtime without copying Python scripts,
Make targets, or provider-global configuration. It covers typed strict
`checkpointPolicy`/`checkpointEvidence`, intent/scenario route enforcement,
required verification declarations, legal unknown paths, and append-only
Contract-amendment revalidation.

`before_edit` remains immutable. A post-verification Contract amendment must
record its previous/current hashes, reason, and invalidated checks; resume
history makes older checkpoint evidence stale. Fresh preflight and verification
are required before a terminal transition. `light`, `standard`, `strict`, and
`release` are verification-strength profiles only; they do not imply Evidence
Assurance.

The Runtime keeps Contract acceptance text in its source language. The human
Outcome localizes fixed presentation labels but does not translate governance
facts. CI integration, planner/performance work, release harness changes, and
large-scale module decomposition remain separate bounded batches.

## Reference correspondence

| Reference responsibility | Rust boundary |
| --- | --- |
| `ai_check_agent_risk.py` | `validate_agent_risk_controls` and lifecycle gate reuse |
| `ai_checkpoint.py` | typed `CheckpointPolicy`, `CheckpointEvidence`, and `revalidate_contract_amendment` |
| intent/scenario route binding | `resolve_verification_route` before command execution |
| static Agent-rule parity | Rust `agent_rule_parity` regression test |

## Acceptance boundary

- malformed, unknown-field, duplicate, foreign, stale, contradictory, and
  symlinked checkpoint inputs fail closed;
- missing or failed required verification gates cannot reach finish/archive;
- amendment and resume history cannot reuse stale evidence;
- repository context remains explicit and isolated for adopter repositories;
- English, Simplified Chinese, and Japanese documentation state the semantic
  (not wire-byte) parity boundary.
