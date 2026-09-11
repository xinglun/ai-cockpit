---
author: AI Cockpit maintainers
title: "WI-797——残留资源清理"
description: "在 v0.2.90 交付后关闭已取代的 PR，并仅删除清单中的分支和工作树。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-release-and-residual-cleanup
workItemId: WI-797-residual-resource-cleanup
lastVerifiedBy: WI-797-residual-resource-cleanup
terminalArchive: .ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json
terminalVerification: .ai/evidence/WI-797-residual-resource-cleanup.verification.json
---

[English](WI-797-residual-resource-cleanup.md) · [日本語](WI-797-residual-resource-cleanup.ja.md)

# WI-797——残留资源清理

## 目标与边界

WI-797 记录 v0.2.90 交付后发现的残留资源的有界清理。范围仅包括清单中的已取代
PR、远端分支、本地分支和工作树。已关闭的 PR 作为不可变历史保留；准确的恢复
stash 也保留以支持恢复。

本 WI 不修改产品或 Runtime 行为、已发布版本字节、历史治理记录、无关分支或工作树，
也不修改全局 Agent/MCP 配置。

## 当前生命周期状态

清理清单和动作已通过 Runtime 验证并归档。本 WI 的 provider merge、finalization、
close 及精确资源清理仍是终态边界；因此在这些记录完成验证前，本页面保持
`in_progress`。

## 验证边界

- archive：`.ai/work-items/archive/WI-797-residual-resource-cleanup.contract.json`
- verification：`.ai/evidence/WI-797-residual-resource-cleanup.verification.json`
- reviewed PR 及 Runtime finalization/close 记录只会在相应的 provider 和 Runtime
  转换实际发生后加入。

三语页面和 parity 行保留相同事实及相同的待处理操作后果。页面本身不授予绕过
provider 或 Runtime 门禁的权限。
