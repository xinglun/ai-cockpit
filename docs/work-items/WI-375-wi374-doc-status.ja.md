---
author: AI Cockpit maintainers
title: "WI-375 — WI-374 terminal documentation promotion"
description: "決定的な post-close promotion のため、三言語 Work Item と parity projection を準備する。"
workItemId: WI-375-wi374-doc-status
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-375-wi374-doc-status
capabilityClaims: [documentation_governance]
---

# WI-375 — WI-374 terminal documentation promotion

[English](WI-375-wi374-doc-status.md) · [简体中文](WI-375-wi374-doc-status.zh-CN.md)

## Intent

明示的な post-close promotion helper により、close 済み WI-374 の三言語文書と parity ledger を正しく保つ。本 Work Item は repository-local documentation boundary だけを準備・検証し、close 後に helper が machine-owned terminal projection を生成する。

## Scope と境界

- 三言語 WI-374 projection と三つの parity ledger を promotion helper が要求する pre-close 形式に保つ。
- documentation、parity、governance integrity gate を検証する。
- WI-374 の immutable Runtime evidence を保持し、`close → promote closed docs → terminal CI` の順序に従う。

Runtime、release asset、historical evidence bytes、global Agent/MCP 設定は本 Work Item の範囲外である。

## Acceptance

1. WI-374 の三言語文書と parity 行が promotion 前の正しい projection で、immutable terminal receipt を参照する。
2. close 前の documentation、parity、governance integrity check が通る。
3. reviewed merge と close 後、promotion helper が terminal frontmatter と parity 行だけを決定的に書ける。
4. Work Item が reviewed merge、finalization、close、正確な cleanup を完了する。

## Verification boundary

Runtime は本 Work Item の Contract、checkpoint、verification、archive、finalization、close evidence を記録する。`promote_closed_work_item.py` は明示的な post-close documentation projection であり、Runtime truth や historical evidence を書き換えない。
