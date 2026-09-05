---
author: AI Cockpit maintainers
title: "WI-581 — WI-580 终态文档推广"
description: "从不可变治理证据推广已关闭 WI-580 的文档投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-581-doc-promotion-wi580
lastVerifiedBy: WI-581-doc-promotion-wi580
terminalArchive: .ai/work-items/archive/WI-581-doc-promotion-wi580.contract.json
terminalVerification: .ai/evidence/WI-581-doc-promotion-wi580.verification.json
terminalFinalization: .ai/decisions/WI-581-doc-promotion-wi580.finalize.2c045b801ad0a39909547eeed34d24da608fa121228b58ffa932079a6461b235.json
terminalDecision: .ai/decisions/WI-581-doc-promotion-wi580.close.json
---

[English](WI-581-doc-promotion-wi580.md) · [日本語](WI-581-doc-promotion-wi580.ja.md)

# WI-581 — WI-580 终态文档推广

## 目标

从已关闭 WI-580 的不可变归档、验证证据、资源终结链和关闭决定中，
推广三语人类可读文档。本文档是投影，不替代这些记录。

## 证据边界

Runtime 生成的记录保持权威。推广只同步三种语言的文档，不改变 Contract、
验证结果、资源终结历史或人工决定。

## 终态证据

终态链接将在文档推广 Work Item 验证关闭证据后，由
`tests/docs/promote_closed_work_item.py` 确定性写入。
