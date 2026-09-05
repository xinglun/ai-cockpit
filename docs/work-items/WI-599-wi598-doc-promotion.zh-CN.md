---
author: AI Cockpit maintainers
title: "WI-599——WI-598 终态文档晋级"
description: "在预先登记三语 parity 证据后晋级已验证的 WI-598 文档投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-599-wi598-doc-promotion
lastVerifiedBy: WI-599-wi598-doc-promotion
terminalArchive: .ai/work-items/archive/WI-599-wi598-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-599-wi598-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-599-wi598-doc-promotion.finalize.eacd2ab5f9639f57f01f2caabefc3f22aaaf2e7842260629b1a8d8d538903a67.json
terminalDecision: .ai/decisions/WI-599-wi598-doc-promotion.close.json
---

[English](WI-599-wi598-doc-promotion.md) · [日本語](WI-599-wi598-doc-promotion.ja.md)

# WI-599——WI-598 终态文档晋级

## 目标

在 WI-598 的不可变 archive、verification、finalization 和 close receipt
有效后，晋级其三语 Work Item 与 reference-parity 投影。在生成新的验证
证据前登记本 WI 自身的投影，使治理完整性门能够审计完整生命周期。

## 边界

本 WI 只修改文档投影。Runtime 行为、对象工程、全局 Agent/MCP 配置、源
实现以及生成的 evidence 或 decision 字节均在边界之外。Contract 的接受
标准仍以其编写语言为权威。

## 验收

1. 三个 WI-598 页面包含从不可变 archive、verification、finalization 和
   close receipt 推导的终态路径。
2. 三个 reference-parity 行将 WI-598 报告为已实现，并包含匹配的终态
   evidence 路径。
3. 在生成任何验证证据前登记本 WI-599 记录及其三个 parity 行，且只在
   close 后晋级。
4. 不修改治理事实、源实现、对象工程或生成的 receipt 字节。

## 验证

使用显式 repository context 运行
`tests/docs/promote_closed_work_item.py --check`、
`tests/docs/documentation_acceptance.sh`、
`tests/docs/parity_status_check.sh`、参考源 inventory/metadata 回归以及
锁定的 workspace 检查。
