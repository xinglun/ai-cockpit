---
author: AI Cockpit maintainers
title: "WI-436 — close 済み文書投影の昇格"
workItemId: WI-436-reference-doc-promotion
description: "WI-435 close 後の三言語文書投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-436-reference-doc-promotion
terminalArchive: .ai/work-items/archive/WI-436-reference-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-436-reference-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-436-reference-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-436-reference-doc-promotion.close.json
---

# WI-436 — close 済み文書投影の昇格

この文書専用 Work Item は、repository の closed Work Item promotion helper
を使って WI-435 を昇格します。semantic reference は maintainer が管理する
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` に固定し、公開
reference repository には接続せず、Runtime の挙動も変更しません。

[English](WI-436-reference-doc-promotion.md) · [简体中文](WI-436-reference-doc-promotion.zh-CN.md)

## Scope

- WI-435 の三言語 Work Item 文書と三言語 reference-parity 行を昇格する。
- immutable な archive、verification、finalization、close の path だけを記録する。
- 他の Work Item と過去の bytes は変更しない。

## Verification

`tests/docs/promote_closed_work_item.py --work-item WI-435-reference-inventory-rebaseline-local`、
`--check-all`、documentation acceptance、parity status、diff checks をすべて通過させます。
