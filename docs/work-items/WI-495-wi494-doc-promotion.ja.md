---
author: AI Cockpit maintainers
title: "WI-495 — WI-494 terminal documentation promotion"
description: "クローズ済み WI-494 の比較証跡を昇格し、documentation gate loop を終了します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-495-wi494-doc-promotion
status: implemented
authority: human-authorized
lastVerifiedBy: WI-495-wi494-doc-promotion
terminalArchive: .ai/work-items/archive/WI-495-wi494-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-495-wi494-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-495-wi494-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-495-wi494-doc-promotion.close.json
---

# WI-495 — WI-494 terminal documentation promotion

この限定的な documentation Work Item は、WI-494 の terminal comparison evidence と
parity 登録を昇格します。post-close の documentation check が再帰しないよう、自身の
3 言語ページも同じ範囲に含めます。

[English](WI-495-wi494-doc-promotion.md) · [简体中文](WI-495-wi494-doc-promotion.zh-CN.md)

## Scope

- WI-494 の 3 ページと parity row を不可変の terminal receipt に結び付けます。
- documentation gate が要求する WI-495 の 3 ページと parity row を提供します。
- Runtime、reference inventory、グローバル Agent/MCP 設定は変更しません。

## Acceptance

- WI-494 の文書が archive、verification、finalization、close の証跡にリンクされます。
- documentation promotion、governance-integrity、status-consistency、parity check が成功します。
- 英語、簡体字中国語、日本語の投影が読みやすい状態になります。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
