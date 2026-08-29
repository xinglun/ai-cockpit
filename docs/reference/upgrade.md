---
author: AI Cockpit maintainers
title: Upgrade
description: Upgrade an installed shared Runtime and repository attachment without confusing it with project readiness.
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - runtime_upgrade
---

# Upgrade

[English](upgrade.md) · [简体中文](upgrade.zh-CN.md) · [日本語](upgrade.ja.md)

An installed Runtime upgrade and a repository schema migration are different
operations. Runtime-only upgrade normally changes the machine's shared binary
and leaves repository `.ai/` bytes unchanged. A migration is an explicit,
reviewed repository Work Item with a plan, backup/rollback evidence, and human
decision.

## Runtime upgrade

Use an immutable public Release archive and verify its manifest, SHA-256, and
runtime identity before installation. Keep the current Runtime available for
rollback until the new binary passes its local doctor and release acceptance.
After installation, every repository still requires explicit attachment and
request-scoped commands:

```sh
ai-cockpit inspect --repo /path/to/project
ai-cockpit compatibility --repo /path/to/project
ai-cockpit doctor --repo /path/to/project
```

The Runtime does not commit, push, open/merge a PR, or edit global Agent/MCP
configuration. Managed adapter changes, if any, are a separate explicit
`agent install` Work Item in the target repository.

## Repository migration

Run `ai-cockpit migrate plan --repo <path>` first. Apply only the reviewed plan
with the explicit approval required by the command. Migration must preserve
Contract, evidence, decision, knowledge, and archive history; it must never
rewrite old evidence merely because the Runtime version changed. If a migration
is incomplete or incompatible, read-only diagnostics remain available while
stateful lifecycle writes fail closed.

The reference source's installer, `Makefile.ai`, Python modules, and provider
marker files are not copied into this Rust repository. The semantic boundary is
the shared external Runtime plus an isolated repository Protocol.
