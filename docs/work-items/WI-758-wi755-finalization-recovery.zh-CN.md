---
author: AI Cockpit maintainers
title: "WI-758——WI-755 finalization recovery"
description: "在不改写不可变历史的前提下修复 WI-755 的合并后治理交接。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-758-wi755-finalization-recovery
status: implemented
authority: human:xinglun
lastVerifiedBy: WI-758-wi755-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-758-wi755-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-758-wi755-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-758-wi755-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-758-wi755-finalization-recovery.close.json
---

[English](WI-758-wi755-finalization-recovery.md) · [日本語](WI-758-wi755-finalization-recovery.ja.md)

# WI-758——WI-755 finalization recovery

## 意图

本 successor 修复 WI-755 的合并后治理交接，并将 WI-755 的归档 Contract、验证证据、
recovery decision 和 finalization receipt 作为不可变历史事实保留。

## 已观察到的边界

PR #735 已完成评审，hosted checks 全部通过，并以
`8dcac7ecdb6878c6e506d86921e4d82b7e16e68b` 合并。该 PR 的已审阅 head 是
`06f7d03f6877405a6885cf1412108e68d9c2893a`。WI-755 现有 canonical finalization receipt
记录的是较早的中间 head
`251c3867c75d1395b4aa203057174fe1c9b1cb52`。

Runtime 正确拒绝未绑定的 finalization head 变化。WI-758 记录这一差异并提供新的 successor
边界；它不重新解释旧 receipt、不改写 predecessor archive，也不把治理修复宣称为实现批准或安全保证。

## 边界与证据

- recovery decision：`.ai/decisions/WI-755-p1-observation-context-successor.recovery.ccd6ce5cf8f1c2a563578437cc313079462d08a88b04fb8b61b734f54bf237a2.json`
- predecessor Contract：`.ai/work-items/archive/WI-755-p1-observation-context-successor.contract.json`
- predecessor 验证：`.ai/evidence/WI-755-p1-observation-context-successor.verification.json`
- predecessor finalization 事实：`.ai/decisions/WI-755-p1-observation-context-successor.finalize.json`
- 已评审 PR：`https://github.com/xinglun/ai-cockpit/pull/735`

本 successor 只可新增自身的三语文档、parity 投影、验证证据和 Runtime 生成的生命周期记录。
源代码、协议 schema、Outcome 文案、授权语义以及 WI-755 的全部历史 bytes 均不在范围内。

## 验证与限制

验证将证明仓库身份、已评审合并 commit 中包含 WI-755 的实现、predecessor 的精确 digest 绑定，
以及没有改写 predecessor bytes。它不测量性能收益、不验证外部用户认知，也不授予发布批准。

## 终端证据

终端 archive、verification、finalization 和 close 路径将在 Runtime `finalize-verify` 与结构化
close 成功后，由 post-close 文档 promotion 写入。
