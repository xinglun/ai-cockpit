---
author: AI Cockpit maintainers
title: "WI-372——WI-370 provider finalization 恢复"
description: "绑定已审查的 PR 身份，在不重写不可变前置字节的前提下关闭性能 Work Item。"
workItemId: WI-372-wi370-finalization-recovery
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-372-wi370-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-372-wi370-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-372-wi370-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-372-wi370-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-372-wi370-finalization-recovery.close.json
capabilityClaims: [governance_integrity, resource_finalization]
---

# WI-372——WI-370 provider finalization 恢复

[English](WI-372-wi370-finalization-recovery.md) · [日本語](WI-372-wi370-finalization-recovery.ja.md)

## 意图与边界

WI-370 在获知已审查 PR 身份之前就已归档，因此不可变 resource context 中保留了占位的
pull-request URL。本有界 successor 会在重新验证前记录真实 PR #333 身份，并完成精确的
branch/worktree finalization。前置 Work Item 的 Contract、verification、archive 和 outcome
字节保持不可变。

## 验收

- 在重新验证前绑定真实的已审查 PR #333 context。
- 保持前置 archive 和 evidence 字节不变。
- 在 close 前验证精确 branch 和 worktree 已删除。
- 记录 hosted review、finalization 和可见的人类 Outcome。

本任务是治理恢复，不修改 Runtime 性能或发布产物。
