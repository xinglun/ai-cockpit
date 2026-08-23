---
author: AI Cockpit maintainers
title: "WI-199——治理质量门实际 PR 上下文"
description: "在合并前把 detached checkout 治理质量门修正绑定到实际审阅分支和 pull request。"
audience:
  - maintainer
  - reviewer
workItemId: WI-199-governance-gate-actual-pr-context
status: in_progress
authority: canonical
lastVerifiedBy: WI-199-governance-gate-actual-pr-context
---

# WI-199——治理质量门实际 PR 上下文

WI-199 是不可变 WI-198 的明确 successor。归档后自检发现，分支改名后 WI-198 仍保留
旧的 branch 名称。默认分支发现实现保持不变；本 Work Item 把审阅中的 PR #153 绑定到
实际分支，并记录严格的 pre-merge finalization 回执。

[English](WI-199-governance-gate-actual-pr-context.md) ·
[日本語](WI-199-governance-gate-actual-pr-context.ja.md)
