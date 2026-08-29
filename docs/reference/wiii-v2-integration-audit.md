---
author: AI Cockpit maintainers
title: Work Item Intelligence integration boundary
description: Auditable Rust-native projection for Work Item Intelligence without source wire compatibility claims.
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Work Item Intelligence integration boundary

[简体中文](wiii-v2-integration-audit.zh-CN.md) · [日本語](wiii-v2-integration-audit.ja.md)

The Rust Runtime exposes a request-scoped, read-only Work Item Intelligence
projection. It keeps schema versions explicit, reports source-bound
inconsistency instead of silently rebuilding it, and does not schedule Work
Items, call providers, or invent human approval.

## Current behavior

`status` and the intelligence commands read repository-local records and
evidence. A V2 projection is rebuilt only by an explicit command and is
validated against its source identity; a malformed or inconsistent record
remains unknown/inconsistent. Query, pagination, and cursor facts are bound to
the repository context supplied by `--repo`.

The projection is deliberately narrower than the reference Python CLI and is
not direct JSON/API compatibility. Source assessment scores, generated audit
bytes, and historical provider results remain reference-only. The same
installed Runtime can serve multiple repositories, but each repository keeps
separate Work Items, evidence, knowledge, and snapshots.

## Limits

This audit does not prove provider identity, distributed scheduling, network
isolation, human approval, or enterprise compliance. Those are separate
policy/provider boundaries and require their own evidence.
