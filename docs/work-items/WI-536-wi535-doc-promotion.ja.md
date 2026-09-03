---
author: AI Cockpit maintainers
title: "WI-536 — WI-535 terminal documentation promotion"
description: "WI-535 の読者向け文書を昇格し、本 Work Item 自身を archive 前に登録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-536-wi535-doc-promotion
lastVerifiedBy: WI-536-wi535-doc-promotion
---

[English](WI-536-wi535-doc-promotion.md) · [简体中文](WI-536-wi535-doc-promotion.zh-CN.md)

## Goal

WI-535 の三言語ページを不変の terminal evidence と同期し、本 Work Item
自身も verification と archive の前にすべての parity 台帳へ登録します。

## Scope and boundary

- WI-535 と WI-536 の三言語 reader page。
- 英語・日本語・簡体字中国語の parity ledger。
- Runtime 挙動、生成 `.ai` 記録、release artifact、対象 repository は対象外です。

## Acceptance

- WI-535 のページと parity 行が正確な terminal evidence を束縛する。
- WI-536 が verification と archive の前に三つの parity 台帳へ登録される。
- 文書、parity、governance integrity のチェックが成功する。

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-535-mcp-fixture-cleanup
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
tests/ci/governance_integrity_gate.py --repo <repo>
```
