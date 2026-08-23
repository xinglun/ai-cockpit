---
author: AI Cockpit maintainers
title: "WI-198——治理质量门默认分支发现"
description: "让分离的 pull request 检出中的 pre-merge 治理校验保持确定性，同时不削弱 identity 检查。"
audience:
  - maintainer
  - reviewer
workItemId: WI-198-governance-gate-default-branch-discovery
status: in_progress
authority: canonical
lastVerifiedBy: WI-198-governance-gate-default-branch-discovery
---

# WI-198——治理质量门默认分支发现

WI-198 是不可变 WI-197 的明确 successor。托管 quality 表明，detached
pull request merge checkout 可能同时缺少 `origin/HEAD` 和事件中的 base
branch 元数据。质量门现在只把 Contract 不可变的
`resourceContext.baseBranch` 作为窄范围回退，同时继续强制 repository、PR、分支、
worktree、证据、Runtime 和 digest 的全部绑定。

回归测试覆盖没有元数据的有效检出，以及外部声明的 base branch 不一致场景。WI-197
保持不可变，并通过 recovery 回执链接。

[English](WI-198-governance-gate-default-branch-discovery.md) ·
[日本語](WI-198-governance-gate-default-branch-discovery.ja.md)
