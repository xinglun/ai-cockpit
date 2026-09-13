---
author: AI Cockpit 维护者
title: "WI-818——evidence Contract 修复"
description: "让必需 evidence class 在验证前尽早失败，同时保持历史 Contract 兼容。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-818-evidence-contract-repair
lastVerifiedBy: WI-818-evidence-contract-repair
terminalArchive: .ai/work-items/archive/WI-818-evidence-contract-repair.contract.json
terminalVerification: .ai/evidence/WI-818-evidence-contract-repair.verification.json
terminalDecision: .ai/decisions/WI-818-evidence-contract-repair.close.json
---

[English](WI-818-evidence-contract-repair.md) · [日本語](WI-818-evidence-contract-repair.ja.md)

# WI-818——evidence Contract 修复

## 意图与边界

本 Work Item 让必需 evidence class 在验证前明确并尽早检查，让诊断保留全部声明的
class，同时保持历史 Contract 可读取。Runtime 生成的 archive、verification evidence
和 close decision 才是权威；本文不重写或重新创建它们。

## 验证

记录的 verification receipt 已通过 evidence-class 与历史兼容性检查。后续 Work Item
必须在启动昂贵验证前使用受支持的 evidence vocabulary。
