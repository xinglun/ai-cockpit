---
author: AI Cockpit maintainers
title: Implementation knowledge
description: Deterministic, evidence-bound knowledge records for completed Work Items.
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - evidence_bound_knowledge
---

# Implementation knowledge

[English](implementation-knowledge.md) · [简体中文](implementation-knowledge.zh-CN.md) · [日本語](implementation-knowledge.ja.md)

Implementation knowledge is a derived projection of validated, archived Work
Items. It is not agent memory, a second fact source, or a design authority.
The Contract, verification evidence, archive, and final Outcome remain the
authoritative records.

## What an adopter can query

```text
ai-cockpit knowledge query --repo /path/to/repository \
  --topic <topic> --component <component> \
  --state verified --work-item-id <id>
```

The Runtime applies the supplied filters conjunctively and returns stable,
repository-bound records. `--v2` requests the richer `KnowledgeV2Record`
projection, including truth state, confidence, evidence references, unknowns,
and the snapshot digest. An explicit query may materialize or rebuild the
repository-local derived index under `.ai/knowledge/`; the response reports
`projection.materialization`, `projection.path`, and
`projection.writeBoundary=repository-local-derived`. This write never
authorizes a new change and never changes Contract, evidence, archive, or
decision authority.

Lifecycle commands do not silently materialize Knowledge. If the derived index
is missing, malformed, stale, or incomplete, the explicit query path rebuilds
and revalidates it from the archived source, or returns a visible
partial/unknown result. The source digest is a cache validator only; archived
records remain the source of truth.

## Explicit boundary with the reference source

The reference documentation also describes date, merged-commit,
`latestKnownRecord`, and explicit supersession filters. The current Rust
projection intentionally exposes only the repository-bound filters above;
those additional dimensions are not silently inferred and are not part of
this release's CLI/MCP contract. A future addition must have its own Contract,
schema, tests, and tri-language documentation.

Knowledge is not semantic search, vector retrieval, fuzzy recommendation,
RAG, or a promise that a new repository has the same implementation. Empty
results do not prove that a topic was never handled. Dates, supersession, and
benefit claims are shown only when explicitly recorded in evidence.

## Shared Runtime and adopter inheritance

The installed Runtime is shared, but every query carries an explicit `--repo`.
Indexes, records, evidence, and adapter state stay inside that repository's
`.ai/` context. An adopter inherits this read-only evidence boundary, not the
reference repository's generated records or source Python/Make commands.
