---
author: AI Cockpit maintainers
title: "WI-492 — WI-491 terminal documentation promotion"
description: "v0.2.58 公開前に、close 済み WI-491 の release evidence を reader-facing documentation へ昇格します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-492-wi491-doc-promotion
status: implemented
authority: human-authorized
lastVerifiedBy: WI-492-wi491-doc-promotion
terminalArchive: .ai/work-items/archive/WI-492-wi491-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-492-wi491-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-492-wi491-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-492-wi491-doc-promotion.close.json
---

# WI-492 — WI-491 terminal documentation promotion

この bounded documentation Work Item は、close 済み WI-491 の release evidence を
3 言語のページと parity ledger に昇格します。immutable な governance record を保持し、
Runtime behavior は変更しません。

[English](WI-492-wi491-doc-promotion.md) · [简体中文](WI-492-wi491-doc-promotion.zh-CN.md)

## Scope

- WI-491 の 3 Work Item pages と 3 parity rows を昇格します。
- 各 projection を WI-491 の archive、verification、finalization、close evidence に bind します。
- この Work Item 自身の pages と parity row も同じ bounded lifecycle に含めます。

## Acceptance

- 6 つの WI-491 projection は immutable record を書き換えず evidence-backed になります。
- closed Work Item promotion check と status-consistency check が成功します。
- Runtime source、reference inventory classification、global Agent/MCP configuration は変更しません。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `git diff --check`
