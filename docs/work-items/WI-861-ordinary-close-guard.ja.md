---
author: AI Cockpit maintainers
title: "WI-861 — ordinary close topology guard"
description: "cleanup evidence を束縛する前に primary または検出された default checkout からの ordinary close を拒否する。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-861-ordinary-close-guard
lastVerifiedBy: WI-861-ordinary-close-guard
terminalArchive: .ai/work-items/archive/WI-861-ordinary-close-guard.contract.json
terminalVerification: .ai/evidence/WI-861-ordinary-close-guard.verification.json
terminalDecision: .ai/decisions/WI-861-ordinary-close-guard.close.json
---

[English](WI-861-ordinary-close-guard.md) · [简体中文](WI-861-ordinary-close-guard.zh-CN.md)

# WI-861 — ordinary close topology guard

この Work Item は、実行不能な cleanup obligation を束縛する前に、repository
の primary または検出された default checkout からの ordinary close を拒否
します。対応する close context は non-default branch の dedicated linked
worktree であり、branch 名を hardcode しません。
