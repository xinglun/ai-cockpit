---
author: AI Cockpit maintainers
title: "WI-852 — active Work Item の retirement"
description: "元の bytes を書き換えず、検証済みと主張せずに、統合済みの active Work Item を退役させる。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-852-active-work-item-retirement
lastVerifiedBy: WI-852-active-work-item-retirement
terminalArchive: .ai/work-items/archive/WI-852-active-work-item-retirement.contract.json
terminalVerification: .ai/evidence/WI-852-active-work-item-retirement.verification.json
terminalDecision: .ai/decisions/WI-852-active-work-item-retirement.close.json
---

[English](WI-852-active-work-item-retirement.md) · [简体中文](WI-852-active-work-item-retirement.zh-CN.md)

# WI-852 — active Work Item の retirement

この Work Item は、同期済み base に成果が既に存在する場合の
`integrated`、または明示的にリンクされた successor がある場合の
`replaced` を Runtime の明示的な retirement として扱います。元の
Contract、Summary、Outcome、event、attempt bytes は immutable archive に
保存しますが、検証済み・完了・履歴書き換えを意味しません。

Runtime は archive を書く前に stale、foreign、malformed、duplicate、未接続の
入力を拒否します。receipt は repository、Work Item、Contract、Summary、snapshot、
Runtime、保持した各 artifact の digest に bind されます。
