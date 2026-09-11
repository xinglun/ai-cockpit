---
author: AI Cockpit maintainers
title: "WI-796——WI-795 按顺序重新交付文档的 successor"
description: "保留 WI-795 的不可变顺序失败，并完成有序的文档晋级。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized-successor-recovery
workItemId: WI-796-wi795-doc-promotion-retry
lastVerifiedBy: WI-796-wi795-doc-promotion-retry
terminalArchive: .ai/work-items/archive/WI-796-wi795-doc-promotion-retry.contract.json
terminalVerification: .ai/evidence/WI-796-wi795-doc-promotion-retry.verification.json
terminalFinalization: .ai/decisions/WI-796-wi795-doc-promotion-retry.finalize.e98e9a07ee6e979dade4b7884f937e1ceb9e298d41c937e59a544ef5d3a86c43.json
terminalDecision: .ai/decisions/WI-796-wi795-doc-promotion-retry.close.json
predecessorWorkItemId: WI-795-wi794-doc-promotion
recoveryDecision: .ai/decisions/WI-795-wi794-doc-promotion.recovery.json
---

[English](WI-796-wi795-doc-promotion-retry.md) · [日本語](WI-796-wi795-doc-promotion-retry.ja.md)

# WI-796——WI-795 按顺序重新交付文档的 successor

## 恢复边界

WI-796 是 WI-795 的有边界 successor。WI-795 作为不可变证据保留：它的
parity 登记和验证证据在同一提交中引入，因此如果不重写历史就无法修复所需的
“先登记、后证据”顺序。Runtime 的恢复决定保留这一事实；本 successor 提供
新的有序文档边界。

WI-796 只修改文档投影及其 Runtime 治理记录。不修改 Rust 行为、Runtime 行为、
发布制品、授权语义，也不修改任何前序 archive、evidence、finalization 或
recovery 字节。

## 验收

- WI-794 终态投影继续绑定其不可变 archive、verification、finalization 和
  close 记录。
- WI-796 自身的三语页面和 parity 行在引入新的验证证据前登记。
- 英文、简体中文和日文保留相同的治理事实、未知项、恢复边界和操作后果。
- 文档、parity、status-consistency 和 governance-integrity 检查通过，且不重写
  前序记录。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-794-release-v0-2-90-closure --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
