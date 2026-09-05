---
author: AI Cockpit maintainers
title: "WI-594 — recovery successor compatibility Runtime fix"
description: "Provide an append-only Runtime path for closing an archived predecessor after valid successor revalidation."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-594-recovery-successor-compatibility
lastVerifiedBy: WI-594-recovery-successor-compatibility
terminalArchive: .ai/work-items/archive/WI-594-recovery-successor-compatibility.contract.json
terminalVerification: .ai/evidence/WI-594-recovery-successor-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-594-recovery-successor-compatibility.finalize.398ee773f1fe88e7e80c09c29b12129d2e1289bc35e7a555421836702d86dc49.json
terminalDecision: .ai/decisions/WI-594-recovery-successor-compatibility.close.json
---

[简体中文](WI-594-recovery-successor-compatibility.zh-CN.md) · [日本語](WI-594-recovery-successor-compatibility.ja.md)

# WI-594 — recovery successor compatibility Runtime fix

## Objective

Allow a valid successor/revalidation record to close an archived predecessor
without rewriting older recovery or finalization bytes. Invalid, foreign, or
contradictory records remain fail-closed.

## Boundary

This Runtime change is repository-bound and append-only. It does not reclassify
PR finalization as direct merge, modify object repositories, or change release
artifacts.

## Verification

The archived Work Item has current verification evidence, a verified finalization
head, and a structured close decision. The finalization head records the exact
reviewed PR, merge, branch, and worktree cleanup facts.
