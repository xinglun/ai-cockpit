---
author: AI Cockpit maintainers
title: "WI-852 — active WI 退休"
description: "在不重写原始字节或声称已验证的前提下，退休已入库的 active Work Item。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-852-active-work-item-retirement
lastVerifiedBy: WI-852-active-work-item-retirement
terminalArchive: .ai/work-items/archive/WI-852-active-work-item-retirement.contract.json
terminalVerification: .ai/evidence/WI-852-active-work-item-retirement.verification.json
terminalDecision: .ai/decisions/WI-852-active-work-item-retirement.close.json
---

[English](WI-852-active-work-item-retirement.md) · [日本語](WI-852-active-work-item-retirement.ja.md)

# WI-852 — active Work Item 退休

本 Work Item 为 Runtime 增加显式退休路径：交付已经存在于同步后的基线时
可以标记为 integrated；未完成范围则必须绑定明确的 successor。退休会把
原始 Contract、Summary、Outcome、事件和尝试记录保存在不可变 archive 中，
但不代表 verification、完成或允许改写历史。

Runtime 会在写入 archive 前拒绝过期、外部、格式错误、重复或未绑定的输入。
退休回执绑定仓库、Work Item、Contract、Summary、快照、Runtime 以及每个保留的
产物摘要。
