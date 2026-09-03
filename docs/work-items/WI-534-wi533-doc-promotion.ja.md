---
author: AI Cockpit maintainers
title: "WI-534 — WI-533 terminal documentation promotion"
description: "WI-533 の verified close 後に読者文書を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-534-wi533-doc-promotion
lastVerifiedBy: WI-534-wi533-doc-promotion
---

[English](WI-534-wi533-doc-promotion.md) · [简体中文](WI-534-wi533-doc-promotion.zh-CN.md)

## Goal

WI-533 の三言語ページと parity 行を、immutable な archive、verification、
finalization、close evidence に同期します。

## Scope

- WI-533 に公式の closed Work Item promotion helper を実行します。
- Runtime 記録、source behavior、release artifact、adopter repository は変更しません。

## Acceptance

- WI-533 ページと parity 行が正確な terminal evidence を参照します。
- documentation と governance integrity のチェックが成功します。

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-533-release-v0-2-66
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
```
