---
author: AI Cockpit maintainers
title: "WI-988 — WI-986 documentation promotion successor"
description: "WI-987 の不変な verification target failure 後に、範囲を限定した WI-986 documentation projection を完了する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-988-wi986-doc-promotion
lastVerifiedBy: WI-988-wi986-doc-promotion
---

[English](WI-988-wi986-doc-promotion.md) · [简体中文](WI-988-wi986-doc-promotion.zh-CN.md)

# WI-988 — WI-986 documentation promotion successor

WI-988 は WI-987 に明示的に bind された successor Work Item である。WI-987 の
不変の failed-attempt と recovery evidence を保持したまま、実際に close 済みの
`WI-986-wi985-doc-promotion` に対して修正済みの verification を実行する。

## Boundary

対象は WI-986 の terminal documentation、WI-987 recovered predecessor の projection、
WI-988 のページ、三つの reference-parity ledger に限定する。source code、release
behavior、既存 lifecycle receipt は対象外である。

## Acceptance

- WI-986 の三言語ページが immutable archive、verification、close の事実を参照する。
- WI-987 のページと parity row が failed attempt を recovered として保持し、WI-988 を successor として bind する。
- WI-986 の単体 projection と repository-wide `--check-all` が通る。
