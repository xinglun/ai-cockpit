---
author: AI Cockpit maintainers
title: "WI-832 — adopter verification reuse"
description: "Keep staged and upgrade adopter verification aligned with the Runtime's package route."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-832-release-adopter-reuse
lastVerifiedBy: WI-832-release-adopter-reuse
---

[简体中文](WI-832-release-adopter-reuse.zh-CN.md) · [日本語](WI-832-release-adopter-reuse.ja.md)

# WI-832 — adopter verification reuse

## Intent

The adopter harness must prove that a successful verification can be reused.
The profile confirmation therefore records the exact command executed by the
Runtime's workspace package route: `cargo test --locked --package adopter`.

## Boundary

Staged and N−1 upgrade acceptance use the same package-scoped command. The
regression checks preserve the first receipt, require a second verification to
report zero spawned processes, and do not change Runtime reuse semantics or
release identities.

## Verification

- Static acceptance and upgrade harness checks pass.
- A real v0.2.92 staged candidate run reports `nodesReused: 1` and
  `processesSpawned: 0` on the second verification.

## Out of scope

Runtime reuse protocol changes, product builds, Release tags and assets,
historical Work Items, and unrelated cleanup.
