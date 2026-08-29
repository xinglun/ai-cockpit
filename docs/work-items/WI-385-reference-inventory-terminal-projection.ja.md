---
author: AI Cockpit maintainers
title: "WI-385 — reference inventory terminal projection"
workItemId: WI-385-reference-inventory-terminal-projection
description: "不変履歴を書き換えずに WI-384 の post-close terminal projection を完了する。"
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-385-reference-inventory-terminal-projection
terminalArchive: .ai/work-items/archive/WI-385-reference-inventory-terminal-projection.contract.json
terminalVerification: .ai/evidence/WI-385-reference-inventory-terminal-projection.verification.json
terminalFinalization: .ai/decisions/WI-385-reference-inventory-terminal-projection.finalize.5000ae21b509964497aa74cb0abb6463b1c0737042b05ae6d130044eed153358.json
terminalDecision: .ai/decisions/WI-385-reference-inventory-terminal-projection.close.json
---

# WI-385 — reference inventory terminal projection

WI-385 は WI-384 close 後に見つかった文書整合性 defect の明示的な successor です。
三言語 parity 行と WI-384 の三言語 status metadata だけを変更し、WI-384 の archive、
evidence、finalization、close、recovery records は不変のまま保持します。

## Acceptance

- parity ledger が WI-384 を `Implemented` とし、terminal records をリンクする。
- WI-384 文書が `implemented` status で archive、verification、finalization、close を bind する。
- Runtime や predecessor bytes を変更せず、documentation と governance integrity gate が通る。

[English](WI-385-reference-inventory-terminal-projection.md) · [简体中文](WI-385-reference-inventory-terminal-projection.zh-CN.md)
