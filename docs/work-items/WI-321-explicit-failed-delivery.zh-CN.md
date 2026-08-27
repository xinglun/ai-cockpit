---
author: AI Cockpit maintainers
title: "WI-321——显式失败交付恢复边界"
workItemId: WI-321-explicit-failed-delivery
description: "在不改写不可变失败交付历史的前提下，记录由 Runtime 绑定的 successor。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-321-explicit-failed-delivery
terminalArchive: .ai/work-items/archive/WI-321-explicit-failed-delivery.contract.json
terminalVerification: .ai/evidence/WI-321-explicit-failed-delivery.verification.json
terminalFinalization: .ai/decisions/WI-321-explicit-failed-delivery.finalize.json
terminalDecision: .ai/decisions/WI-321-explicit-failed-delivery.close.json
---

# WI-321——显式失败交付恢复边界

## 意图与边界

WI-313 是不可变的失败交付：PR #277 未合并，其 retry receipt 没有终态决定或
successor。本 Work Item 记录由 Runtime 生成的 successor receipt，使治理门不会留下
孤立历史，也不会把它静默投影成已完成实现。

前置 Work Item 仍是历史真相。本 Work Item 不改写或删除其 Contract、Summary、Outcome、
Events、archive、verification、retry receipt、branch 或 PR 记录，也不声称 WI-313 的
实现已经合并。

## 范围与验收

- Runtime 生成的 WI-313 successor receipt 绑定本 Work Item、repository identity、前置
  digest、Runtime identity 和明确的人类授权。
- governance integrity gate 具有确定性的回归：没有 successor 的孤立 retry 不能成为
  终态成功；明确的 successor 则被接受并投影为 `已恢复`。
- 英语、简体中文和日语的 Work Item/parity 投影说明 PR 未合并的失败边界，并使用
  recovery receipt 作为 evidence。
- 既有历史字节和全局 Agent/MCP 配置保持不变。

## 验证

运行孤立 retry 与 recovery chain 静态回归、文档验收、locked workspace 测试和审阅分支
的 hosted CI。所有 repository-bound Runtime 命令都显式带 repository 路径；源码构建不作为
发布 evidence。

## 相关历史

- WI-313：PR #277 的不可变失败交付，现在由本 successor 明确恢复。
- WI-314 和 WI-315：独立的 recovery chain，保持不变。

[English](WI-321-explicit-failed-delivery.md) ·
[日本語](WI-321-explicit-failed-delivery.ja.md)
