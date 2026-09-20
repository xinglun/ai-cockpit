---
author: AI Cockpit maintainers
title: "WI-957 — start 授权值与 typed 验证校验"
description: "在创建活跃 Work Item 状态前拒绝不受支持的授权值，并以声明身份执行 typed 必需验证。"
audience: [maintainer, reviewer, contributor]
status: implemented
authority: human:sei-rinn
workItemId: WI-957-start-authority-recovery
lastVerifiedBy: WI-957-start-authority-recovery
terminalArchive: .ai/work-items/archive/WI-957-start-authority-recovery.contract.json
terminalVerification: .ai/evidence/WI-957-start-authority-recovery.verification.json
terminalDecision: .ai/decisions/WI-957-start-authority-recovery.close.json
---

[English](WI-957-start-authority-recovery.md) · [日本語](WI-957-start-authority-recovery.ja.md)

# WI-957 — start 授权值与 typed 验证校验

Runtime 会在写入活跃 Contract 或 Summary 前拒绝不受支持的授权值。
CLI 也会以 typed 必需验证声明的 `check` 身份计划该验证，使验证 receipt
能够满足同名的 finish 门禁，而不会回退到无关的默认 workspace 测试。
当 checkpoint 后的追加 amendment 使该证据失效时，允许启动一次当前替换验证；
无关的控制失败仍会在子进程启动前阻止执行。
