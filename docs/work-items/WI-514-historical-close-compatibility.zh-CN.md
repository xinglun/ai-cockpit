---
author: AI Cockpit maintainers
title: "WI-514——历史收尾兼容"
description: "为旧版共享 worktree 与 direct-merge 提供窄范围、证据绑定的恢复投影，不改写历史。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-514-historical-close-compatibility
lastVerifiedBy: WI-514-historical-close-compatibility
terminalArchive: .ai/work-items/archive/WI-514-historical-close-compatibility.contract.json
terminalVerification: .ai/evidence/WI-514-historical-close-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-514-historical-close-compatibility.finalize.json
terminalDecision: .ai/decisions/WI-514-historical-close-compatibility.close.json
---

[English](WI-514-historical-close-compatibility.md) · [日本語](WI-514-historical-close-compatibility.ja.md)

## 目标

Runtime 升级后，在不改写不可变前置收据的前提下，重新验证诚实的历史收尾记录；历史证据不得被当作当前高 assurance 证据。

## 范围与边界

- 只有本地 provider、主 checkout 共享 worktree、`retained`，且分支、worktree、仓库、Contract 与 clean 状态全部可验证绑定时，才投影为 `historical_low`。
- 普通 retained linked worktree、外部 provider、拓扑不明确、格式错误或事实过期时继续 fail closed。
- 历史 direct-merge 使用真实 merge commit、parents、base revision 与 repository identity，绝不虚构 PR 号。

## 证据

- `.ai/evidence/WI-514-historical-close-compatibility.verification.json`
- `crates/cockpit-repository/tests/resource_finalization_transition.rs`
- `docs/reference/work-item-lifecycle-closure.zh-CN.md`

archive、recovery 与 projection 只追加仓库绑定的恢复事实，保留原始收据 bytes。

## 不主张的内容

本 WI 不修改对象工程、provider 授权、发布打包或无关生命周期策略；`historical_low` 不是新的绿色验证结果。
