---
author: AI Cockpit maintainers
title: "WI-513 — WI-512 終端ドキュメント昇格"
description: "不変な governance record を書き換えず WI-512 projection を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-513-wi512-doc-promotion
lastVerifiedBy: WI-513-wi512-doc-promotion
---

[English](WI-513-wi512-doc-promotion.md) · [简体中文](WI-513-wi512-doc-promotion.zh-CN.md)

## Goal

WI-512 の close evidence が存在した後、parity projection を pre-archive
registration から terminal `Implemented` row に昇格する。helper は決定的に
動作し、WI-512 の Contract、Summary、Outcome、Events、verification、
finalization、close bytes を書き換えない。

## Scope

- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- この WI の三言語 reader record。

## Acceptance

- `promote_closed_work_item.py --check-all` が WI-512 の stale projection を報告しない。
- documentation、parity、governance-integrity check が pass する。
- WI-512 の immutable generated record が byte-identical のまま保たれる。
- Runtime、reference source、object repository、global Agent/MCP 設定を変更しない。

## Boundary

これは close 後の documentation projection だけを扱う。governance fact や
approval を変更せず、reference implementation もコピーしない。
