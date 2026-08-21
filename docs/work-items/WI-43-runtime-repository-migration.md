---
author: AI Cockpit maintainers
title: "WI-43 — Runtime compatibility and Repository Migration Protocol"
description: "The implementation contract and user-visible boundary for Runtime-only upgrades and explicit repository migrations."
audience:
  - maintainer
  - adopter
status: current
authority: canonical
lastVerifiedBy: implementation-acceptance
capabilityClaims:
  - runtime_upgrade_boundary
  - repository_migration
---

# WI-43 — Runtime compatibility and Repository Migration Protocol

## Goal

Keep one shared Runtime upgradeable without silently changing a repository's
governance state. A Runtime-only upgrade leaves `.ai/` unchanged. A repository
schema change is explicit, reviewable, approved, and receipt-bound.

## User flow

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
ai-cockpit migrate apply --repo /path/to/repository --approved
```

The compatibility states are:

- `COMPATIBLE`: normal lifecycle, Agent, MCP, and verification operations may run;
- `MIGRATION_REQUIRED`: inspect and read-only planning remain available, while
  state-changing or evidence-producing operations stop;
- `INCOMPATIBLE`: fail closed until a Runtime supporting the stored schema is installed.

The current Repository Protocol is version 1. The current repository schema
target is version 2. Existing schema-1 files are read as legacy state and are
never upgraded by `status`, `attach`, or a normal Runtime invocation.

## Receipt and preservation rules

An applied migration writes `.ai/migrations/<migration-id>.json` with the source
and target schema, before/after digests, Runtime version, Runtime digest, changed
protocol files, and result. It may update only the versioned protocol files and
the migration record. Archived Work Items, evidence, decisions, knowledge, and
other historical records remain unchanged. There is no global current repository
or global Work Item in the Runtime.

## Acceptance

- old schema defaults to version 1 and reports `MIGRATION_REQUIRED`;
- `migrate plan` is read-only and declares human approval;
- `migrate apply` without `--approved` fails without changing bytes;
- an approved migration reaches `COMPATIBLE` and emits the Runtime-bound receipt;
- repeated application is refused;
- historical evidence and archived Work Item bytes are unchanged;
- all repository commands continue to require explicit `--repo`.
