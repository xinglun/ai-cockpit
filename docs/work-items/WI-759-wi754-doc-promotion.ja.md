---
author: AI Cockpit maintainers
title: "WI-759 — WI-754 terminal documentation promotion"
description: "Merged WI-754 recovery successor を三言語 Work Item documentation boundary に投影します。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorization
workItemId: WI-759-wi754-doc-promotion
lastVerifiedBy: WI-760-wi759-doc-repair
---

[English](WI-759-wi754-doc-promotion.md) · [简体中文](WI-759-wi754-doc-promotion.zh-CN.md)

# WI-759 — WI-754 terminal documentation promotion

## Intent

WI-759 は PR #742 で merged WI-754 recovery successor の三言語 documentation
boundary を promotion しました。immutable archive、verification、
resource-finalization chain が governance の事実源です。

## Boundary

これは documentation と governance projection だけを扱います。Runtime behavior、
product code、authorization semantics、exit codes、WI-759 の immutable archive と
evidence bytes は変更しません。不足していた self-projection pages は WI-759 を close
する前に successor WI-760 が補完します。

## Evidence boundary

- Archive: `.ai/work-items/archive/WI-759-wi754-doc-promotion.archive.json`
- Contract: `.ai/work-items/archive/WI-759-wi754-doc-promotion.contract.json`
- Verification: `.ai/evidence/WI-759-wi754-doc-promotion.verification.json`
- Recovery binding: `.ai/decisions/WI-759-wi754-doc-promotion.recovery.json`
- Finalization head: `.ai/decisions/WI-759-wi754-doc-promotion.finalize.95f5f4d266ae632f8203327fea649e30eee82088d4a9242e89e7a2a0f7bcc85d.json`
- Reviewed delivery: [PR #742](https://github.com/xinglun/ai-cockpit/pull/742)

## Closure condition

WI-759 は WI-760 が pages を補完し、通常の Runtime close と
`promote_closed_work_item.py --check-all` が通るまで pending のままです。
