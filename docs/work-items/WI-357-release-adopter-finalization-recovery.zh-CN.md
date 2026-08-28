---
author: AI Cockpit maintainers
title: "WI-357——发布 adopter 收尾恢复"
workItemId: WI-357-release-adopter-finalization-recovery
description: "在不改写不可变前置证据的前提下，把 WI-356 交付重新绑定到真实 provider 上下文。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-357-release-adopter-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-357-release-adopter-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-357-release-adopter-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-357-release-adopter-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-357-release-adopter-finalization-recovery.close.json
predecessor: WI-356-release-adopter-script-order
capabilityClaims:
  - adopter_finalization_recovery
---

# WI-357——发布 adopter 收尾恢复

[English](WI-357-release-adopter-finalization-recovery.md) · [日本語](WI-357-release-adopter-finalization-recovery.ja.md)

## 意图与边界

WI-357 是 WI-356 的显式恢复 successor。WI-356 已作为 PR #321 合并，
但其不可变 archive 在 PR 创建之前生成，因此保留了 provisional 的
`pending` resource context。本 Work Item 通过新的可审计生命周期记录真实
provider 绑定与精确清理；绝不改写 WI-356 的 archive、evidence 或 outcome bytes。

范围仅包括 recovery decision、三语治理记录、已审阅 PR 的资源绑定以及
finalization/close evidence。Runtime 功能变更、adopter harness 行为、版本发布
和 provider 自动化不在本边界内。

## 交付与验证

- recovery receipt 绑定前置 Contract、Summary、Outcome 和 Events digest，
  并明确链接 WI-356 与 PR #321。
- 在记录 verification evidence 之前，用独立 branch/worktree 和 reviewed PR
  绑定真实 GitHub 上下文。
- `finalize-verify` 必须证明 PR 已合并、默认分支已同步，并且本地/远端分支与
  worktree 已精确清理，之后才能结构化 close。
- 所有 receipt 均保持 repository 绑定并由已安装 Runtime 生成；前置历史 bytes 保持不可变。

## 恢复边界

由于不可变前置 Contract 中的 provisional resource context 不能被真实 provider
receipt 替换，因此必须使用 successor 的显式恢复路径。该路径 fail-closed、可审阅，
不使用伪造 URL，也不编辑前置记录。
