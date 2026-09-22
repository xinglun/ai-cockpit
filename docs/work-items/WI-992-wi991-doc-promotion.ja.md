---
author: AI Cockpit maintainers
title: "WI-992 — WI-991 terminal documentation projection"
description: "WI-991 の attempt が replacement された後、修正済みの terminal documentation projection を完了する。"
audience: [maintainer, reviewer, contributor]
status: implemented
authority: authorized
workItemId: WI-992-wi991-doc-promotion
lastVerifiedBy: WI-992-wi991-doc-promotion
terminalArchive: .ai/work-items/archive/WI-992-wi991-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-992-wi991-doc-promotion.verification.json
terminalDecision: .ai/decisions/WI-992-wi991-doc-promotion.close.json
---

[English](WI-992-wi991-doc-promotion.md) · [简体中文](WI-992-wi991-doc-promotion.zh-CN.md)

# WI-992 — WI-991 terminal documentation projection

WI-992 は WI-991 に明示的に bind された successor である。修正された verification
command set で WI-990 terminal projection を完了し、WI-991 の failed attempt と
retirement facts を保持する。

## Boundary

対象は WI-990、WI-991、WI-992 の documentation projection、三つの reference-parity
ledger、Runtime が生成する `.ai/` evidence に限定する。source behavior、release
behavior、既存 lifecycle fact は対象外である。

## Acceptance

- 六つの WI-990 terminal projection が immutable archive、verification、close evidence と一致する。
- WI-991 の failed attempt、recovery、retirement が明示され、verification は主張しない。
- 三つの WI-992 page と parity row が close まで自身の bounded self-projection を保持する。
- WI-990 単体 projection、repository-wide `--check-all`、parity、status consistency が通る。
