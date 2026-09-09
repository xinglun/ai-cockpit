---
author: AI Cockpit maintainers
title: "WI-718——WI-717 Work Item 文档投影"
description: "补齐已关闭 WI-717 缺失的三语文档投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human:repository-owner
workItemId: WI-718-wi717-doc-projection
lastVerifiedBy: WI-718-wi717-doc-projection
terminalArchive: .ai/work-items/archive/WI-718-wi717-doc-projection.contract.json
terminalVerification: .ai/evidence/WI-718-wi717-doc-projection.verification.json
terminalFinalization: .ai/decisions/WI-718-wi717-doc-projection.finalize.json
terminalDecision: .ai/decisions/WI-718-wi717-doc-projection.close.json
---

[English](WI-718-wi717-doc-projection.md) · [日本語](WI-718-wi717-doc-projection.ja.md)

# WI-718——WI-717 Work Item 文档投影

## 意图

补齐已关闭 WI-717 投影缺失的三语 Work Item 页面，并在不改写的前提下持久化
WI-717 Runtime 生成的 finalization 与 close receipts。

## 边界

这是窄范围文档和治理记录投影。不改变 Runtime 行为、Outcome 语义、机器 JSON、
退出码、授权语义或历史 archive/evidence bytes。

## 验收

- 三语 WI-717 页面展示不可变终态 evidence 与 approved close。
- WI-717 Runtime 生成的 finalization 与 close receipts 保持逐字节不变。
- closed Work Item promotion、文档验收和 governance integrity 检查通过。
