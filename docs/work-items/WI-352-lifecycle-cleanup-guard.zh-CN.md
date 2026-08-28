---
author: AI Cockpit 维护者
title: "WI-352——生命周期清理门"
workItemId: WI-352-lifecycle-cleanup-guard
description: "让未完成的生命周期清理对仓库可见，并对 release-adopter 运行 fail-closed。"
audience: [maintainer, reviewer]
status: in_progress
authority: translation
canonical: docs/work-items/WI-352-lifecycle-cleanup-guard.md
lastVerifiedBy: WI-352-lifecycle-cleanup-guard
terminalArchive: .ai/work-items/archive/WI-352-lifecycle-cleanup-guard.contract.json
terminalVerification: .ai/evidence/WI-352-lifecycle-cleanup-guard.verification.json
capabilityClaims: [lifecycle_governance, cleanup_handoff]
---

# WI-352——生命周期清理门

[English](WI-352-lifecycle-cleanup-guard.md) · [日本語](WI-352-lifecycle-cleanup-guard.ja.md)

## 意图与边界

让缺少或无效 close 证据的已归档 Work Item 明确保持非终态。Runtime 必须在
status 和面向人的 Outcome 中给出精确的清理/finalization/close 下一步，同时保持
仓库本地状态和共享 Runtime 边界。release-adopter harness 必须在写入收据后，无论
成功或失败都删除隔离运行根目录；清理不能改写验收事实。

## 验证

- 已归档但未 close 的状态是阻断、黄色且可操作的；不能报告为绿色，也不能允许
  下一个 Work Item。
- 有效 finalization 和 close 仍绑定已审查 PR、分支、工作树、仓库和 Runtime 身份。
- 成功与失败路径都测试 harness/wrapper 清理，并保持 HOME/XDG_CONFIG_HOME 禁止写入，
  TMPDIR/CARGO_HOME 作为隔离的 Runtime 写入根。
- 英语、简体中文和日语文档表达相同边界，并保留不可变 archive/evidence 记录。

## 交付状态

实现和验证证据已经归档。审查中的 PR 仍需 provider finalization、精确资源清理以及
结构化 close 决定，之后本 Work Item 才能成为终态。
