---
author: AI Cockpit maintainers
title: "WI-861 — ordinary close topology guard"
description: "Reject ordinary close from the primary or discovered default checkout before binding cleanup evidence."
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-861-ordinary-close-guard
lastVerifiedBy: WI-861-ordinary-close-guard
terminalArchive: .ai/work-items/archive/WI-861-ordinary-close-guard.contract.json
terminalVerification: .ai/evidence/WI-861-ordinary-close-guard.verification.json
terminalDecision: .ai/decisions/WI-861-ordinary-close-guard.close.json
---

[简体中文](WI-861-ordinary-close-guard.zh-CN.md) · [日本語](WI-861-ordinary-close-guard.ja.md)

# WI-861 — ordinary close topology guard

This Work Item rejects ordinary close from the repository primary or discovered
default checkout before it can bind an impossible cleanup obligation. A
dedicated linked worktree on a non-default branch remains the supported close
context, and the guard does not hardcode a branch name.
