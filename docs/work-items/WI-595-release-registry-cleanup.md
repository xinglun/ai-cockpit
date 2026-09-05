---
author: AI Cockpit maintainers
title: "WI-595 — release registry cleanup"
description: "Remove the stale pending-parity projection after WI-594 is closed."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-595-release-registry-cleanup
lastVerifiedBy: WI-595-release-registry-cleanup
terminalArchive: .ai/work-items/archive/WI-595-release-registry-cleanup.contract.json
terminalVerification: .ai/evidence/WI-595-release-registry-cleanup.verification.json
terminalFinalization: .ai/decisions/WI-595-release-registry-cleanup.finalize.b6cc3d25fa5d2ccdadcc9e441e9944f87e0ecd381cc845386f0ddd1f88f7adec.json
terminalDecision: .ai/decisions/WI-595-release-registry-cleanup.close.json
---

[简体中文](WI-595-release-registry-cleanup.zh-CN.md) · [日本語](WI-595-release-registry-cleanup.ja.md)

# WI-595 — release registry cleanup

## Objective

Remove the stale WI-594 entry from `docs/reference/pending-parity-registry.json`
and keep the three-language parity projection aligned with the closed Runtime
records. Historical `.ai/` bytes remain immutable.

## Boundary

This documentation Work Item changes only the pending registry, parity
projections, and the readable WI-594/WI-595 pages. Runtime behavior, release
artifacts, object repositories, and global Agent/MCP configuration are out of scope.

## Verification

Run the JSON parser, `tests/docs/parity_status_check.sh`, the tag-mode
`tests/ci/governance_integrity_gate.py`, documentation acceptance, and status
consistency checks with an explicit repository context.
