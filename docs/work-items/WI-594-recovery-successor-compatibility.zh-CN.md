---
author: AI Cockpit maintainers
title: "WI-594——恢复 successor compatibility Runtime 修复"
description: "为有效的 successor 重验证提供追加式 Runtime 收尾路径。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-594-recovery-successor-compatibility
lastVerifiedBy: WI-594-recovery-successor-compatibility
terminalArchive: .ai/work-items/archive/WI-594-recovery-successor-compatibility.contract.json
terminalVerification: .ai/evidence/WI-594-recovery-successor-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-594-recovery-successor-compatibility.finalize.398ee773f1fe88e7e80c09c29b12129d2e1289bc35e7a555421836702d86dc49.json
terminalDecision: .ai/decisions/WI-594-recovery-successor-compatibility.close.json
---

[English](WI-594-recovery-successor-compatibility.md) · [日本語](WI-594-recovery-successor-compatibility.ja.md)

# WI-594——恢复 successor compatibility Runtime 修复

## 目标

允许有效的 successor/revalidation 记录在不重写旧 recovery 或 finalization
字节的前提下关闭归档前置 Work Item。无效、外部或矛盾记录继续 fail-closed。

## 边界

这是 repository-bound、append-only 的 Runtime 修复，不把 PR finalization
重分类为 direct merge，不修改对象工程或发布制品。

## 验证

归档 Work Item 具有当前验证证据、已验证的 finalization head 和结构化 close
决定；head 绑定了准确的 reviewed PR、merge、分支及 worktree 清理事实。
