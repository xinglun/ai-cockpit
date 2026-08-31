---
author: AI Cockpit maintainers
title: "WI-443 — 历史 finalization 兼容"
workItemId: WI-443-historical-finalization-compat
description: "为旧共享 worktree 与无 PR 本地合并提供诚实 recovery 路径。"
audience: [maintainer, adopter, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-443-historical-finalization-compat
terminalArchive: .ai/work-items/archive/WI-443-historical-finalization-compat.contract.json
terminalVerification: .ai/evidence/WI-443-historical-finalization-compat.verification.json
terminalFinalization: .ai/decisions/WI-443-historical-finalization-compat.finalize.json
terminalDecision: .ai/decisions/WI-443-historical-finalization-compat.close.json
---

# WI-443 — 历史 finalization 兼容

为旧共享主 worktree 与无 PR 本地合并增加显式 `historical_low` 记录。direct merge
必须绑定真实 commit、parents、base、repository identity 与 authority；Runtime 对照 Git
校验，绝不编造 PR。Readiness 分类仓库级历史债务并给出 recovery action，不削弱新入口门槛。

对象工程在此 Runtime 发布并重新验收前保持冻结。

[English](WI-443-historical-finalization-compat.md) · [日本語](WI-443-historical-finalization-compat.ja.md)
