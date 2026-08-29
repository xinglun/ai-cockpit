---
author: AI Cockpit maintainers
title: WI-407 — Knowledge projection materialization boundary
description: Make derived Knowledge projections explicit, deterministic, and repository-local.
workItemId: WI-407-knowledge-materialization-boundary
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-407-knowledge-materialization-boundary
terminalArchive: .ai/work-items/archive/WI-407-knowledge-materialization-boundary.contract.json
terminalVerification: .ai/evidence/WI-407-knowledge-materialization-boundary.verification.json
---

# WI-407 — Knowledge projection materialization boundary

## Intent

Make the Knowledge directory, indexes, source digest, and refresh timing
explicit and verifiable without introducing a second governance authority.

## Scope

- Report the repository-local derived-write boundary from CLI and MCP Knowledge queries.
- Keep legacy and v2 projections deterministic, isolated, and rebuildable when stale or malformed.
- Preserve Contract, evidence, archive, and decision records as the authoritative source.
- Document the same boundary in English, Simplified Chinese, and Japanese.

## Evidence

- Archive Contract: `.ai/work-items/archive/WI-407-knowledge-materialization-boundary.contract.json`
- Verification: `.ai/evidence/WI-407-knowledge-materialization-boundary.verification.json`
- Pull request: [#372](https://github.com/xinglun/ai-cockpit/pull/372)

## Boundary

Knowledge is a derived repository-local projection. Explicit queries may
materialize or rebuild `.ai/knowledge/`, but never authorize a change or alter
governance authority. Lifecycle commands do not silently materialize it.
