---
author: AI Cockpit maintainers
title: Schemas and record authority
description: The Rust-native record map and validation boundary for AI Cockpit.
audience:
  - adopter
  - contributor
  - maintainer
  - auditor
status: current
authority: canonical
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - typed_record_schemas
---

# Schemas and record authority

[English](schemas.md) · [简体中文](schemas.zh-CN.md) · [日本語](schemas.ja.md)

The executable Rust Protocol and repository validators decide whether a record
is valid. Documentation and examples explain the boundary; they do not grant
authority. Every repository-bound record carries the repository identity and
the relevant Work Item or snapshot binding where applicable.

| Record or surface | Rust-native authority | Boundary |
| --- | --- | --- |
| Work Item Contract | `cockpit-protocol` typed Contract plus repository validation | Human intent, scope, authority, acceptance, and verification declarations are not inferred. |
| Change Summary | Runtime-generated Summary under `.ai/work-items/` | Changed paths, checkpoint, preflight, acceptance evidence, and cost facts are derived or bound; the Summary cannot authorize a change. |
| Project Profile | `.ai/project.json` and profile policy | Detection facts and human confirmation are separate; a candidate proposal never changes the baseline. |
| Repository Protocol | `.ai/cockpit.toml`, `project.json`, and attached identity | The Runtime has no persistent current repository or global Work Item. |
| Verification Evidence | `.ai/evidence/<work-item>.verification.json` | Schema, Work Item, repository, snapshot, runtime, receipt, and `passed` fields are validated; file existence alone is not evidence. |
| Checkpoint Evidence | typed `checkpointEvidence` in the Summary | Stage, order, hashes, counts, amendment, and resume freshness are fail-closed. |
| Delegated Evidence | `evidence import` metadata plus exact raw-byte digest | Provider/enterprise assurance remains external; imported bytes are displayed and bound, not invented. |
| Archive and decision | archive manifest, finalization receipt, close decision | These are immutable history and human-decision boundaries, not editable status caches. |
| Outcome and status | Runtime projections (`work-item outcome`, `status`) | Derived views cannot authorize merge, release, or approval. |
| Audit export | `audit export` event bundle | External SIEM/WORM/retention systems own long-term immutability. |

## Strictness and compatibility

Current V2 records reject malformed required fields, unsafe paths, duplicate
identities, unknown nested fields where the typed schema is strict, stale
snapshots, and cross-repository evidence. Legacy records remain immutable and
are projected as historical/unknown when they cannot satisfy current identity
requirements; they are not silently rewritten or upgraded in place.

The Rust records are semantically compatible with the reference responsibilities
but are not direct JSON-wire or Python-module compatibility. Reference
`.ai/project_profile.yaml`, `.ai/cockpit/checks.yaml`, generated status files,
and source-specific registries are comparison material unless a Rust-native
counterpart is explicitly documented.
