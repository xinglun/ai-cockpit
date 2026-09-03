---
author: AI Cockpit maintainers
title: "WI-529 — WI-528 terminal documentation promotion"
description: "WI-528 の verified close 後に文書投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-529-wi528-doc-promotion
lastVerifiedBy: WI-529-wi528-doc-promotion
---

[English](WI-529-wi528-doc-promotion.md) · [简体中文](WI-529-wi528-doc-promotion.zh-CN.md)

## Goal

WI-528 の三言語ページと reference-parity 投影を、immutable な archive、
verification、finalization、close evidence に同期します。

## Scope

- WI-528 に公式の closed Work Item promotion helper を実行します。
- Runtime 記録、source behavior、adopter repository は変更しません。

## Acceptance

- WI-528 ページと parity 行が正確な terminal evidence を参照します。
- documentation と governance integrity のチェックが成功します。

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-528-doc-promotion
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```
