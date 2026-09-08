---
author: AI Cockpit maintainers
title: “WI-701 — WI-700 current-base revalidation”
description: “current remote default base から observation-context delivery を再検証します。”
audience: [contributor, maintainer, reviewer]
workItemId: WI-701-wi700-current-base-revalidation
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-701-wi700-current-base-revalidation
---

[English](WI-701-wi700-current-base-revalidation.md) · [简体中文](WI-701-wi700-current-base-revalidation.zh-CN.md)

# WI-701 — WI-700 current-base revalidation

WI-701 は WI-700 の fresh recovery successor です。保持された observation-context
implementation と predecessor lineage を current remote default revision に bind し、
fresh verification、hosted lifecycle、Runtime terminal lifecycle を実行します。

## Boundary

predecessor の archive、evidence、recovery decision は書き換えません。新しい code
semantics、governance rule、protocol format、または他 agent の Work Item 変更も導入しません。
三言語 page と parity row は同じ recovery boundary の projection です。

## Acceptance

- current default base と predecessor digest が明示的に bind されること。
- 必須の observation-context scenario と locked workspace gate が pass すること。
- terminal を宣言する前に hosted quality、provider finalization、archive、close、exact cleanup
  の evidence がそろうこと。
