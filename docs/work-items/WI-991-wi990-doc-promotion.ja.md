---
author: AI Cockpit maintainers
title: "WI-991 — WI-990 terminal documentation projection"
description: "WI-990 close 後に必要となった terminal documentation projection を完了する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-991-wi990-doc-promotion
lastVerifiedBy: WI-991-wi990-doc-promotion
---

[English](WI-991-wi990-doc-promotion.md) · [简体中文](WI-991-wi990-doc-promotion.zh-CN.md)

# WI-991 — WI-990 terminal documentation projection

WI-991 は、WI-990 の close 後チェックで必要と判定された六つの terminal projection
を生成する、範囲を限定した successor である。source behavior や immutable lifecycle
evidence は変更しない。

## Boundary

対象は三つの WI-990 language page、三つの reference-parity ledger、およびこの
Work Item 自身の三言語 self-projection に限定する。source code、test behavior、
release logic、既存 lifecycle record は対象外である。

## Acceptance

- 六つの WI-990 terminal projection が archive、verification、close evidence と一致する。
- 三つの WI-991 page と parity row が close まで自身の bounded self-projection を保持する。
- WI-990 単体 projection、repository-wide `--check-all`、parity、status consistency が通る。
