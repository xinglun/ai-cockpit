---
author: AI Cockpit maintainers
title: WI-406——已关闭文档晋级对齐
description: 使已关闭 Work Item 文档晋级与 Runtime finalReport evidence 绑定保持一致。
workItemId: WI-406-close-promotion-alignment
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-406-close-promotion-alignment
terminalArchive: .ai/work-items/archive/WI-406-close-promotion-alignment.contract.json
terminalVerification: .ai/evidence/WI-406-close-promotion-alignment.verification.json
terminalFinalization: .ai/decisions/WI-406-close-promotion-alignment.finalize.json
terminalDecision: .ai/decisions/WI-406-close-promotion-alignment.close.json
---

# WI-406——已关闭文档晋级对齐

## 意图

使已关闭 Work Item 文档晋级器接受 Runtime 有效的 `finalReport` evidence
绑定，同时对格式错误或不完整的 close 记录保持 fail-closed。

## 范围

- 接受由 `finalReport.bindings` 绑定的 verification 引用。
- 保持结构化人工决定引用非空且可审计。
- 为本 Work Item 保留三语文档与 parity 登记。

## 证据

- 归档 Contract：`.ai/work-items/archive/WI-406-close-promotion-alignment.contract.json`
- Verification：`.ai/evidence/WI-406-close-promotion-alignment.verification.json`
- Pull Request：[ #371 ](https://github.com/xinglun/ai-cockpit/pull/371)

## 边界

本 Work Item 不改写历史 Runtime evidence，也不改变 Runtime lifecycle 语义。
只有在 reviewed merge 与 close evidence 可用后，才晋级终态文档。
