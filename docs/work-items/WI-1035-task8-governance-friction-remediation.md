---
author: AI Cockpit maintainers
workItemId: WI-1035-task8-governance-friction-remediation
title: Task 8 governance-friction remediation
description: Resolve the complete 20-item Task 8 governance-friction inventory in one serial Work Item, expose usable Agent collaboration capabilities, and verify the resulting behavior before the release-review boundary.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1035-task8-governance-friction-remediation
---

[简体中文](WI-1035-task8-governance-friction-remediation.zh-CN.md) · [日本語](WI-1035-task8-governance-friction-remediation.ja.md)

# WI-1035 — Task 8 governance-friction remediation

This Work Item follows the WI-1033 handoff and keeps all 20 observed governance frictions in one implementation stream: the original 17 handoff items, two independently reproduced close-projection and delivery-communication issues, and the general verification-evidence reuse process. The authoritative acceptance criteria, inventory, implementation stages, and evidence plan are in the Runtime-bound Contract and [Task 8 specification](WI-1034-task8-governance-friction-remediation/spec.md) with its [implementation plan](WI-1034-task8-governance-friction-remediation/implementation-plan.md).

## Boundaries

- Implementation is serial in one Work Item, branch, worktree, and repository context; it is organized into reviewable commits rather than one oversized commit.
- Agent collaboration capabilities must be discoverable and actually usable. Immediately after exposure, acceptance must inspect the manifest and MCP schemas and exercise real multi-process, multiple-linked-worktree coordination as well as the single-WI serial fallback.
- Before any declared verification, inspect Runtime freshness and existing receipt bindings; reuse complete fresh evidence, rerun only missing or invalidated checks, and keep exact-PR-head hosted evidence distinct from local receipts.
- Verification, PR review, merge, cleanup, and Work Item lifecycle remain distinct. This task stops before release; it does not create a tag or publish a version.
- Current state is in progress. No acceptance item or verification result is claimed complete by this projection.

## Acceptance outline

All 20 Contract criteria must be linked to current tests or real acceptance evidence, the canonical CI gate, and Runtime-bound evidence. Missing evidence, stale candidate identity, or an unverified required scenario remains a blocker.

Hosted verification uses a gate-manifested shared runner that follows the candidate Runtime's current action admission, refreshes preflight only when admitted, rechecks admission immediately before verification, and derives workspace coverage from the same executable-bound receipt without rerunning package checks.

The `verify` command's stdout is only an execution summary. The shared runner keeps it separate and copies the Runtime-authored `.ai/evidence/<WI>.verification.json` formal envelope byte-for-byte as the hosted receipt. Coverage validates both receipt identity layers and the complete Cargo package/node set, then requires exact byte equality with the repository evidence; malformed, mismatched, or reconstructed receipts fail closed.

A same-version candidate Runtime rebuild makes a fully valid predecessor receipt stale when its executable digest changes; it never reuses that receipt or asks for a redundant retry decision. The helper follows the candidate's admitted next action, while a Runtime version change or invalid identity/evidence retains its explicit fail-closed recovery boundary.
