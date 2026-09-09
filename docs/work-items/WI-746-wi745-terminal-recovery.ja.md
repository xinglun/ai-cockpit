---
author: AI Cockpit maintainers
title: "WI-746 — WI-745 terminal recovery"
description: "immutable evidence を書き換えず、merge 済み WI-745 の documentation successor を調整します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-746-wi745-terminal-recovery
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-746-wi745-terminal-recovery
predecessorWorkItem: WI-745-wi743-doc-promotion
recoveryDecision: .ai/decisions/WI-745-wi743-doc-promotion.recovery.json
---

# WI-746 — WI-745 terminal recovery

WI-746 は merge 済み WI-745 documentation promotion の current-base recovery successor です。
WI-745 と WI-743 の historical bytes を保持し、三言語 projection を修正したうえで、Runtime
finalization と close の前に fresh verification を記録します。production または performance
behavior は変更しません。

[English](WI-746-wi745-terminal-recovery.md) · [简体中文](WI-746-wi745-terminal-recovery.zh-CN.md)

## Recovery boundary

Runtime recovery receipt は WI-745、その Contract/Summary digest、PR #718、そしてこの successor
を bind します。WI-743 は immutable な closed evidence で表現され、current-state prose もその終端事実に一致します。

## Evidence and lifecycle

- fresh verification は `.ai/evidence/WI-746-wi745-terminal-recovery.verification.json` に記録します。
- archive、finalization、Outcome、close は Runtime が生成します。
- この recovery は user-visible な performance benefit を主張しません。
