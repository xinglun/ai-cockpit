---
author: AI Cockpit maintainers
title: "WI-532 — WI-531 terminal documentation promotion"
description: "WI-531 の verified close 後に読者文書を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-532-wi531-doc-promotion
lastVerifiedBy: WI-532-wi531-doc-promotion
---

[English](WI-532-wi531-doc-promotion.md) · [简体中文](WI-532-wi531-doc-promotion.zh-CN.md)

## Goal

WI-531 の三言語ページと parity 行を、immutable な archive、verification、
finalization、close evidence に同期します。

## Scope

- WI-531 に公式の closed Work Item promotion helper を実行します。
- Runtime 記録、source behavior、adopter repository は変更しません。

## Acceptance

- WI-531 ページと parity 行が正確な terminal evidence を参照します。
- documentation と governance integrity のチェックが成功します。

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-531-historical-direct-merge-apply
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```
