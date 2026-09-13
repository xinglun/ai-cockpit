---
author: AI Cockpit 维护者
title: "WI-826——历史 evidence 归档兼容性"
description: "为旧 Runtime 捕获的 typed evidence 提供显式、fail-closed 的归档路径，不改写历史字节。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-826-historical-lifecycle-reconciliation
lastVerifiedBy: WI-826-historical-lifecycle-reconciliation
terminalArchive: .ai/work-items/archive/WI-826-historical-lifecycle-reconciliation.contract.json
terminalVerification: .ai/evidence/WI-826-historical-lifecycle-reconciliation.verification.json
terminalDecision: .ai/decisions/WI-826-historical-lifecycle-reconciliation.close.json
---

[English](WI-826-historical-lifecycle-reconciliation.md) · [日本語](WI-826-historical-lifecycle-reconciliation.ja.md)

# WI-826——历史 evidence 归档兼容性

## 意图与边界

本 Work Item 为旧 Runtime 捕获的 typed verification evidence 增加显式的
历史归档路径。它保留原始 evidence 字节，并将归档 manifest 绑定到原始
digest、Work Item、仓库、Contract、snapshot、Runtime 和 receipt 身份；普通
当前 Runtime 归档行为保持不变。

## Fail-closed 行为

历史归档会拒绝当前 Runtime evidence、格式错误或 legacy untyped evidence、
digest 篡改及身份不匹配，且不会写入部分归档。归档验证器会重新检查绑定的
evidence 文件和全部记录身份；兼容路径必须显式调用，不构成通用历史豁免。

## 验收 evidence

- archive：`.ai/work-items/archive/WI-826-historical-lifecycle-reconciliation.contract.json`
- 正式 verification：`.ai/evidence/WI-826-historical-lifecycle-reconciliation.verification.json`
- close decision：`.ai/decisions/WI-826-historical-lifecycle-reconciliation.close.json`
