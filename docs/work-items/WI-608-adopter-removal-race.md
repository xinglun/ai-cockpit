---
title: "WI-608 — race-safe adopter checkout cleanup"
description: "Release acceptance cleanup with bounded retries and fail-closed receipts."
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
lastVerifiedBy: WI-608-adopter-removal-race
terminalArchive: .ai/work-items/archive/WI-608-adopter-removal-race.contract.json
terminalVerification: .ai/evidence/WI-608-adopter-removal-race.verification.json
terminalFinalization: .ai/decisions/WI-608-adopter-removal-race.finalize.json
terminalDecision: .ai/decisions/WI-608-adopter-removal-race.close.json
workItemId: WI-608-adopter-removal-race
---

[简体中文](WI-608-adopter-removal-race.zh-CN.md) · [日本語](WI-608-adopter-removal-race.ja.md)

# WI-608 — race-safe adopter checkout cleanup

## Purpose

The release adopter harness removes only its exact temporary checkout. A
bounded retry tolerates a transient Git maintenance race, while cleanup
failure remains explicit and preserves the acceptance receipt.

## Boundary

This change covers the staged/public adopter and N-1 acceptance scripts,
their regression wrappers, and the release workflow's authentication handoff.
It does not change Runtime governance semantics, object repositories, or
global Agent/MCP configuration.

## Evidence

- Archive: `.ai/work-items/archive/WI-608-adopter-removal-race.archive.json`
- Verification: `.ai/evidence/WI-608-adopter-removal-race.verification.json`
- Finalization: `.ai/decisions/WI-608-adopter-removal-race.finalize.json`
- Close: `.ai/decisions/WI-608-adopter-removal-race.close.json`
