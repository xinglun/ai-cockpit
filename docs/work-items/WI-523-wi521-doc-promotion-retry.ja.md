---
author: AI Cockpit maintainers
title: "WI-523 — WI-521 documentation promotion retry"
description: "predecessor の pre-merge finalization が stale になった後、bounded な WI-521 文書投影を最新の reviewed base から再配信する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-523-wi521-doc-promotion-retry
lastVerifiedBy: WI-523-wi521-doc-promotion-retry
---

[English](WI-523-wi521-doc-promotion-retry.md) · [简体中文](WI-523-wi521-doc-promotion-retry.zh-CN.md)

## Goal

WI-522 の immutable archive と recovery record を保持したまま、最新の reviewed
default branch から WI-521 の terminal documentation projection を再配信します。

## Scope

- immutable predecessor WI-522 を recovered として記録し、recovery と successor をリンクする。
- WI-521 と WI-523 の reader-facing page および三言語 parity projection を昇格する。
- Runtime 生成 evidence、predecessor bytes、object repository、global configuration は変更しない。

## Acceptance

- WI-522 は明示的に recovered のままで、stale finalization を成功として表示しない。
- 三つの WI-523 ページと parity 行は terminal 化後の archive、verification、finalization、close evidence を正確に参照する。
- 文書、parity、status consistency、governance integrity の各 check が最終 archive commit で pass する。
- archive 後にだけ pre-merge finalization を作成し、その head が reviewed PR head と一致する。
- predecessor evidence と object repository file を変更しない。

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
