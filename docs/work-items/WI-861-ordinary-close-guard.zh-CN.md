---
author: AI Cockpit maintainers
title: "WI-861——普通 close 拓扑保护"
description: "在绑定 cleanup evidence 前，拒绝从主 checkout 或发现的默认 checkout 执行普通 close。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-861-ordinary-close-guard
lastVerifiedBy: WI-861-ordinary-close-guard
terminalArchive: .ai/work-items/archive/WI-861-ordinary-close-guard.contract.json
terminalVerification: .ai/evidence/WI-861-ordinary-close-guard.verification.json
terminalDecision: .ai/decisions/WI-861-ordinary-close-guard.close.json
---

[English](WI-861-ordinary-close-guard.md) · [日本語](WI-861-ordinary-close-guard.ja.md)

# WI-861——普通 close 拓扑保护

本 Work Item 在绑定不可能完成的 cleanup obligation 之前，拒绝从仓库主
checkout 或发现的默认 checkout 执行普通 close。受支持的 close 上下文仍是
非默认分支上的专用 linked worktree，保护逻辑不会写死分支名称。
