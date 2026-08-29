---
author: AI Cockpit maintainers
title: "WI-381 — reference parity supersede decision link 修正"
description: "history evidence を変更せず、versioned supersede decision を全 parity projection に bind します。"
workItemId: WI-381-reference-parity-decision-link-fix
canonical: docs/work-items/WI-381-reference-parity-decision-link-fix.md
audience: [maintainer, reviewer]
status: implemented
authority: translation
lastVerifiedBy: WI-381-reference-parity-decision-link-fix
terminalArchive: .ai/work-items/archive/WI-381-reference-parity-decision-link-fix.contract.json
terminalVerification: .ai/evidence/WI-381-reference-parity-decision-link-fix.verification.json
terminalFinalization: .ai/decisions/WI-381-reference-parity-decision-link-fix.finalize.json
terminalDecision: .ai/decisions/WI-381-reference-parity-decision-link-fix.close.json
capabilityClaims: [governance_integrity, reference_parity]
---

# WI-381 — reference parity supersede decision link 修正

[English](WI-381-reference-parity-decision-link-fix.md) · [简体中文](WI-381-reference-parity-decision-link-fix.zh-CN.md)

## Intent と boundary

close 済み WI-379 predecessor には canonical successor decision と digest-versioned supersede decision が存在します。三言語 parity projection は exact な terminal decision path を示す必要があります。本 Work Item は documentation link のみを変更し、archive、evidence、decision bytes は Runtime-owned の immutable records として保持します。

## Scope

- 全 parity ledger に exact な WI-379 versioned supersede decision path を追加します。
- WI-379 の archive、evidence、recovery、close record は変更しません。
- English、簡体中文、日本語の projection を同じ意味に保ちます。

## Out of scope

Runtime code、generated `.ai` records、release artifacts、global Agent/MCP configuration。

## Acceptance

- 各 parity row が WI-379 archive、verification、canonical recovery、versioned supersede recovery、superseded close path を参照すること。
- governance integrity gate が WI-379 `missing_parity_decision` を報告しないこと。
- historical archive/evidence digest が変更されないこと。

## Verification と terminal records

明示的な `--repo` を使う installed Runtime、governance/documentation checks、`cargo test --locked --workspace` を実行します。reviewed merge 後、header に宣言した archive、verification、finalization、close path を記録します。
