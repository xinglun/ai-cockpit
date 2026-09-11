---
author: AI Cockpit maintainers
title: "WI-797 — residual resource cleanup"
description: "Close superseded Pull Requests and remove only the inventoried branches and worktrees after the v0.2.90 delivery."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-release-and-residual-cleanup
workItemId: WI-797-residual-resource-cleanup
lastVerifiedBy: WI-797-residual-resource-cleanup
terminalArchive: .ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json
terminalVerification: .ai/evidence/WI-797-residual-resource-cleanup.verification.json
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

## Current lifecycle state

The cleanup inventory and actions have been verified and archived through the
Runtime. Provider merge, finalization, close, and exact cleanup of this Work
Item remain the terminal lifecycle boundary; this page therefore remains
`in_progress` until those records are verified.

## Verification boundary

- archive: `.ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json`
- verification: `.ai/evidence/WI-797-residual-resource-cleanup.verification.json`
- the reviewed PR and Runtime finalization/close records will be added only
  after their corresponding provider and Runtime transitions occur.

The three language pages and parity rows preserve the same facts and the same
pending operational consequence. No page grants authority to bypass provider
or Runtime gates.
