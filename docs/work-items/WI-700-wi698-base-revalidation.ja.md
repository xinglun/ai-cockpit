---
author: AI Cockpit maintainers
title: “WI-700 — WI-698 base revalidation”
description: “immutable な WI-698 delivery を保持し、当時の default base に再検証を bind します。”
audience: [contributor, maintainer, reviewer]
workItemId: WI-700-wi698-base-revalidation
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-700-wi698-base-revalidation
---

[English](WI-700-wi698-base-revalidation.md) · [简体中文](WI-700-wi698-base-revalidation.zh-CN.md)

# WI-700 — WI-698 base revalidation

WI-700 は WI-698 の immutable な recovery predecessor です。Contract と verification
が古い default branch に bind されたため、WI-701 が current default base から同じ
bounded delivery を再検証します。WI-698 と WI-700 の archive/evidence bytes は書き換えません。

## Recovery boundary

- Archive: `.ai/work-items/archive/WI-700-wi698-base-revalidation.contract.json`
- Historical verification: `.ai/evidence/WI-700-wi698-base-revalidation.verification.json`
- Recovery decision: `.ai/decisions/WI-700-wi698-base-revalidation.recovery.json`
- Successor: WI-701-wi700-current-base-revalidation

この recovery projection は新しい implementation semantics、governance rule、protocol
format、または他 agent の branch 変更を導入しません。
