---
author: AI Cockpit maintainers
title: "WI-385 — reference inventory terminal projection"
workItemId: WI-385-reference-inventory-terminal-projection
description: "不変履歴を書き換えずに WI-384 の post-close terminal projection を完了する。"
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-385-reference-inventory-terminal-projection
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
