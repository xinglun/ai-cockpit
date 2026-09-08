---
author: AI Cockpit maintainers
title: "WI-681 — WI-674 terminal documentation promotion"
description: "クローズ済み WI-674 Repository 分割 Work Item を、管理された三言語ドキュメント投影へ反映します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-681-wi674-doc-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-681-wi674-doc-promotion
---

[English](WI-681-wi674-doc-promotion.md) · [简体中文](WI-681-wi674-doc-promotion.zh-CN.md)

# WI-681 — WI-674 terminal documentation promotion

## Intent

WI-674 の三言語 Work Item ページと reference-parity 行を、immutable archive、
verification、finalization、close record と同期します。

## Boundary

これは documentation-only projection です。変更対象は 6 つの WI-674 projection
file と 3 つの WI-681 self-registration page のみです。immutable な `.ai`
lifecycle record は read-only input とし、Runtime 挙動、governance rule、history
evidence、他の agent の worktree は変更しません。

## Acceptance

- 検証済み close 後、WI-674 の三言語 page が terminal `Implemented` status と正確な
  terminal evidence path を示すこと。
- WI-674 の三言語 parity row が一致する terminal status と evidence を示すこと。
- WI-681 自身を三言語 page と parity row に登録し、closed-work-item promotion
  check を bounded かつ repeatable に保つこと。
- 正確な reviewed head 上で documentation、parity、status-consistency、
  closed-work-item promotion check が pass すること。
- `user_visible_benefit_not_declared` を明示的に保持し、この projection から
  user-visible、performance、cognitive benefit を主張しないこと。

WI-674 terminal evidence:

- archive: `.ai/work-items/archive/WI-674-p1-repository-split.contract.json`
- verification: `.ai/evidence/WI-674-p1-repository-split.verification.json`
- finalization: `.ai/decisions/WI-674-p1-repository-split.finalize.json`
- close: `.ai/decisions/WI-674-p1-repository-split.close.json`
