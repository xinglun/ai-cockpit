---
author: AI Cockpit maintainers
title: "WI-985 — repository resource lifecycle boundary"
description: "Extract resource finalization, ordinary cleanup, and close-time resource validation into a same-crate module while preserving public APIs, receipts, errors, and recovery behavior."
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:user
workItemId: WI-985-repository-resource-lifecycle
lastVerifiedBy: WI-985-repository-resource-lifecycle
---

[简体中文](WI-985-repository-resource-lifecycle.zh-CN.md) · [日本語](WI-985-repository-resource-lifecycle.ja.md)

# WI-985 — repository resource lifecycle boundary

This Work Item tightens an existing same-crate boundary in
`cockpit-repository`. Resource finalization, ordinary cleanup, and close-time
resource validation will move to `resource_lifecycle.rs`; `lib.rs` will retain
stable public exports and generic repository primitives.

The change preserves public signatures, JSON and receipt schemas, file paths,
write order, error semantics, historical reads, recovery behavior, and
fail-closed handling of duplicate, stale, foreign, dirty, missing, or unknown
resources. It does not claim a runtime performance improvement and does not
change protocol, CLI/MCP, release scripts, or legacy compatibility behavior.

Acceptance is based on focused finalization and cleanup regressions, archive/
close and recovery regressions, the Contract-declared repository and CLI/MCP
tests, and the Runtime-bound verification evidence.
