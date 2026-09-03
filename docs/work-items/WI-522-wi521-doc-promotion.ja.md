---
author: AI Cockpit maintainers
title: "WI-522 — WI-521 terminal documentation promotion"
description: "WI-521 の verified close 後に文書投影を昇格し、immutable Runtime 記録を書き換えない。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human-authorized
workItemId: WI-522-wi521-doc-promotion
lastVerifiedBy: WI-522-wi521-doc-promotion
---

[English](WI-522-wi521-doc-promotion.md) · [简体中文](WI-522-wi521-doc-promotion.zh-CN.md)

## Goal

WI-521 の読者向けページと parity 行を、Runtime が既に記録した terminal truth
へ昇格する。

WI-522 は immutable predecessor として保持されます。archive によって branch
 HEAD が進み、pre-merge finalization が stale になったため、Runtime の recovery
 decision を `.ai/decisions/WI-522-wi521-doc-promotion.recovery.json` に記録しました。
同じ文書投影は最新の reviewed base から WI-523 が再配信します。前駆証拠は書き換えず、
新しい成功として扱いません。

## Scope

- WI-521 の三言語 Work Item ページ。
- 三つの `docs/reference/reference-parity` 投影。
- WI-522 自身の三言語記録。

Runtime source、reference implementation、object repository、release 公開、
global Agent/MCP 設定、生成済み WI-521 記録は対象外です。

## Acceptance

- `promote_closed_work_item.py --check-all` が WI-521 の stale projection を報告しない。
- WI-521 のページと parity 行が Implemented となり、archive、verification、
  finalization、close の terminal evidence を正確に参照する。
- 文書、parity、status consistency、governance integrity の各チェックが通り、
  immutable Runtime 記録が変わらない。

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
