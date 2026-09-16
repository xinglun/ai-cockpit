---
author: AI Cockpit maintainers
title: "WI-862——普通 cleanup binding 连续性"
description: "当 close 接受合法历史 verification evidence 时，仍保留准确的普通 cleanup binding。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-862-ordinary-cleanup-binding
lastVerifiedBy: WI-862-ordinary-cleanup-binding
terminalArchive: .ai/work-items/archive/WI-862-ordinary-cleanup-binding.contract.json
terminalVerification: .ai/evidence/WI-862-ordinary-cleanup-binding.verification.json
terminalDecision: .ai/decisions/WI-862-ordinary-cleanup-binding.close.json
---

[English](WI-862-ordinary-cleanup-binding.md) · [日本語](WI-862-ordinary-cleanup-binding.ja.md)

# WI-862——普通 cleanup binding 连续性

本 Work Item 将工作结果保证与 cleanup 身份分开。当 close 接受合法的历史
verification receipt 时，不能因此抑制 Runtime 所有的 branch 和 linked-worktree
binding。
