---
author: AI Cockpit 维护者
title: "WI-824——verification 前置条件与持久化 attempt"
description: "在启动任务前拒绝无效 verification 输入，并为可恢复执行保留有界 attempt。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-824-verification-preconditions
lastVerifiedBy: WI-824-verification-preconditions
terminalArchive: .ai/work-items/archive/WI-824-verification-preconditions.contract.json
terminalVerification: .ai/evidence/WI-824-verification-preconditions.verification.json
terminalDecision: .ai/decisions/WI-824-verification-preconditions.close.json
---

[English](WI-824-verification-preconditions.md) · [日本語](WI-824-verification-preconditions.ja.md)

# WI-824——verification 前置条件与持久化 attempt

## 意图与边界

本 Work Item 在启动验证任务前检查当前 preflight、Contract 和仓库身份；条件不可用时提前失败。同时，独立保存每个已执行节点的有界输出、退出码、超时状态、耗时、命令身份、Runtime 身份和源码快照，不把正式 completion receipt 当作唯一结果。历史 evidence 不被改写。

## 恢复行为

只有源码快照、命令和依赖输入、Runtime digest、仓库身份以及 Contract execution scope 全部匹配时，成功 attempt 才可以复用。因此仅治理投影修正可以避免重跑未变化的命令；源码、命令、Runtime 或相关依赖变化会使复用失效。前置条件拒绝会记录启动的项目进程数为 0，并输出结构化诊断。

## 验收 evidence

- archive：`.ai/work-items/archive/WI-824-verification-preconditions.contract.json`
- 正式 verification：`.ai/evidence/WI-824-verification-preconditions.verification.json`
- attempt 记录：`.ai/evidence/WI-824-verification-preconditions.verification-attempt.*.json`
- close decision：`.ai/decisions/WI-824-verification-preconditions.close.json`
