---
author: AI Cockpit maintainers
title: "WI-540 — WI-539 terminal documentation promotion"
description: "不変の close evidence に基づき、WI-539 の比較ドキュメントと parity 行を昇格します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-540-wi539-doc-promotion
lastVerifiedBy: WI-540-wi539-doc-promotion
---

[English](WI-540-wi539-doc-promotion.md) · [简体中文](WI-540-wi539-doc-promotion.zh-CN.md)

## Goal

WI-539 の三言語 reader page と reference parity 行を、不変の close evidence と
同期します。これは読者向けの限定された投影であり、Runtime の evidence や
governance fact を変更しません。

## Scope and boundary

- WI-539 の三言語 reader page と三つの reference parity ledger。
- Runtime behavior、生成された `.ai` evidence、release artifact、対象 repository
  はこの Work Item の範囲外です。

## Acceptance

- 三言語の WI-539 page が `implemented` と terminal evidence link を持つ。
- 三言語の parity 行が verified terminal lifecycle path を持ち、言語リンクを保つ。
- Documentation acceptance、parity integrity、closed Work Item promotion check が通る。

## Evidence boundary

Promotion は reader-facing projection のみを変更します。不変の Contract、
verification、finalization、close record は Runtime-owned evidence として残します。
