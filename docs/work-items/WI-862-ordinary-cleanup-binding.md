---
author: AI Cockpit maintainers
title: "WI-862 — ordinary cleanup binding continuity"
description: "Keep the exact ordinary cleanup binding when close accepts valid historical verification evidence."
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-862-ordinary-cleanup-binding
lastVerifiedBy: WI-862-ordinary-cleanup-binding
terminalArchive: .ai/work-items/archive/WI-862-ordinary-cleanup-binding.contract.json
terminalVerification: .ai/evidence/WI-862-ordinary-cleanup-binding.verification.json
terminalDecision: .ai/decisions/WI-862-ordinary-cleanup-binding.close.json
---

[简体中文](WI-862-ordinary-cleanup-binding.zh-CN.md) · [日本語](WI-862-ordinary-cleanup-binding.ja.md)

# WI-862 — ordinary cleanup binding continuity

This Work Item keeps work-result assurance separate from cleanup identity.
Accepting a valid historical verification receipt must not suppress the
Runtime-owned branch and linked-worktree binding captured by an ordinary close.
