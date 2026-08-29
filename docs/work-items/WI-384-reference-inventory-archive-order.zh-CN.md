---
author: AI Cockpit maintainers
title: "WI-384——参考 inventory 归档顺序"
workItemId: WI-384-reference-inventory-archive-order
description: "从 origin/main 重新交付参考 inventory parity 文档，并验证保留快照绑定证据的 finish/archive 顺序。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-384-reference-inventory-archive-order
terminalArchive: .ai/work-items/archive/WI-384-reference-inventory-archive-order.contract.json
terminalVerification: .ai/evidence/WI-384-reference-inventory-archive-order.verification.json
terminalFinalization: .ai/decisions/WI-384-reference-inventory-archive-order.finalize.33860f23c671c0707f6b0816ba55089af33c14b244b71855c31fb51af40ac81c.json
terminalDecision: .ai/decisions/WI-384-reference-inventory-archive-order.close.json
---

# WI-384——参考 inventory 归档顺序

## 意图与边界

WI-384 是不可变 WI-383 的显式恢复 successor。WI-383 在 `verify` 和 `archive`
之间提交生成 lifecycle records 后，Runtime 正确拒绝了失效证据；本 Work Item
保留 WI-382、WI-383 的全部 bytes，并从干净的 `origin/main` 基线重新交付同一
有界文档修正。

## 范围与验收

三语 comparison 页面必须与 5,119 条 inventory 标记一致。三语 parity ledger
必须在 verification 前登记 WI-382、WI-383 的已恢复行和 WI-384 的当前交付行。
WI-382、WI-383、WI-384 的 Work Item 页面必须保持身份和状态元数据一致。

顺序是验收边界的一部分：绑定已审阅 PR，执行 `verify`、`finish`、`archive`，
且仅在 archive 成功后提交生成 lifecycle records。不修改任何前驱 bytes、Runtime、
protocol、inventory 分类、CI/release 逻辑或全局 Agent/MCP 配置。

## 验证

使用已安装 Runtime 并为每条命令显式提供 repository 路径，同时运行 inventory、
文档状态和 governance-integrity 检查。最终 Outcome 必须对人可见；绿色 Outcome
不等于获准合并或发布。

[English](WI-384-reference-inventory-archive-order.md) ·
[日本語](WI-384-reference-inventory-archive-order.ja.md)
