---
author: AI Cockpit maintainers
title: "WI-797 — residual resource cleanup"
description: "Close superseded Pull Requests and remove only the inventoried branches and worktrees after the v0.2.90 delivery."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized-release-and-residual-cleanup
workItemId: WI-797-residual-resource-cleanup
lastVerifiedBy: WI-797-residual-resource-cleanup
terminalArchive: .ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json
terminalVerification: .ai/evidence/WI-797-residual-resource-cleanup.verification.json
terminalFinalization: .ai/decisions/WI-797-residual-resource-cleanup.finalize.66aa81cb9de929b6c46e8a4020ae5349db3cbb3b58db47e6e53a3284aa7caaf5.json
terminalDecision: .ai/decisions/WI-797-residual-resource-cleanup.close.json
---

[简体中文](WI-797-residual-resource-cleanup.zh-CN.md) · [日本語](WI-797-residual-resource-cleanup.ja.md)

# WI-797 — residual resource cleanup

## Objective and boundary

WI-797 records the bounded cleanup of the residual resources found after the
v0.2.90 delivery. It covers only the inventoried superseded Pull Requests,
remote branches, local branches, and worktrees. The closed Pull Requests are
retained as immutable history; exact recovery stashes are retained for
recoverability.

It does not change product or Runtime behavior, published release bytes,
historical governance records, unrelated branches or worktrees, or global
Agent/MCP configuration.

## Terminal lifecycle state

The cleanup inventory and actions have been verified and archived through the
Runtime. PR #774 passed its hosted checks and merged as
`01caa7f3c7bfce9ec0add01ef98ea858495acd42`. The Runtime then recorded the
merge observation and the exact deletion of the WI-797 branch and worktree;
the PR history and four recovery stashes remain retained.

## Verification boundary

- archive: `.ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json`
- verification: `.ai/evidence/WI-797-residual-resource-cleanup.verification.json`
- finalization: `.ai/decisions/WI-797-residual-resource-cleanup.finalize.66aa81cb9de929b6c46e8a4020ae5349db3cbb3b58db47e6e53a3284aa7caaf5.json`
- close: `.ai/decisions/WI-797-residual-resource-cleanup.close.json`

The three language pages and parity rows preserve the same facts and
operational consequence. No page grants authority to bypass provider or
Runtime gates.
